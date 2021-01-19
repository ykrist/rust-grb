// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use gurobi_sys as ffi;
use itertools::{Itertools, Zip};
use std::ffi::CString;
use std::mem::transmute;
use std::ops::{Deref};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null, null_mut};
use std::hash::{Hash};
use std::slice::Iter;
use std::sync::atomic::{Ordering, AtomicU32};

use crate::{Error, Result, Env};
use crate::param;
use crate::attr;
use crate::attr::Attr;
use crate::callback::{Callback, New};
use crate::expr::{LinExpr, Expr};
use fnv::FnvHashMap;
use gurobi_sys::GRBmodel;

/// Type for new variable
#[derive(Debug, Clone, Copy)]
pub enum VarType {
  Binary,
  Continuous,
  Integer,
  SemiCont,
  SemiInt,
}

impl Into<ffi::c_char> for VarType {
  fn into(self) -> ffi::c_char {
    match self {
      VarType::Binary => 'B' as ffi::c_char,
      VarType::Continuous => 'C' as ffi::c_char,
      VarType::Integer => 'I' as ffi::c_char,
      VarType::SemiCont => 'S' as ffi::c_char,
      VarType::SemiInt => 'N' as ffi::c_char,
    }
  }
}


/// Sense for new linear/quadratic constraint
#[derive(Debug, Copy, Clone)]
pub enum ConstrSense {
  Equal,
  Greater,
  Less,
}

impl Into<ffi::c_char> for ConstrSense {
  fn into(self) -> ffi::c_char {
    match self {
      ConstrSense::Equal => '=' as ffi::c_char,
      ConstrSense::Less => '<' as ffi::c_char,
      ConstrSense::Greater => '>' as ffi::c_char,
    }
  }
}


/// Sense of new objective function
#[derive(Debug, Copy, Clone)]
pub enum ModelSense {
  Minimize = 1,
  Maximize = -1,
}

impl Into<i32> for ModelSense {
  fn into(self) -> i32 { (unsafe { transmute::<_, i8>(self) }) as i32 }
}


/// Type of new SOS constraint
#[derive(Debug, Copy, Clone)]
pub enum SOSType {
  SOSType1 = 1,
  SOSType2 = 2,
}

impl Into<i32> for SOSType {
  fn into(self) -> i32 { (unsafe { transmute::<_, i8>(self) }) as i32 }
}


/// Status of a model
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Status {
  Loaded = 1,
  Optimal,
  Infeasible,
  InfOrUnbd,
  Unbounded,
  CutOff,
  IterationLimit,
  NodeLimit,
  TimeLimit,
  SolutionLimit,
  Interrupted,
  Numeric,
  SubOptimal,
  InProgress,
}

impl From<i32> for Status {
  fn from(val: i32) -> Status {
    match val {
      1..=14 => unsafe { transmute(val as i8) },
      _ => panic!("cannot convert to Status: {}", val)
    }
  }
}

/// Type of cost function at feasibility relaxation
#[derive(Debug, Copy, Clone)]
pub enum RelaxType {
  /// The weighted magnitude of bounds and constraint violations
  /// ($penalty(s\_i) = w\_i s\_i$)
  Linear = 0,

  /// The weighted square of magnitude of bounds and constraint violations
  /// ($penalty(s\_i) = w\_i s\_i\^2$)
  Quadratic = 1,

  /// The weighted count of bounds and constraint violations
  /// ($penalty(s\_i) = w\_i \cdot [s\_i > 0]$)
  Cardinality = 2,
}

impl Into<i32> for RelaxType {
  fn into(self) -> i32 { (unsafe { transmute::<_, i8>(self) }) as i32 }
}


struct CallbackData<'a> {
  model: &'a Model,
  callback: &'a mut dyn FnMut(Callback) -> Result<()>,
}

#[allow(unused_variables)]
extern "C" fn callback_wrapper(model: *mut ffi::GRBmodel, cbdata: *mut ffi::c_void, loc: ffi::c_int,
                               usrdata: *mut ffi::c_void)
                               -> ffi::c_int {
  let usrdata = unsafe { &mut *(usrdata as *mut CallbackData) };
  let (callback, model) = (&mut usrdata.callback, &usrdata.model);

  #[allow(clippy::useless_conversion)]
  match Callback::new(cbdata, loc.into(), model) {
    Err(err) => {
      println!("failed to create context: {:?}", err);
      -3
    }
    Ok(context) => {
      match catch_unwind(AssertUnwindSafe(|| if callback(context).is_ok() { 0 } else { -1 })) {
        Ok(ret) => ret,
        Err(_e) => -3000,
      }
    }
  }
}

extern "C" fn null_callback_wrapper(_model: *mut ffi::GRBmodel, _cbdata: *mut ffi::c_void, _loc: ffi::c_int,
                                    _usrdata: *mut ffi::c_void)
                                    -> ffi::c_int {
  0
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ModelObj {
  pub(crate) id : u32,
  pub(crate) model_id: u32,
}

mod private_traits {
  use super::*;
  pub trait ModelObjectPrivate: Deref<Target=ModelObj> + Sized { // TODO remove the Deref trait bound and implement id() getters instead
    fn idx_manager_mut(model: &mut Model) -> &mut IdxManager;
    fn idx_manager(model: &Model) -> &IdxManager;
    fn new(model: &mut Model) -> Result<Self>;
    unsafe fn gurobi_remove(m: *mut GRBmodel, inds: &[i32]) -> ffi::c_int;
  }
}

use private_traits::ModelObjectPrivate;
pub trait ModelObject: ModelObjectPrivate {}


macro_rules! create_model_obj_ty {
    ($t:ident, $model_attr:ident, $delfunc:path) => {
      #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
      pub struct $t(ModelObj);

      impl ModelObjectPrivate for $t {
        fn new(model: &mut Model) -> Result<Self> {
          let update_lazy = model.update_mode_lazy()?;
          Ok(Self(model.$model_attr.add_new(update_lazy)))
        }

        fn idx_manager_mut(model: &mut Model) -> &mut IdxManager {
          &mut model.$model_attr
        }

        fn idx_manager(model: &Model) -> &IdxManager {
          &model.$model_attr
        }

        unsafe fn gurobi_remove(m: *mut GRBmodel, inds: &[i32]) -> ffi::c_int {
          $delfunc(m, inds.len() as i32, inds.as_ptr())
        }
      }

      impl Deref for $t {
       type Target = ModelObj;
       fn deref(&self) -> &ModelObj { &self.0 }
      }

      impl ModelObject for $t {}

    };
}

create_model_obj_ty!(Var, vars, ffi::GRBdelvars);
create_model_obj_ty!(Constr, constrs, ffi::GRBdelconstrs);
create_model_obj_ty!(QConstr, qconstrs, ffi::GRBdelqconstrs);
create_model_obj_ty!(SOS, sos, ffi::GRBdelsos);


enum IdxState {
  Present(i32),
  Pending,
}

pub struct IdxManager {
  dirty: bool,
  next_id: u32,
  model_id: u32,
  order: Vec<ModelObj>,
  lookup: FnvHashMap<ModelObj, IdxState>,
}

impl IdxManager {
  pub fn new(model_id: u32) -> IdxManager {
    let order = Vec::new();
    let lookup = FnvHashMap::default();
    IdxManager {order, lookup, model_id, next_id: 0, dirty: false}
  }

  pub fn get_index(&self, p: &impl Deref<Target=ModelObj>) -> Result<i32> {
    if let Some(state) = self.lookup.get(p) {
      match *state {
        IdxState::Pending => Err(Error::ModelObjectPending),
        IdxState::Present(idx) => Ok(idx)
      }
    }
    else {
      if p.model_id == self.model_id {
        Err(Error::ModelObjectRemoved)
      } else {
        Err(Error::ModelObjectMismatch)
      }
    }
  }

  pub fn remove(&mut self, p: ModelObj, update_lazy: bool) -> Result<()> {
    if p.model_id != self.model_id {
      return Err(Error::ModelObjectMismatch)
    }
    if self.lookup.remove(&p).is_none() {
      return Err(Error::ModelObjectRemoved)
    }
    self.dirty = true;
    if update_lazy {
      self.update();
    }
    Ok(())
  }

  pub fn add_new(&mut self, update_lazy: bool) -> ModelObj {
    let o = ModelObj{ id: self.next_id, model_id: self.model_id };
    self.next_id += 1;
    let state = if update_lazy {
      self.dirty = true;
      IdxState::Pending
    } else {
      debug_assert_eq!(self.lookup.len(), self.order.len()); // should be no remove vars
      IdxState::Present(self.lookup.len() as i32)
    };
    self.lookup.insert(o, state);
    self.order.push(o);
    o
  }

  fn update(&mut self) {
    if !self.dirty {
      return;
    }
    use std::collections::hash_map::Entry;
    let mut k = 0i32;
    let order = &mut self.order;
    let lookup = &mut self.lookup;
    order.retain(|p| {
      match lookup.entry(p.clone()) {
        Entry::Occupied(mut e) => {
          e.insert(IdxState::Present(k));
          k+=1;
          true
        }
        Entry::Vacant(_) => false
      }
    });
    debug_assert_eq!(k as usize, self.lookup.len());
    debug_assert_eq!(k as usize, self.order.len());
    self.dirty = false;
  }

  pub fn is_empty(&self) -> bool { self.lookup.is_empty() }

  pub fn len(&self) -> usize {
    assert!(!self.dirty);
    self.lookup.len()
  }
}


/// Gurobi model object associated with certain environment.
pub struct Model {
  model: *mut ffi::GRBmodel,
  id: u32,
  env: Env,
  pub(crate) vars: IdxManager,
  constrs: IdxManager,
  qconstrs: IdxManager,
  sos: IdxManager,
}


// macro_rules! model_create_proxy_impl {
//     ($p_ty:ty, $attr:ident, $method_name:ident, $batch_method_name:ident) => {
//       fn $method_name(&mut self) -> Result<$p_ty> {
//         if self.update_mode_lazy()? {
//           Ok(<$p_ty>::new(IndexState::PendingAdd, &self))
//         } else {
//           let obj = <$p_ty>::new(IndexState::Added(self.$attr.len() as u32), &self);
//           self.update()?;
//           Ok(obj)
//         }
//       }
//
//       fn $batch_method_name(&mut self, num: u32) -> Result<Vec<$p_ty>> {
//         if self.update_mode_lazy()? {
//           Ok(vec![<$p_ty>::new(IndexState::PendingAdd, &self); num as usize])
//         } else {
//           if num == 0 {
//             Ok(Vec::new())
//           } else {
//             let start_idx = self.$attr.len() as u32;
//             let objs = (start_idx..start_idx+num).map(|idx| <$p_ty>::new(IndexState::Added(idx), &self)).collect();
//             self.update()?;
//             Ok(objs)
//           }
//       }
//     }
//   }
// }

/// Helper function to convert LinExpr objects into Compressed Sparse Row (CSR) format
fn csr_format(expr: Vec<LinExpr>) -> Result<(Vec<i32>, Vec<i32>, Vec<f64>)> {
  todo!();
  // let lhs: Vec<(_, _)> = expr.into_iter().map(|e| e.into_parts()).collect();
  //
  // let mut constr_index_end = Vec::with_capacity(lhs.len());
  // let mut cumulative_nz = 0;
  //
  // for (coeff, _) in lhs.iter() {
  //   cumulative_nz += coeff.len();
  //   constr_index_end.push(cumulative_nz as i32);
  // }
  //
  // let mut variable_indices = Vec::with_capacity(cumulative_nz);
  // let mut coeff = Vec::with_capacity(cumulative_nz);
  // for (coeffs, _) in lhs {
  //   for (x, a) in coeffs {
  //     // variable_indices.push(x.index()? as i32); // fixme
  //     coeff.push(a);
  //   }
  // }
  // // FIXME, bug: should return offsets
  // Ok((constr_index_end, variable_indices, coeff))
}

fn convert_to_cstring_ptrs(strings: &Vec<&str>) -> Result<Vec<*const ffi::c_char>> {
  strings.iter().map(|&s| {
    let s = CString::new(s)?;
    Ok(s.as_ptr())
  }).collect()
}


impl Model {
  /// Create an empty Gurobi model from the environment.
  ///
  /// Note that the given environment will be copied by the Gurobi API
  /// and a new environment associated with the model will be created.
  /// If you want to query/modify the value of parameters, use `get_env()`/`get_env_mut()`.
  ///
  /// # Arguments
  /// * __modelname__ : Name of the model
  /// * __env__ : An environment object.
  ///
  /// # Example
  /// ```
  /// use gurobi::*;
  ///
  /// let mut env = gurobi::Env::new("").unwrap();
  /// env.set(param::OutputFlag, 0).unwrap();
  /// env.set(param::Heuristics, 0.5).unwrap();
  /// let env = env;
  /// // ...
  ///
  /// let mut model = Model::new("model1", &env).unwrap();
  /// assert_eq!(model.get_env().get(param::OutputFlag).unwrap(), 0);
  ///
  /// model.get_env_mut().set(param::OutputFlag, 1).unwrap();
  /// // ...
  /// assert_eq!(model.get_env().get(param::OutputFlag).unwrap(), 1);
  /// assert_eq!(env.get(param::OutputFlag).unwrap(), 0); // original env is copied
  /// ```

  fn next_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    eprintln!("New Model id created: {}", id);
    id
  }

  pub fn new(modelname: &str, env: &Env) -> Result<Model> {
    let modelname = CString::new(modelname)?;
    let mut model = null_mut();
    env.check_apicall(unsafe {
      ffi::GRBnewmodel(env.get_ptr(),
                       &mut model,
                       modelname.as_ptr(),
                       0,
                       null(),
                       null(),
                       null(),
                       null(),
                       null())
    })?;
    Self::from_raw(model)
  }

  /// create an empty model which associated with certain environment.
  fn from_raw(model: *mut ffi::GRBmodel) -> Result<Model> {
    let env = unsafe { ffi::GRBgetenv(model) };
    if env.is_null() {
      return Err(Error::FromAPI("Failed to retrieve GRBenv from given model".to_owned(),
                                2002));
    }
    let env = Env::from_raw(env);
    let id=Model::next_id();
    let mut model = Model {
      model,
      id: id,
      env,
      vars: IdxManager::new(id),
      constrs: IdxManager::new(id),
      qconstrs: IdxManager::new(id),
      sos: IdxManager::new(id),
    };
    model.populate()?;
    Ok(model)
  }

  /// Read a model from a file
  pub fn read_from(filename: &str, env: &Env) -> Result<Model> {
    let filename = CString::new(filename)?;
    let mut model = null_mut();
    env.check_apicall(unsafe { ffi::GRBreadmodel(env.get_ptr(), filename.as_ptr(), &mut model) })?;
    Self::from_raw(model)
  }

  /// create a copy of the model
  pub fn copy(&self) -> Result<Model> {
    let copied = unsafe { ffi::GRBcopymodel(self.model) };
    if copied.is_null() {
      return Err(Error::FromAPI("Failed to create a copy of the model".to_owned(), 20002));
    }

    Model::from_raw(copied)
  }

  pub(crate) fn get_index<O: ModelObject>(&self, item: &O) -> Result<i32> {
    O::idx_manager(&self).get_index(item)
  }

  #[inline]
  pub(crate) fn get_indices(&self, items: &[impl ModelObject]) -> Result<Vec<i32>> {
    items.iter().map(|item| self.get_index(item)).collect()
  }

  #[inline]
  pub(crate) fn get_indices_ref<O: ModelObject>(&self, items: &[&O]) -> Result<Vec<i32>> {
    items.iter().map(|&item| self.get_index(item)).collect()
  }


  // model_create_proxy_impl!(Var, vars, create_var_proxy, create_var_proxies);
  // model_create_proxy_impl!(Constr, constrs, create_constr_proxy, create_constr_proxies);
  // model_create_proxy_impl!(QConstr, qconstrs, create_qconstr_proxy, create_qconstr_proxies);
  // model_create_proxy_impl!(SOS, sos, create_sos_proxy, create_sos_proxies);

  /// Create an fixed model associated with the model.
  ///
  /// In fixed model, each integer variable is fixed to the value that it takes in the
  /// original MIP solution.
  /// Note that the model must be MIP and have a solution loaded.
  pub fn fixed(&self) -> Result<Model> {
    let fixed = unsafe { ffi::GRBfixedmodel(self.model) };
    if fixed.is_null() {
      return Err(Error::FromAPI("failed to create fixed model".to_owned(), 20002));
    }
    Model::from_raw(fixed)
  }

  /// Create an relaxation of the model (undocumented).
  pub fn relax(&self) -> Result<Model> {
    let relaxed = unsafe { ffi::GRBrelaxmodel(self.model) };
    if relaxed.is_null() {
      return Err(Error::FromAPI("failed to create relaxed model".to_owned(), 20002));
    }
    Model::from_raw(relaxed)
  }

  /// Perform presolve on the model.
  pub fn presolve(&self) -> Result<Model> {
    let presolved = unsafe { ffi::GRBpresolvemodel(self.model) };
    if presolved.is_null() {
      return Err(Error::FromAPI("failed to create presolved model".to_owned(), 20002));
    }
    Model::from_raw(presolved)
  }

  /// Create a feasibility model (undocumented).
  pub fn feasibility(&self) -> Result<Model> {
    let feasibility = unsafe { ffi::GRBfeasibility(self.model) };
    if feasibility.is_null() {
      return Err(Error::FromAPI("failed to create feasibility model".to_owned(), 20002));
    }
    Model::from_raw(feasibility)
  }

  /// Get immutable reference of an environment object associated with the model.
  pub fn get_env(&self) -> &Env { &self.env }

  /// Get mutable reference of an environment object associated with the model.
  pub fn get_env_mut(&mut self) -> &mut Env { &mut self.env }

  /// Apply all queued modification of the model
  pub fn update(&mut self) -> Result<()> {
    self.vars.update();
    self.constrs.update();
    self.qconstrs.update();
    self.sos.update();
    self.check_apicall(unsafe { ffi::GRBupdatemodel(self.model) })?;
    Ok(())
  }

  /// Query update mode. See [https://www.gurobi.com/documentation/9.1/refman/updatemode.html]
  fn update_mode_lazy(&self) -> Result<bool> {
    //  0 => pending until update() or optimize() called.
    //  1 => all changes are immediate
    Ok(self.env.get(param::UpdateMode)? == 0)
  }

  /// Optimize the model synchronously
  pub fn optimize(&mut self) -> Result<()> {
    self.update()?;
    self.check_apicall(unsafe { ffi::GRBoptimize(self.model) })
  }

  /// Optimize the model asynchronously
  pub fn optimize_async(&mut self) -> Result<()> {
    self.update()?;
    self.check_apicall(unsafe { ffi::GRBoptimizeasync(self.model) })
  }

  /// Optimize the model with a callback function
  pub fn optimize_with_callback<F>(&mut self, mut callback: F) -> Result<()>
    where F: FnMut(Callback) -> Result<()> + 'static
  {
    self.update()?;
    let usrdata = CallbackData {
      model: self,
      callback: &mut callback,
    };
    self.check_apicall(unsafe { ffi::GRBsetcallbackfunc(self.model, callback_wrapper, transmute(&usrdata)) })?;

    self.check_apicall(unsafe { ffi::GRBoptimize(self.model) })?;

    // clear callback from the model.
    // Notice: Rust does not have approproate mechanism which treats "null" C-style function
    // pointer.
    self.check_apicall(unsafe { ffi::GRBsetcallbackfunc(self.model, null_callback_wrapper, null_mut()) })
  }

  /// Wait for a optimization called asynchronously.
  pub fn sync(&self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBsync(self.model) }) }

  /// Compute an Irreducible Inconsistent Subsystem (IIS) of the model.
  pub fn compute_iis(&mut self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBcomputeIIS(self.model) }) }

  /// Send a request to the model to terminate the current optimization process.
  pub fn terminate(&self) { unsafe { ffi::GRBterminate(self.model) } }

  /// Reset the model to an unsolved state.
  ///
  /// All solution information previously computed are discarded.
  pub fn reset(&self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBresetmodel(self.model) }) }

  /// Perform an automated search for parameter settings that improve performance on the model.
  /// See also references [on official
  /// manual](https://www.gurobi.com/documentation/6.5/refman/parameter_tuning_tool.html#sec:Tuning).
  pub fn tune(&self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBtunemodel(self.model) }) }

  /// Prepare to retrieve the results of `tune()`.
  /// See also references [on official
  /// manual](https://www.gurobi.com/documentation/6.5/refman/parameter_tuning_tool.html#sec:Tuning).
  pub fn get_tune_result(&self, n: i32) -> Result<()> {
    self.check_apicall(unsafe { ffi::GRBgettuneresult(self.model, n) })
  }

  /// Create/retrieve a concurrent environment for the model
  ///
  /// Note that the number of concurrent environments (`num`) must be contiguously numbered.
  ///
  /// # Example
  /// ```ignore
  /// let env1 = model.get_concurrent_env(0).unwrap();
  /// let env2 = model.get_concurrent_env(1).unwrap();
  /// let env3 = model.get_concurrent_env(2).unwrap();
  /// ...
  /// ```
  #[deprecated]
  pub fn get_concurrent_env(&self, num: i32) -> Result<Env> {
    let env = unsafe { ffi::GRBgetconcurrentenv(self.model, num) };
    if env.is_null() {
      return Err(Error::FromAPI("Cannot get a concurrent environment.".to_owned(), 20003));
    }
    Ok(Env::from_raw(env))
  }

  /// Discard all concurrent environments for the model.
  #[deprecated]
  pub fn discard_concurrent_envs(&self) { unsafe { ffi::GRBdiscardconcurrentenvs(self.model) } }

  /// Insert a message into log file.
  ///
  /// When **message** cannot convert to raw C string, a panic is occurred.
  pub fn message(&self, message: &str) { self.env.message(message); }

  /// Import optimization data of the model from a file.
  pub fn read(&mut self, filename: &str) -> Result<()> {
    let filename = CString::new(filename)?;
    self.check_apicall(unsafe { ffi::GRBread(self.model, filename.as_ptr()) })
  }

  /// Export optimization data of the model to a file.
  pub fn write(&self, filename: &str) -> Result<()> {
    let filename = CString::new(filename)?;
    self.check_apicall(unsafe { ffi::GRBwrite(self.model, filename.as_ptr()) })
  }


  /// add a decision variable to the model.
  pub fn add_var(&mut self, name: &str, vtype: VarType, obj: f64, lb: f64, ub: f64, colconstrs: &[Constr],
                 colvals: &[f64])
                 -> Result<Var> {
    if colconstrs.len() != colvals.len() {
      return Err(Error::InconsistentDims);
    }
    let colconstrs = self.get_indices(colconstrs)?;
    let name = CString::new(name)?;
    self.check_apicall(unsafe {
      ffi::GRBaddvar(self.model,
                     colvals.len() as ffi::c_int,
                     colconstrs.as_ptr(),
                     colvals.as_ptr(),
                     obj,
                     lb,
                     ub,
                     vtype.into(),
                     name.as_ptr())
    })?;

    Ok(Var::new(self)?)
  }

  /// add decision variables to the model.
  pub fn add_vars(&mut self, names: &[&str], vtypes: &[VarType], objs: &[f64], lbs: &[f64], ubs: &[f64],
                  colconstrs: &[&[Constr]], colvals: &[&[f64]])
                  -> Result<Vec<Var>> {
    if names.len() != vtypes.len() || vtypes.len() != objs.len() || objs.len() != lbs.len() ||
      lbs.len() != ubs.len() || ubs.len() != colconstrs.len() || colconstrs.len() != colvals.len() {
      return Err(Error::InconsistentDims);
    }

    let names = {
      let mut buf = Vec::with_capacity(names.len());
      for &name in names.iter() {
        let name = CString::new(name)?;
        buf.push(name.as_ptr());
      }
      buf
    };

    let vtypes = {
      let mut buf = Vec::with_capacity(vtypes.len());
      for &vtype in vtypes.iter() {
        let vtype = vtype.into();
        buf.push(vtype);
      }
      buf
    };

    let (beg, ind, val) = {
      let len_ind = colconstrs.iter().map(|constrs| constrs.len()).sum();
      let mut buf_beg = Vec::with_capacity(colconstrs.len());
      let mut buf_ind = Vec::with_capacity(len_ind);
      let mut buf_val: Vec<f64> = Vec::with_capacity(len_ind);

      let mut beg = 0i32;
      for (constrs, &vals) in Zip::new((colconstrs, colvals)) {
        if constrs.len() != vals.len() {
          return Err(Error::InconsistentDims);
        }
        buf_beg.push(beg);
        beg += constrs.len() as i32;
        for c in constrs.iter() {
          buf_ind.push(self.get_index(c)?)
        }
        buf_val.extend_from_slice(vals);
      }
      (buf_beg, buf_ind, buf_val)
    };

    self.check_apicall(unsafe {
      ffi::GRBaddvars(self.model,
                      names.len() as ffi::c_int,
                      beg.len() as ffi::c_int,
                      beg.as_ptr(),
                      ind.as_ptr(),
                      val.as_ptr(),
                      objs.as_ptr(),
                      lbs.as_ptr(),
                      ubs.as_ptr(),
                      vtypes.as_ptr(),
                      names.as_ptr())
    })?;

    Ok(vec![Var::new(self)?; names.len()])
  }


  /// add a linear constraint to the model.
  pub fn add_constr(&mut self, name: &str, expr: Expr, sense: ConstrSense, rhs: f64) -> Result<Constr> {
    let expr = expr.into_linexpr()?;
    let constrname = CString::new(name)?;
    let offset = expr.get_offset();
    let (vinds, cval) = expr.get_coeff_indices(&self)?;
    self.check_apicall(unsafe {
      ffi::GRBaddconstr(self.model,
                        cval.len() as ffi::c_int,
                        vinds.as_ptr(),
                        cval.as_ptr(),
                        sense.into(),
                        rhs - offset,
                        constrname.as_ptr())
    })?;

    Ok(Constr::new(self)?)
  }


  /// add linear constraints to the model.
  pub fn add_constrs(&mut self, names: Vec<&str>, lhs: Vec<Expr>, sense: Vec<ConstrSense>, mut rhs: Vec<f64>) -> Result<Vec<Constr>> {
    if !(names.len() == lhs.len() && lhs.len() == sense.len() && sense.len() == rhs.len()) {
      return Err(Error::InconsistentDims);
    }
    let lhs : Result<Vec<LinExpr>>= lhs.into_iter().map(|e| e.into_linexpr()).collect();
    let lhs = lhs?;
    let sense = sense.iter().map(|&s| s.into()).collect_vec();
    rhs.iter_mut().zip(lhs.iter()).for_each(|(rhs, lhs)| *rhs -= lhs.get_offset());
    let constrnames = convert_to_cstring_ptrs(&names)?;
    let (cbeg, cind, cval) = csr_format(lhs)?;

    self.check_apicall(unsafe {
      ffi::GRBaddconstrs(self.model,
                         constrnames.len() as ffi::c_int,
                         cbeg.len() as ffi::c_int,
                         cbeg.as_ptr(),
                         cind.as_ptr(),
                         cval.as_ptr(),
                         sense.as_ptr(),
                         rhs.as_ptr(),
                         constrnames.as_ptr())
    })?;

    Ok(vec![Constr::new(self)?; constrnames.len()])
  }

  /// Add a range constraint to the model.
  ///
  /// This operation adds a decision variable with lower/upper bound, and a linear
  /// equality constraint which states that the value of variable must equal to `expr`.
  ///
  /// # Returns
  /// * An decision variable associated with the model. It has lower/upper bound constraints.
  /// * An linear equality constraint associated with the model.
  pub fn add_range(&mut self, name: &str, expr: LinExpr, lb: f64, ub: f64) -> Result<(Var, Constr)> {
    let constrname = CString::new(name)?;
    let offset = expr.get_offset();
    let (inds, coeff) = expr.get_coeff_indices(&self)?;
    self.check_apicall(unsafe {
      ffi::GRBaddrangeconstr(self.model,
                             coeff.len() as ffi::c_int,
                             inds.as_ptr(),
                             coeff.as_ptr(),
                             lb - offset,
                             ub - offset,
                             constrname.as_ptr())
    })?;

    let var = Var::new(self)?;
    let cons = Constr::new(self)?;
    Ok((var, cons))
  }

  #[allow(unused_variables)]
  /// Add range constraints to the model.
  pub fn add_ranges(&mut self, names: Vec<&str>, expr: Vec<LinExpr>, mut lb: Vec<f64>, mut ub: Vec<f64>)
                    -> Result<(Vec<Var>, Vec<Constr>)> {
    let constrnames = convert_to_cstring_ptrs(&names)?;
    ub.iter_mut().zip(expr.iter()).for_each(|(x, e)| *x -= e.get_offset());
    lb.iter_mut().zip(expr.iter()).for_each(|(x, e)| *x -= e.get_offset());
    let (cbeg, cind, cval) = csr_format(expr)?;

    self.check_apicall(unsafe {
      ffi::GRBaddrangeconstrs(self.model,
                              constrnames.len() as ffi::c_int,
                              cbeg.len() as ffi::c_int,
                              cbeg.as_ptr(),
                              cind.as_ptr(),
                              cval.as_ptr(),
                              lb.as_ptr(),
                              ub.as_ptr(),
                              constrnames.as_ptr())
    })?;

    let ncons = names.len();
    let vars = vec![Var::new(self)?; ncons];
    let cons = vec![Constr::new(self)?; ncons];
    Ok((vars, cons))
  }

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self, constrname: &str, expr: Expr, sense: ConstrSense, rhs: f64) -> Result<QConstr> {
    let constrname = CString::new(constrname)?;
    let expr = expr.into_quadexpr();
    let offset = expr.get_offset();
    let (qrow, qcol, qval) = expr.get_qcoeff_indices(&self)?;
    let (_, expr) = expr.into_parts();
    let (lvar, lval) = expr.get_coeff_indices(&self)?;
    self.check_apicall(unsafe {
      ffi::GRBaddqconstr(self.model,
                         lval.len() as ffi::c_int,
                         lvar.as_ptr(),
                         lval.as_ptr(),
                         qval.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr(),
                         sense.into(),
                         rhs - offset,
                         constrname.as_ptr())
    })?;

    Ok(QConstr::new(self)?)
  }

  /// add Special Order Set (SOS) constraint to the model.
  pub fn add_sos(&mut self, vars: &[Var], weights: &[f64], sostype: SOSType) -> Result<SOS> {
    if vars.len() != weights.len() {
      return Err(Error::InconsistentDims);
    }

    let vars = self.get_indices(vars)?;
    let beg = 0;

    self.check_apicall(unsafe {
      ffi::GRBaddsos(self.model,
                     1,
                     vars.len() as ffi::c_int,
                     &sostype.into(),
                     &beg,
                     vars.as_ptr(),
                     weights.as_ptr())
    })?;

    Ok(SOS::new(self)?)
  }

  /// Set the objective function of the model.
  pub fn set_objective(&mut self, expr: impl Into<Expr>, sense: ModelSense) -> Result<()> {
    // if self.updatemode.is_some() {
    //   return Err(Error::FromAPI("The objective function cannot be set before any pending modifies existed".to_owned(),
    //                             50000));
    // }
    let expr = expr.into().into_quadexpr();
    let (qrow, qcol, qval) = expr.get_qcoeff_indices(&self)?;
    self.del_qpterms()?;
    self.add_qpterms(&qrow, &qcol, &qval)?;
    let (_, expr) = expr.into_parts();
    let (obj_vals, _) = expr.into_parts();
    let mut vars = Vec::with_capacity(obj_vals.len());
    let mut vals = Vec::with_capacity(obj_vals.len());
    for (var, c) in obj_vals {
      vars.push(var);
      vals.push(c);
    }
    self.set_obj_attr_batch(attr::Obj, &vars, &vals)?;
    self.set_attr(attr::ModelSense, sense.into())
  }


  pub fn get_constr_by_name(&self, name: &str) -> Result<Constr> {
    todo!()
  }

  pub fn get_var_by_name(&self, name: &str) -> Result<Var> {
    todo!()
  }

  /// Query a Model attribute
  pub fn get_attr<A: Attr>(&self, attr: A) -> Result<A::Value> {
    unsafe { attr.get(self.model) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Query a model object attribute (Constr, Var, etc)
  pub fn get_obj_attr<A,E>(&self, attr: A, elem: &E) -> Result<A::Value>
  where
    A: Attr,
    E: ModelObject
  {
    let index = self.get_index(elem)?;
    unsafe { attr.get_element(self.model, index) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Query an attribute of multiple model objectis
  pub fn get_obj_attr_batch<A,E>(&self, attr: A, elem: &[E]) -> Result<Vec<A::Value>>
    where
        A: Attr,
        E: ModelObject
  {
    let index = self.get_indices(elem)?;
    unsafe { attr.get_elements(self.model, &index) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Set a Model attribute
  pub fn set_attr<A: Attr>(&self, attr: A, value: A::Value) -> Result<()> {
    unsafe { attr.set(self.model, value) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Set an attribute of a Model object (Const, Var, etc)
  pub fn set_obj_attr<A,E>(&self, attr: A, elem: &E, value: A::Value) -> Result<()>
    where
        A: Attr,
        E: ModelObject
  {
    let index = self.get_index(elem)?;
    unsafe { attr.set_element(self.model, index, value) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Set an attribute of multiple Model objects (Const, Var, etc)
  pub fn set_obj_attr_batch<A,E>(&self, attr: A, elem: &[E], values: &[A::Value]) -> Result<()>
    where
        A: Attr,
        E: ModelObject
  {
    if elem.len() != values.len() {
      return Err(Error::InconsistentDims);
    }
    let indices = self.get_indices(elem)?;
    unsafe { attr.set_elements(self.model, &indices, values) }.map_err(|code| self.env.error_from_api(code))
  }



  /// Modify the model to create a feasibility relaxation.
  ///
  /// $$
  ///   \text{minimize}\quad f(x) + \sum_{i \in IIS} penalty_i(s_i)
  /// $$
  /// where $s\_i > 0$ is the slack variable of $i$ -th constraint.
  ///
  /// This method will modify the model.
  /// If you don't want to modify the model, copy the model before invoking
  /// this method (see also [`copy()`](#method.copy)).
  ///
  /// ## Arguments
  /// * `relaxtype` : The type of cost function used when finding the minimum cost relaxation.
  ///   See also [`RelaxType`](enum.RelaxType.html).
  /// * `minrelax` : The type of feasibility relaxation to perform.
  /// * `vars` : Variables whose bounds are allowed to be violated.
  /// * `lbpen` / `ubpen` : Penalty for violating a variable lower/upper bound.
  ///   `INFINITY` means that the bounds doesn't allow to be violated.
  /// * `constrs` : Linear constraints that are allowed to be violated.
  /// * `rhspen` : Penalty for violating a linear constraint.
  ///   `INFINITY` means that the bounds doesn't allow to be violated.
  ///
  /// ## Returns
  /// * The objective value for the relaxation performed (if `minrelax` is `true`).
  /// * Slack variables for relaxation and related linear/quadratic constraints.
  #[allow(clippy::type_complexity)]
  pub fn feas_relax(&mut self, relaxtype: RelaxType, minrelax: bool, vars: &[Var], lbpen: &[f64], ubpen: &[f64],
                    constrs: &[Constr], rhspen: &[f64])
                    -> Result<(f64, Vec<Var>, Vec<Constr>, Vec<QConstr>)> {
    if vars.len() != lbpen.len() || vars.len() != ubpen.len() {
      return Err(Error::InconsistentDims);
    }

    if constrs.len() != rhspen.len() {
      return Err(Error::InconsistentDims);
    }


    let (pen_lb, pen_ub) = if vars.is_empty() {
      // Gurobi API docs allow for this optimisation
      (std::ptr::null(), std::ptr::null())
    } else {
      let mut pen_lb = vec![super::INFINITY; self.vars.len()];
      let mut pen_ub = vec![super::INFINITY; self.vars.len()];
      for (v, &lb, &ub) in Zip::new((vars, lbpen, ubpen)) {
        let idx = self.get_index(v)? as usize;
        if idx >= self.vars.len() {
          return Err(Error::InconsistentDims); // FIXME is this needed?
        }
        pen_lb[idx] = lb;
        pen_ub[idx] = ub;
      }
      (pen_lb.as_ptr(), pen_ub.as_ptr())
    };

    let pen_rhs = if constrs.is_empty() {
      std::ptr::null()
    } else {
      let mut pen_rhs = vec![super::INFINITY; self.constrs.len()];
      for (c, &rhs) in Zip::new((constrs, rhspen)) {
        let idx = self.get_index(c)? as usize;
        if idx >= self.constrs.len() {
          return Err(Error::InconsistentDims);
        }

        pen_rhs[idx] = rhs;
      }
      pen_rhs.as_ptr()
    };

    let minrelax = if minrelax { 1 } else { 0 };

    let feasobj = 0f64;
    self.check_apicall(unsafe {
      ffi::GRBfeasrelax(self.model,
                        relaxtype.into(),
                        minrelax,
                        pen_lb,
                        pen_ub,
                        pen_rhs,
                        &feasobj)
    })?;
    self.update()?;

    let n_vars = self.get_attr(attr::NumVars)? as usize;
    assert!(n_vars >= self.vars.len());
    let num_new_vars = n_vars - self.vars.len();
    let new_vars = vec![Var::new(self)?; num_new_vars];

    let n_cons = self.get_attr(attr::NumConstrs)? as usize;
    assert!(n_cons >= self.constrs.len());
    let num_new_cons = n_cons - self.constrs.len();
    let new_cons = vec![Constr::new(self)?; num_new_cons];

    let n_qcons = self.get_attr(attr::NumQConstrs)? as usize;
    assert!(n_qcons >= self.qconstrs.len());
    let num_new_qcons = n_qcons - self.qconstrs.len();
    let new_qcons = vec![QConstr::new(self)?; num_new_qcons];


    Ok((feasobj, new_vars, new_cons, new_qcons))
  }

  /// Set a piecewise-linear objective function for the variable.
  /// 
  /// The piecewise-linear objective function $f(x)$ is defined as follows:
  /// \begin{align}
  ///   f(x) = 
  ///   \begin{cases}
  ///     y_1 + \dfrac{y_2 - y_1}{x_2 - x_1} \\, (x - x_1)         & \text{if $x \leq x_1$}, \\\\
  ///   \\\\
  ///     y_i + \dfrac{y_{i+1} - y_i}{x_{i+1}-x_i} \\, (x - x_i)   & \text{if $x_i \leq x \leq x_{i+1}$}, \\\\
  ///   \\\\
  ///     y_n + \dfrac{y_n - y_{n-1}}{x_n-x_{n-1}} \\, (x - x_n)   & \text{if $x \geq x_n$},
  ///   \end{cases}
  /// \end{align}
  /// where $\bm{x} = \\{ x_1, ..., x_n \\}$, $\bm{y} = \\{ y_1, ..., y_n \\}$ is the points.
  ///
  /// The attribute `Obj` will be set to 0.
  /// To delete the piecewise-linear function on the variable, set the value of `Obj` attribute to non-zero.
  ///
  /// # Arguments
  /// * `var` :
  /// * `x` : $n$-points from domain of the variable. The order of entries should be
  /// non-decreasing.
  /// * `y` : $n$-points of objective values at each point $x_i$
  pub fn set_pwl_obj(&mut self, var: &Var, x: &[f64], y: &[f64]) -> Result<()> {
    if x.len() != y.len() {
      return Err(Error::InconsistentDims);
    }
    self.check_apicall(unsafe {
      ffi::GRBsetpwlobj(self.model,
                        self.get_index(var)?,
                        x.len() as ffi::c_int,
                        x.as_ptr(),
                        y.as_ptr())
    })?;
    self.update()
  }

  /// Retrieve the status of the model.
  pub fn status(&self) -> Result<Status> { self.get_attr(attr::Status).map(|val| val.into()) }

  /// Retrieve an iterator of the variables in the model.
  pub fn get_vars(&self) -> Iter<Var> { todo!() }

  /// Retrieve an iterator of the linear constraints in the model.
  pub fn get_constrs(&self) -> Iter<Constr> { todo!() }

  /// Retrieve an iterator of the quadratic constraints in the model.
  pub fn get_qconstrs(&self) -> Iter<QConstr> { todo!() }

  /// Retrieve an iterator of the special order set (SOS) constraints in the model.
  pub fn get_sos(&self) -> Iter<SOS> { todo!() }

  // FIXME, bug - the item should update according to self.updatemode
  /// Remove a variable from the model.
  pub fn remove<O: ModelObject>(&mut self, item: O) -> Result<()> {
    let lazy = self.update_mode_lazy()?;
    let im = O::idx_manager_mut(self);
    let idx = im.get_index(&item)?;
    im.remove(*item, lazy)?;
    self.check_apicall(unsafe {O::gurobi_remove(self.model, &[idx]) })
  }


  /// Retrieve a single constant matrix coefficient of the model.
  pub fn get_coeff(&self, var: &Var, constr: &Constr) -> Result<f64> {
    let mut value = 0.0;
    self.check_apicall(unsafe {
      ffi::GRBgetcoeff(self.model, self.get_index(constr)?,self.get_index(var)?, &mut value)
    })?;
    Ok(value)
  }

  /// Change a single constant matrix coefficient of the model.
  pub fn set_coeff(&mut self, var: &Var, constr: &Constr, value: f64) -> Result<()> {
    self.check_apicall(unsafe {
      ffi::GRBchgcoeffs(self.model, 1, &self.get_index(constr)?, &self.get_index(var)?, &value)
    })
  }

  /// Change a set of constant matrix coefficients of the model.
  pub fn set_coeffs(&mut self, vars: &[&Var], constrs: &[&Constr], values: &[f64]) -> Result<()> {
    if vars.len() != values.len() || constrs.len() != values.len() {
      return Err(Error::InconsistentDims);
    }

    let vars = self.get_indices_ref(&vars)?;
    let constrs = self.get_indices_ref(&constrs)?;

    self.check_apicall(unsafe {
      ffi::GRBchgcoeffs(self.model,
                        vars.len() as ffi::c_int,
                        constrs.as_ptr(),
                        vars.as_ptr(),
                        values.as_ptr())
    })
  }

  fn populate(&mut self) -> Result<()> {
    assert!(self.vars.is_empty());
    assert!(self.constrs.is_empty());
    assert!(self.qconstrs.is_empty());
    assert!(self.sos.is_empty());
    let update_lazy = self.update_mode_lazy()?;
    for _ in 0..self.get_attr(attr::NumVars)? {
      self.vars.add_new(update_lazy);
    }
    for _ in 0..self.get_attr(attr::NumConstrs)? {
      self.constrs.add_new(update_lazy);
    }
    for _ in 0..self.get_attr(attr::NumQConstrs)? {
      self.qconstrs.add_new(update_lazy);
    }
    for _ in 0..self.get_attr(attr::NumSOS)? {
      self.sos.add_new(update_lazy);
    }
    Ok(())
  }


  // add quadratic terms of objective function.
  fn add_qpterms(&mut self, qrow: &[i32], qcol: &[i32], qval: &[f64]) -> Result<()> {
    self.check_apicall(unsafe {
      ffi::GRBaddqpterms(self.model,
                         qrow.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr())
    })?;
    if !self.update_mode_lazy()? {
      self.update()?;
    }
    Ok(())
  }

  // remove quadratic terms of objective function.
  fn del_qpterms(&mut self) -> Result<()> {
    self.check_apicall(unsafe { ffi::GRBdelq(self.model) })?;
    self.update()
  }

  pub(crate) fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
    if error != 0 {
      return Err(self.env.error_from_api(error));
    }
    Ok(())
  }
}


impl Drop for Model {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreemodel(self.model) };
    self.model = null_mut();
  }
}


#[cfg(test)]
mod tests {
  use super::super::*;
  use crate::model::ModelObj;

  #[test]
  fn modelsense_conversion() {
    use self::ModelSense;
    assert_eq!(Into::<i32>::into(ModelSense::Minimize), 1i32);
    assert_eq!(Into::<i32>::into(ModelSense::Maximize), -1i32);
  }

  #[test]
  fn model_id_factory() {
    let mut env = Env::new("").unwrap();
    env.set(param::OutputFlag, 0).unwrap();

    let mut m1 = Model::new("test1", &env).unwrap();
    let mut m2 = Model::new("test2", &env).unwrap();

    let x1 = m1.add_var("x1", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let x2 = m2.add_var("x2", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_ne!(m1.id, m2.id);

    assert_eq!(x1.model_id, m1.id);
    assert_eq!(x1.id, 0);
    assert_eq!(x2.model_id, m2.id);
    assert_eq!(x2.id, 0);
  }


  #[test]
  fn remove_and_add_variable_eager_update() {
    let mut env = Env::new("").unwrap();
    env.set(param::OutputFlag, 0).unwrap();
    assert_eq!(env.get(param::OutputFlag).unwrap(), 0);
    dbg!(env.get(param::OutputFlag));
    assert_eq!(env.get(param::UpdateMode).unwrap(), 1);

    let mut model = Model::new("hoge", &env).unwrap();
    assert!(!model.update_mode_lazy().unwrap());
    let x = model.add_var("x", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let y = model.add_var("y", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let idx = |m : &Model, var| m.get_index(&var).unwrap();

    // model.update().unwrap();
    dbg!(model.get_obj_attr(attr::VarName, &x).unwrap());
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(idx(&model, x), 0);
    assert_eq!(idx(&model, y), 1);

    model.update().unwrap(); // should have no effect
    assert_eq!(idx(&model, x), 0);
    assert_eq!(idx(&model, y), 1);

    let z = model.add_var("z", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 3);
    // assert_eq!(x.index().unwrap(), 0);
    // assert_eq!(y.index().unwrap(), 1);
    // assert_eq!(z.index().unwrap(), 2);

    model.remove(y.clone()).unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    // assert_eq!(x.index().unwrap(), 0);
    // assert_eq!(y.index(), Err(Error::ModelObjectRemoved)); // No longer available
    // assert_eq!(z.index().unwrap(), 1);

    let w = model.add_var("w", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 3);
    // assert_eq!(x.index().unwrap(), 0);
    // assert_eq!(y.index(), Err(Error::ModelObjectRemoved)); // No longer available
    // assert_eq!(z.index().unwrap(), 1);
    // assert_eq!(w.index().unwrap(), 2);
  }

  #[test]
  fn remove_and_add_variable_lazy_update() {
    let mut env = Env::new("").unwrap();
    env.set(param::OutputFlag, 0).unwrap();
    env.set(param::UpdateMode, 0).unwrap();

    let mut model = Model::new("bug", &env).unwrap();
    assert!(model.update_mode_lazy().unwrap());

    let x = model.add_var("x", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let y = model.add_var("y", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 0);
    // assert_eq!(x.index().unwrap_err(), Error::ModelObjectPending);
    // assert_eq!(y.index().unwrap_err(), Error::ModelObjectPending);

    model.update().unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    // assert_eq!(x.index().unwrap(), 0);
    // assert_eq!(y.index().unwrap(), 1);

    model.remove(y).unwrap();
    let z = model.add_var("z", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let w = model.add_var("w", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    // assert_eq!(y.index().unwrap_err(), Error::ModelObjectRemoved); // this is updated instantly, but gurobi is only informed later
    // assert_eq!(z.index().unwrap_err(), Error::ModelObjectPending);
    // assert_eq!(w.index().unwrap_err(), Error::ModelObjectPending);

    model.update().unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 3);
    // assert_eq!(x.index().unwrap(), 0);
    // assert_eq!(y.index().unwrap_err(), Error::ModelObjectRemoved);
    // assert_eq!(z.index().unwrap(), 1);
    // assert_eq!(w.index().unwrap(), 2);
  }

  #[test]
  fn proxy_size() {
    assert_eq!(std::mem::size_of::<ModelObj>(), 8)
  }

  #[test]
  fn multiple_models() {
    let mut env = Env::new("").unwrap();
    env.set(param::OutputFlag, 0).unwrap();
    assert_eq!(env.get(param::UpdateMode).unwrap(), 1);

    let mut model1 = Model::new("model1", &env).unwrap();
    let mut model2 = Model::new("model1", &env).unwrap();
    let x1 = model1.add_var("x1", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let y1 = model1.add_var("y1", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let x2 = model2.add_var("x2", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let y2 = model2.add_var("y2", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();

    // assert_eq!(x1.get(&model1, attr::VarName).unwrap(), "x1");
    // assert_eq!(y1.get(&model1, attr::VarName).unwrap(), "y1");
    // assert_eq!(x2.get(&model2, attr::VarName).unwrap(), "x2");
    // assert_eq!(y2.get(&model2, attr::VarName).unwrap(), "y2");
    //
    // assert!(x1.get(&model2, attr::VarName).is_err());
    // assert!(y1.get(&model2, attr::VarName).is_err());
    // assert!(x2.get(&model1, attr::VarName).is_err());
    // assert!(y2.get(&model1, attr::VarName).is_err());
  }
}
