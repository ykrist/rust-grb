// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

#[path = "callback.rs"]
pub mod callback;
#[path = "expr.rs"]
pub mod expr;

use param;
use ffi;
use itertools::{Itertools, Zip};

use std::cell::Cell;
use std::ffi::CString;
use std::iter;
use std::mem::transmute;
use std::ops::{Deref};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::hash::{Hash, Hasher};
use std::slice::Iter;

use attr;
use attribute::{Attr, AttrArray};
use self::callback::{Callback, New};
use self::expr::{LinExpr, QuadExpr};
use env::{Env, EnvAPI};
use error::{Error, Result};
use util;

// use util::Into; // FIXME: wat


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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum IndexState {
  Added(u32),
  PendingRemove(u32),
  PendingAdd,
  Removed,
}

// #[derive(Debug, Clone)]
/// Provides methods to query/modify attributes associated with certain element.
/// Is only public because of Deref impls
#[derive(Debug, Clone)]
pub struct Proxy {
  index_state: Rc<Cell<IndexState>>,
  id: u32,
  model_id: u32,
}

// MEMO:
// 0,1,2,...,INTMAX   : active
// -1                 : wait for adding (before calling update())
// -2                 : removed from the model.
// -3,-4,...          : wait for removing (before calling update())
//  * -3 - index  => indices

impl Proxy {
  fn new(idx: IndexState, id: u32, model_id: u32) -> Proxy {
    Proxy {
      index_state: Rc::new(Cell::new(idx)),
      id,
      model_id,
    }
  }

  pub fn index(&self) -> Result<u32> {
    match self.index_state.get() {
      IndexState::Added(idx) => Ok(idx),
      IndexState::Removed | IndexState::PendingRemove(_) => Err(Error::ModelObjectRemoved),
      IndexState::PendingAdd => Err(Error::ModelObjectPending),
    }
  }

  /// Query the value of attribute.
  pub fn get<A: AttrArray>(&self, model: &Model, attr: A) -> Result<A::Out> {
    model.get_attr_element(attr, &self)
  }

  /// Set the value of attribute.
  pub fn set<A: AttrArray>(&self, model: &mut Model, attr: A, val: A::Out) -> Result<()> {
    model.set_attr_element(attr, &self, val)
  }
}



macro_rules! impl_traits_for_proxy {
  ($t:ident) => {
    impl $t {
      fn new(inner: IndexState, model: &Model) -> $t {
        let id = <IdFactory as NextId<$t, u32>>::next_id(&model.next_id);
        let proxy = Proxy::new(inner, id, model.id);
        $t(proxy)
      }
    }

    impl Deref for $t {
      type Target = Proxy;
      fn deref(&self) -> &Proxy { &self.0 }
    }

    impl PartialEq for $t {
      fn eq(&self, other:&$t) -> bool { self.0.id == other.0.id && self.0.model_id == other.0.model_id }
    }

    impl Eq for $t {}

    impl Hash for $t {
     fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.model_id.hash(state);
      }
    }
  }
}

/// Proxy object of a variables
#[derive(Debug, Clone)]
pub struct Var(Proxy);

impl Var {
  // fn new(inner: IndexState, model: &Model) -> Self {
  //   Var(Proxy::new(inner, model.next_id.next_var_id(), model_id))
  // }
  /// Returns the variable type, lower bound and upper bound in that order.
  ///
  /// Variable type is 'C' for continuous, 'B' for binary, 'I' for integer,
  /// 'S' for semi-continuous, or 'N' for semi-integer.
  pub fn get_type(&self, model: &Model) -> Result<(char, f64, f64)> {
    let lb = self.get(&model, attr::LB)?;
    let ub = self.get(&model, attr::UB)?;
    let vtype: i8 = self.get(&model, attr::VType)?;
    let vtype = vtype as u8 as char;
    Ok((vtype, lb, ub))
  }
}
impl_traits_for_proxy!(Var);

/// Proxy object of a linear constraint
#[derive(Clone, Debug)]
pub struct Constr(Proxy);
impl_traits_for_proxy!(Constr);

/// Proxy object of a quadratic constraint
#[derive(Clone, Debug)]
pub struct QConstr(Proxy);
impl_traits_for_proxy!(QConstr);

/// Proxy object of a Special Order Set (SOS) constraint
#[derive(Clone, Debug)]
pub struct SOS(Proxy);
impl_traits_for_proxy!(SOS);

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



use std::sync::atomic::{Ordering, AtomicU32};

struct IdFactory {
  next_var: AtomicU32,
  next_constr: AtomicU32,
  next_qconstr: AtomicU32,
  next_sos: AtomicU32,
}


trait NextId<T, I> {
  fn next_id(&self) -> I;
}

impl IdFactory {
  pub fn new() -> Self {
    IdFactory {
      next_var: AtomicU32::new(0),
      next_constr: AtomicU32::new(0),
      next_qconstr: AtomicU32::new(0),
      next_sos: AtomicU32::new(0),
    }
  }

}

macro_rules! impl_next_id {
    ($t:ty, $attr:ident) => {
      impl NextId<$t, u32> for IdFactory {
        fn next_id(&self) -> u32 {
          self.$attr.fetch_add(1, Ordering::Relaxed)
        }
      }
    }
}

impl_next_id!(Var, next_var);
impl_next_id!(Constr, next_constr);
impl_next_id!(QConstr, next_qconstr);
impl_next_id!(SOS, next_sos);



/// Gurobi model object associated with certain environment.
pub struct Model {
  model: *mut ffi::GRBmodel,
  id: u32,
  next_id: IdFactory,
  env: Env,
  vars: Vec<Var>,
  constrs: Vec<Constr>,
  qconstrs: Vec<QConstr>,
  sos: Vec<SOS>,
}


macro_rules! model_create_proxy_impl {
    ($p_ty:ty, $attr:ident, $method_name:ident, $batch_method_name:ident) => {
      fn $method_name(&mut self) -> Result<$p_ty> {
        if self.update_mode_lazy()? {
          Ok(<$p_ty>::new(IndexState::PendingAdd, &self))
        } else {
          let obj = <$p_ty>::new(IndexState::Added(self.$attr.len() as u32), &self);
          self.update()?;
          Ok(obj)
        }
      }

      fn $batch_method_name(&mut self, num: u32) -> Result<Vec<$p_ty>> {
        if self.update_mode_lazy()? {
          Ok(vec![<$p_ty>::new(IndexState::PendingAdd, &self); num as usize])
        } else {
          if num == 0 {
            Ok(Vec::new())
          } else {
            let start_idx = self.$attr.len() as u32;
            let objs = (start_idx..start_idx+num).map(|idx| <$p_ty>::new(IndexState::Added(idx), &self)).collect();
            self.update()?;
            Ok(objs)
          }
      }
    }
  }
}

/// Helper function to convert LinExpr objects into Compressed Sparse Row (CSR) format
fn csr_format(expr: Vec<LinExpr>) -> Result<(Vec<i32>, Vec<i32>, Vec<f64>)> {
  let lhs: Vec<(_, _, _)> = expr.into_iter().map(|e| e.into_parts()).collect();

  let mut constr_index_end = Vec::with_capacity(lhs.len());
  let mut cumulative_nz = 0;

  for (vars, _, _) in lhs.iter() {
    cumulative_nz += vars.len();
    constr_index_end.push(cumulative_nz as i32);
  }

  let mut variable_indices = Vec::with_capacity(cumulative_nz);
  let mut coeff = Vec::with_capacity(cumulative_nz);
  for (vars, coeffs, _) in lhs.iter() {
    for v in vars {
      variable_indices.push(v.index()? as i32)
    };
    coeff.extend_from_slice(coeffs);
  }
  Ok((constr_index_end, variable_indices, coeff))
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
    use env::FromRaw;
    let env = unsafe { ffi::GRBgetenv(model) };
    if env.is_null() {
      return Err(Error::FromAPI("Failed to retrieve GRBenv from given model".to_owned(),
                                2002));
    }
    let env = Env::from_raw(env);

    let mut model = Model {
      model,
      id: Model::next_id(),
      next_id: IdFactory::new(),
      env,
      vars: Vec::new(),
      constrs: Vec::new(),
      qconstrs: Vec::new(),
      sos: Vec::new(),
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

  fn get_index(&self, item: & impl Deref<Target=Proxy>) -> Result<i32> {
    if item.model_id != self.id {
      Err(Error::ModelObjectMismatch)
    } else {
      item.index().map(|idx| idx as i32)
    }
  }

  #[inline]
  fn get_indices(&self, items: &[impl Deref<Target=Proxy>]) -> Result<Vec<i32>> {
    items.iter().map(|item| self.get_index(item)).collect()
  }

  #[inline]
  fn get_indices_ref<T: Deref<Target=Proxy>>(&self, items: &[&T]) -> Result<Vec<i32>> {
    items.iter().map(|&item| self.get_index(item)).collect()
  }

  model_create_proxy_impl!(Var, vars, create_var_proxy, create_var_proxies);
  model_create_proxy_impl!(Constr, constrs, create_constr_proxy, create_constr_proxies);
  model_create_proxy_impl!(QConstr, qconstrs, create_qconstr_proxy, create_qconstr_proxies);
  model_create_proxy_impl!(SOS, sos, create_sos_proxy, create_sos_proxies);

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


  fn update_items<P: Deref<Target=Proxy> + Clone>(&self, list: &[P]) -> Result<Vec<P>> {
    let mut keep = Vec::with_capacity(list.len());
    let mut remove_inds = Vec::new();

    for item in list.iter() {
      match item.deref().index_state.get() {
        IndexState::Added(_) | IndexState::PendingAdd => {
          item.index_state.set(IndexState::Added(keep.len() as u32));
          keep.push(item.clone())
        }
        IndexState::PendingRemove(idx) => {
          remove_inds.push(idx as i32)
        }
        IndexState::Removed => unreachable!()
      }
    }

    if !remove_inds.is_empty() {
      self.check_apicall(unsafe { ffi::GRBdelvars(self.model, remove_inds.len() as ffi::c_int, remove_inds.as_ptr()) })?;
    }
    Ok(keep)
  }


  /// Apply all modification of the model to process
  pub fn update(&mut self) -> Result<()> {
    self.vars = self.update_items(&self.vars)?;
    self.constrs = self.update_items(&self.constrs)?;
    self.qconstrs = self.update_items(&self.qconstrs)?;
    self.sos = self.update_items(&self.sos)?;
    // process all of the modification.
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
    self.update()?; // TODO should be unnecessary if self.update_mode_lazy() is false
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
    use env::FromRaw;

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


    let colconstrs = {
      let mut buf = Vec::with_capacity(colconstrs.len());
      for elem in colconstrs.iter() {
        let idx = elem.index()? as i32;
        if idx < 0 {
          return Err(Error::InconsistentDims);
        }
        buf.push(idx);
      }
      buf
    };

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

    let var = self.create_var_proxy()?;
    self.vars.push(var.clone());
    Ok(var)
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
      let len_ind = colconstrs.iter().fold(0usize, |e, &c| e + c.len());
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
          let idx = c.index()? as i32;
          if idx < 0 {
            return Err(Error::InconsistentDims);
          }
          buf_ind.push(idx);
        }

        buf_val.extend(vals);
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

    let vars = self.create_var_proxies(names.len() as u32)?;
    self.vars.extend_from_slice(&vars);
    Ok(vars)
  }


  /// add a linear constraint to the model.
  pub fn add_constr(&mut self, name: &str, expr: LinExpr, sense: ConstrSense, rhs: f64) -> Result<Constr> {
    let constrname = CString::new(name)?;
    let (vars, coeff, offset) = expr.into_parts();
    let vinds = self.get_indices(&vars)?;
    self.check_apicall(unsafe {
      ffi::GRBaddconstr(self.model,
                        coeff.len() as ffi::c_int,
                        vinds.as_ptr(),
                        coeff.as_ptr(),
                        sense.into(),
                        rhs - offset,
                        constrname.as_ptr())
    })?;

    let cons = self.create_constr_proxy()?;
    self.constrs.push(cons.clone());
    Ok(cons)
  }


  /// add linear constraints to the model.
  pub fn add_constrs(&mut self, names: Vec<&str>, lhs: Vec<LinExpr>, sense: Vec<ConstrSense>, mut rhs: Vec<f64>) -> Result<Vec<Constr>> {
    if !(names.len() == lhs.len() && lhs.len() == sense.len() && sense.len() == rhs.len()) {
      return Err(Error::InconsistentDims);
    }
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

    let new_constr = self.create_constr_proxies(constrnames.len() as u32)?;
    self.constrs.extend_from_slice(&new_constr);
    Ok(new_constr)
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
    let (vars, coeff, offset) = expr.into_parts();
    let inds = self.get_indices(&vars)?;
    self.check_apicall(unsafe {
      ffi::GRBaddrangeconstr(self.model,
                             coeff.len() as ffi::c_int,
                             inds.as_ptr(),
                             coeff.as_ptr(),
                             lb - offset,
                             ub - offset,
                             constrname.as_ptr())
    })?;

    let var = self.create_var_proxy()?;
    let cons = self.create_constr_proxy()?;
    self.vars.push(var.clone());
    self.constrs.push(cons.clone());
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

    let ncons = names.len() as u32;
    let vars = self.create_var_proxies(ncons)?;
    let cons = self.create_constr_proxies(ncons)?;
    self.vars.extend_from_slice(&vars);
    self.constrs.extend_from_slice(&cons);
    Ok((vars, cons))
  }

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self, constrname: &str, expr: QuadExpr, sense: ConstrSense, rhs: f64) -> Result<QConstr> {
    let constrname = CString::new(constrname)?;
    let (qrow, qcol, qval, lvar, lval, offset) = expr.into_parts();
    self.check_apicall(unsafe {
      ffi::GRBaddqconstr(self.model,
                         lval.len() as ffi::c_int,
                         self.get_indices(&lvar)?.as_ptr(),
                         lval.as_ptr(),
                         qval.len() as ffi::c_int,
                         self.get_indices(&qrow)?.as_ptr(),
                         self.get_indices(&qcol)?.as_ptr(),
                         qval.as_ptr(),
                         sense.into(),
                         rhs - offset,
                         constrname.as_ptr())
    })?;

    let qconstr = self.create_qconstr_proxy()?;
    self.qconstrs.push(qconstr.clone());
    Ok(qconstr)
  }

  /// add Special Order Set (SOS) constraint to the model.
  pub fn add_sos(&mut self, vars: &[Var], weights: &[f64], sostype: SOSType) -> Result<SOS> {
    if vars.len() != weights.len() {
      return Err(Error::InconsistentDims);
    }

    let vars: Result<Vec<i32>> = vars.iter().map(|v| v.index().map(|idx| idx as i32)).collect();
    let vars = vars?;
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

    let sos = self.create_sos_proxy()?;
    self.sos.push(sos.clone());
    Ok(sos)
  }

  /// Set the objective function of the model.
  pub fn set_objective<Expr: Into<QuadExpr>>(&mut self, expr: Expr, sense: ModelSense) -> Result<()> {
    // if self.updatemode.is_some() {
    //   return Err(Error::FromAPI("The objective function cannot be set before any pending modifies existed".to_owned(),
    //                             50000));
    // }
    let (qrow, qcol, qval, lvar, lval, _) = expr.into().into_parts();
    self.del_qpterms()?;
    self.add_qpterms(&self.get_indices(&qrow)?, &self.get_indices(&qcol)?, &qval)?;
    self.set_list(attr::Obj, &self.get_indices(&lvar)?, lval.as_slice())?;
    self.set(attr::ModelSense, sense.into())
  }


  pub fn get_constr_by_name(&self, name: &str) -> Result<Constr> {
    let constrname = CString::new(name)?;
    let mut value: ffi::c_int = util::Init::init();

    self.check_apicall(unsafe {
      use util::AsRawPtr;
      ffi::GRBgetconstrbyname(self.model, constrname.as_ptr(), value.as_rawptr())
    })?;

    if value == -1 {
      Err(Error::FromAPI("Tried to use a constraint or variable that is not in the model, either because it was removed or because it has not yet been added".to_owned(), 20001))
    } else {
      debug_assert!(self.constrs.len() < value as usize);
      Ok(self.constrs[value as usize].clone())
    }
  }

  pub fn get_var_by_name(&self, name: &str) -> Result<Var> {
    let varname = CString::new(name)?;
    let mut value: ffi::c_int = util::Init::init();

    self.check_apicall(unsafe {
      use util::AsRawPtr;
      ffi::GRBgetvarbyname(self.model, varname.as_ptr(), value.as_rawptr())
    })?;

    if value == -1 {
      Err(Error::FromAPI("Tried to use a constraint or variable that is not in the model, either because it was removed or because it has not yet been added".to_owned(), 20001))
    } else {
      debug_assert!(self.constrs.len() < value as usize);
      Ok(self.vars[value as usize].clone())
    }
  }


  /// Query the value of attributes which associated with variable/constraints.
  pub fn get<A: Attr>(&self, attr: A) -> Result<A::Out> {
    let mut value: A::Buf = util::Init::init();

    self.check_apicall(unsafe {
      use util::AsRawPtr;
      A::get_attr(self.model, attr.into().as_ptr(), value.as_rawptr())
    })?;

    Ok(util::Into::into(value))
  }

  /// Set the value of attributes which associated with variable/constraints.
  pub fn set<A: Attr>(&mut self, attr: A, value: A::Out) -> Result<()> {
    self.check_apicall(unsafe { A::set_attr(self.model, attr.into().as_ptr(), util::From::from(value)) })?;
    self.update()
  }


  fn get_attr_element<A: AttrArray>(&self, attr: A, element: &Proxy) -> Result<A::Out> {
    let index = self.get_index(&element)?;
    let mut value: A::Buf = util::Init::init();

    self.check_apicall(unsafe {
      use util::AsRawPtr;
      A::get_attrelement(self.model, attr.into().as_ptr(), index, value.as_rawptr())
    })?;

    Ok(util::Into::into(value))
  }

  fn set_attr_element<A: AttrArray>(&mut self, attr: A, element: &Proxy, value: A::Out) -> Result<()> {
    let index = self.get_index(&element)?;
    self.check_apicall(unsafe {
      A::set_attrelement(self.model,
                         attr.into().as_ptr(),
                         index,
                         util::From::from(value))
    })?;
    self.update()
  }

  /// Query the value of attributes which associated with variable/constraints.
  pub fn get_values<A: AttrArray, P>(&self, attr: A, item: &[P]) -> Result<Vec<A::Out>>
    where P: Deref<Target=Proxy>
  {
    self.get_list(attr, &self.get_indices(item)?)
  }

  fn get_list<A: AttrArray>(&self, attr: A, ind: &[i32]) -> Result<Vec<A::Out>> {
    let mut values: Vec<_> = iter::repeat(util::Init::init()).take(ind.len()).collect();

    let ind = {
      let mut buf = Vec::with_capacity(ind.len());
      for &i in ind {
        if i < 0 {
          return Err(Error::InconsistentDims);
        }
        buf.push(i);
      }
      buf
    };

    self.check_apicall(unsafe {
      A::get_attrlist(self.model,
                      attr.into().as_ptr(),
                      ind.len() as ffi::c_int,
                      ind.as_ptr(),
                      values.as_mut_ptr())
    })?;

    Ok(values.into_iter().map(util::Into::into).collect())
  }


  /// Set the value of attributes which associated with variable/constraints.
  pub fn set_values<A: AttrArray, P>(&mut self, attr: A, item: &[P], val: &[A::Out]) -> Result<()>
    where P: Deref<Target=Proxy>
  {
    self.set_list(attr, &self.get_indices(item)?, val)?;
    self.update() // TODO: why is this here?
  }

  fn set_list<A: AttrArray>(&mut self, attr: A, ind: &[i32], values: &[A::Out]) -> Result<()> {
    if ind.len() != values.len() {
      return Err(Error::InconsistentDims);
    }

    let ind = {
      let mut buf = Vec::with_capacity(ind.len());
      for &i in ind {
        if i < 0 {
          return Err(Error::InconsistentDims);
        }
        buf.push(i);
      }
      buf
    };
    let values = A::to_rawsets(values)?;

    assert_eq!(ind.len(), values.len());

    self.check_apicall(unsafe {
      A::set_attrlist(self.model,
                      attr.into().as_ptr(),
                      values.len() as ffi::c_int,
                      ind.as_ptr(),
                      values.as_ptr())
    })
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
        let idx = v.index()? as usize;
        if idx >= self.vars.len() {
          return Err(Error::InconsistentDims); // FIXME is this needed?
        }
        pen_lb[idx as usize] = lb;
        pen_ub[idx as usize] = ub;
      }
      (pen_lb.as_ptr(), pen_ub.as_ptr())
    };

    let pen_rhs = if constrs.is_empty() {
      std::ptr::null()
    } else {
      let mut pen_rhs = vec![super::INFINITY; self.constrs.len()];
      for (c, &rhs) in Zip::new((constrs, rhspen)) {
        let idx = c.index()?;
        if idx >= self.constrs.len() as u32 {
          return Err(Error::InconsistentDims);
        }

        pen_rhs[idx as usize] = rhs;
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


    let n_vars = self.get(attr::NumConstrs)? as u32;
    assert!(n_vars >= self.vars.len() as u32);
    let num_new_vars = n_vars - self.vars.len() as u32;
    let new_vars = if num_new_vars > 0 {
      let new_vars = self.create_var_proxies(num_new_vars)?;
      self.vars.extend_from_slice(&new_vars);
      new_vars
    } else {
      Vec::new()
    };

    let n_cons = self.get(attr::NumConstrs)? as u32;
    assert!(n_cons >= self.vars.len() as u32);
    let num_new_cons = n_cons - self.vars.len() as u32;
    let new_cons = if num_new_cons > 0 {
      let new_cons = self.create_constr_proxies(num_new_cons)?;
      self.constrs.extend_from_slice(&new_cons);
      new_cons
    } else {
      Vec::new()
    };

    let n_qcons = self.get(attr::NumConstrs)? as u32;
    assert!(n_qcons >= self.vars.len() as u32);
    let num_new_qcons = n_qcons - self.vars.len() as u32;
    let new_qcons = if num_new_qcons > 0 {
      let new_qcons = self.create_qconstr_proxies(num_new_qcons)?;
      self.qconstrs.extend_from_slice(&new_qcons);
      new_qcons
    } else {
      Vec::new()
    };

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
                        var.index()? as i32,
                        x.len() as ffi::c_int,
                        x.as_ptr(),
                        y.as_ptr())
    })?;
    self.update()
  }

  /// Retrieve the status of the model.
  pub fn status(&self) -> Result<Status> { self.get(attr::Status).map(|val| val.into()) }

  /// Retrieve an iterator of the variables in the model.
  pub fn get_vars(&self) -> Iter<Var> { self.vars.iter() }

  /// Retrieve an iterator of the linear constraints in the model.
  pub fn get_constrs(&self) -> Iter<Constr> { self.constrs.iter() }

  /// Retrieve an iterator of the quadratic constraints in the model.
  pub fn get_qconstrs(&self) -> Iter<QConstr> { self.qconstrs.iter() }

  /// Retrieve an iterator of the special order set (SOS) constraints in the model.
  pub fn get_sos(&self) -> Iter<SOS> { self.sos.iter() }

  // FIXME, bug - the item should update according to self.updatemode
  /// Remove a variable from the model.
  pub fn remove<P: Deref<Target=Proxy>>(&mut self, item: P) -> Result<()> {
    let item: &Proxy = item.deref();
    item.index_state.set(match item.index_state.get() {
      IndexState::PendingRemove(_) | IndexState::Removed | IndexState::PendingAdd => todo!(),
      IndexState::Added(idx) => IndexState::PendingRemove(idx)
    });
    if !self.update_mode_lazy()? {
      self.update()?;
    }
    Ok(())
  }


  /// Retrieve a single constant matrix coefficient of the model.
  pub fn get_coeff(&self, var: &Var, constr: &Constr) -> Result<f64> {
    let mut value = 0.0;
    self.check_apicall(unsafe { ffi::GRBgetcoeff(self.model, var.index()? as i32, constr.index()? as i32, &mut value) })?;
    Ok(value)
  }

  /// Change a single constant matrix coefficient of the model.
  pub fn set_coeff(&mut self, var: &Var, constr: &Constr, value: f64) -> Result<()> {
    self.check_apicall(unsafe { ffi::GRBchgcoeffs(self.model, 1, &(constr.index()? as i32), &(var.index()? as i32), &value) })?;
    self.update()
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
    })?;
    self.update()
  }

  fn populate(&mut self) -> Result<()> {
    assert!(self.vars.is_empty());
    assert!(self.constrs.is_empty());
    assert!(self.qconstrs.is_empty());
    assert!(self.sos.is_empty());
    self.vars = self.create_var_proxies(self.get(attr::NumVars)? as u32)?;
    self.constrs = self.create_constr_proxies(self.get(attr::NumConstrs)? as u32)?;
    self.qconstrs = self.create_qconstr_proxies(self.get(attr::NumQConstrs)? as u32)?;
    self.sos = self.create_sos_proxies(self.get(attr::NumSOS)? as u32)?;
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

  fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
    if error != 0 {
      use env::ErrorFromAPI;
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
  use super::IndexState;

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
    assert_eq!(env.get(param::UpdateMode).unwrap(), 1);

    let mut model = Model::new("hoge", &env).unwrap();
    let x = model.add_var("x", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let y = model.add_var("y", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get(attr::NumVars).unwrap(), 2);
    assert_eq!(x.index().unwrap(), 0);
    assert_eq!(y.index().unwrap(), 1);

    model.update().unwrap(); // should be no effect
    assert_eq!(x.index().unwrap(), 0);
    assert_eq!(y.index().unwrap(), 1);

    let z = model.add_var("z", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get(attr::NumVars).unwrap(), 3);
    assert_eq!(x.index().unwrap(), 0);
    assert_eq!(y.index().unwrap(), 1);
    assert_eq!(z.index().unwrap(), 2);

    model.remove(y.clone()).unwrap();
    assert_eq!(model.get(attr::NumVars).unwrap(), 2);
    assert_eq!(x.index().unwrap(), 0);
    assert_eq!(y.index(), Err(Error::ModelObjectRemoved)); // No longer available
    assert_eq!(z.index().unwrap(), 1);

    let w = model.add_var("w", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get(attr::NumVars).unwrap(), 3);
    assert_eq!(x.index().unwrap(), 0);
    assert_eq!(y.index(), Err(Error::ModelObjectRemoved)); // No longer available
    assert_eq!(z.index().unwrap(), 1);
    assert_eq!(w.index().unwrap(), 2);
  }

  #[test]
  fn remove_and_add_variable_lazy_update() {
    let mut env = Env::new("").unwrap();
    env.set(param::OutputFlag, 0).unwrap();
    env.set(param::UpdateMode, 0).unwrap();

    let mut model = Model::new("bug", &env).unwrap();
    let x = model.add_var("x", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let y = model.add_var("y", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get(attr::NumVars).unwrap(), 0);
    assert_eq!(x.index().unwrap_err(), Error::ModelObjectPending);
    assert_eq!(y.index().unwrap_err(), Error::ModelObjectPending);

    model.update().unwrap();
    assert_eq!(model.get(attr::NumVars).unwrap(), 2);
    assert_eq!(x.index().unwrap(), 0);
    assert_eq!(y.index().unwrap(), 1);

    model.remove(y.clone()).unwrap();
    let z = model.add_var("z", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    let w = model.add_var("w", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    assert_eq!(model.get(attr::NumVars).unwrap(), 2);
    assert_eq!(x.index().unwrap(), 0);
    assert_eq!(y.index().unwrap_err(), Error::ModelObjectRemoved); // this is updated instantly, but gurobi is only informed later
    assert_eq!(z.index().unwrap_err(), Error::ModelObjectPending);
    assert_eq!(w.index().unwrap_err(), Error::ModelObjectPending);

    model.update().unwrap();
    assert_eq!(model.get(attr::NumVars).unwrap(), 3);
    assert_eq!(x.index().unwrap(), 0);
    assert_eq!(y.index().unwrap_err(), Error::ModelObjectRemoved);
    assert_eq!(z.index().unwrap(), 1);
    assert_eq!(w.index().unwrap(), 2);
  }

  #[test]
  fn proxy_size() {
    use super::Proxy;
    assert_eq!(std::mem::size_of::<Proxy>(), 16)
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

    assert_eq!(x1.get(&model1, attr::VarName).unwrap(), "x1");
    assert_eq!(y1.get(&model1, attr::VarName).unwrap(), "y1");
    assert_eq!(x2.get(&model2, attr::VarName).unwrap(), "x2");
    assert_eq!(y2.get(&model2, attr::VarName).unwrap(), "y2");

    assert!(x1.get(&model2, attr::VarName).is_err());
    assert!(y1.get(&model2, attr::VarName).is_err());
    assert!(x2.get(&model1, attr::VarName).is_err());
    assert!(y2.get(&model1, attr::VarName).is_err());
  }
}
