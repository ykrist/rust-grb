// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use gurobi_sys as ffi;
use std::ffi::CString;
use std::mem::transmute;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null, null_mut};
use std::sync::atomic::{Ordering, AtomicU32};

use crate::{Error, Result, Env, QuadExpr, LinExpr, Expr};
use crate::param;
use crate::attr;
use crate::attr::Attr;
use crate::callback::Callback;
use crate::model_object::*;
use crate::{VarType, ConstrSense, ModelSense, SOSType, RelaxType, Status};
use gurobi_sys::GRBmodel;


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



/// Gurobi model object associated with certain environment.
pub struct Model {
  model: *mut ffi::GRBmodel,
  #[allow(dead_code)]
  id: u32,
  env: Env,
  pub(crate) vars: IdxManager<Var>,
  pub(crate) constrs: IdxManager<Constr>,
  pub(crate) qconstrs: IdxManager<QConstr>,
  pub(crate) sos: IdxManager<SOS>,
}


fn convert_to_cstring_ptrs(strings: &[&str]) -> Result<Vec<*const ffi::c_char>> {
  strings.iter().map(|&s| {
    let s = CString::new(s)?;
    Ok(s.as_ptr())
  }).collect()
}

macro_rules! impl_object_list_getter {
    ($name:ident, $t:ty, $attr:ident, $noun:literal) => {
      #[doc="Retrieve the "]
      #[doc=$noun]
      #[doc=" in the model. Returns an error if a model update is needed"]
      pub fn $name<'a>(&'a self) -> Result<&'a [$t]> {
        if self.$attr.model_update_needed() {  Err(Error::ModelUpdateNeeded)  }
        else  { Ok(self.$attr.objects()) }
      }
    };
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
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
  }

  pub fn new(modelname: &str, env: &Env) -> Result<Model> {
    let modelname = CString::new(modelname)?;
    let mut model = null_mut();
    env.check_apicall(unsafe {
      ffi::GRBnewmodel(env.as_ptr(),
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

  pub fn with_default_env(name: &str) -> Result<Model> {
    let env = Env::new("gurobi.log")?;
    Model::new(name, &env)
  }

  /// create an empty model which associated with certain environment.
  fn from_raw(model: *mut ffi::GRBmodel) -> Result<Model> {
    let env = unsafe { ffi::GRBgetenv(model) };
    if env.is_null() {
      return Err(Error::FromAPI("Failed to retrieve GRBenv from given model".to_owned(),
                                2002));
    }
    let env = Env::from_raw(env);
    let id= Model::next_id();


    let mut model = Model {
      model,
      id,
      env,
      vars: IdxManager::new(id),
      constrs: IdxManager::new(id),
      qconstrs: IdxManager::new(id),
      sos: IdxManager::new(id),
    };

    let nvars = model.get_attr(attr::NumVars)?;
    let nconstr = model.get_attr(attr::NumConstrs)?;
    let nqconstr = model.get_attr(attr::NumQConstrs)?;
    let sos = model.get_attr(attr::NumSOS)?;

    model.vars = IdxManager::new_with_existing_obj(id, nvars as usize);
    model.constrs = IdxManager::new_with_existing_obj(id, nconstr as usize);
    model.qconstrs = IdxManager::new_with_existing_obj(id, nqconstr as usize);
    model.sos = IdxManager::new_with_existing_obj(id, sos as usize);

    Ok(model)
  }

  /// Read a model from a file
  pub fn read_from(filename: &str, env: &Env) -> Result<Model> {
    let filename = CString::new(filename)?;
    let mut model = null_mut();
    env.check_apicall(unsafe { ffi::GRBreadmodel(env.as_ptr(), filename.as_ptr(), &mut model) })?;
    Self::from_raw(model)
  }

  /// create a copy of the model
  pub fn copy(&self) -> Result<Model> {
    if self.model_update_needed() { return Err(Error::ModelUpdateNeeded) }

    let copied = unsafe { ffi::GRBcopymodel(self.model) };
    if copied.is_null() {
      return Err(Error::FromAPI("Failed to create a copy of the model".to_owned(), 20002));
    }

    Model::from_raw(copied)
  }

  fn model_update_needed(&self) -> bool {
    self.vars.model_update_needed() ||
      self.constrs.model_update_needed() ||
      self.qconstrs.model_update_needed() ||
      self.sos.model_update_needed()
  }

  #[inline]
  pub(crate) fn get_index<O: ModelObject>(&self, item: &O) -> Result<i32> {
    O::idx_manager(&self).get_index(item)
  }

  #[inline]
  pub(crate) fn get_indices(&self, items: &[impl ModelObject]) -> Result<Vec<i32>> {
    items.iter().map(|item| self.get_index(item)).collect()
  }

  #[inline]
  pub(crate) fn get_index_build<O: ModelObject>(&self, item: &O) -> Result<i32> {
    O::idx_manager(&self).get_index_build(item)
  }

  #[inline]
  pub(crate) fn get_indices_build(&self, items: &[impl ModelObject]) -> Result<Vec<i32>> {
    items.iter().map(|item| self.get_index_build(item)).collect()
  }

  #[inline]
  pub(crate) fn get_coeffs_indices_build(&self, expr: &LinExpr) -> Result<(Vec<i32>, Vec<f64>)> {
    let nterms = expr.n_terms();
    let mut inds = Vec::with_capacity(nterms);
    let mut coeff = Vec::with_capacity(nterms);
    for (x, &c) in expr.iter_terms() {
      inds.push(self.get_index_build(x)?);
      coeff.push(c);
    }
    Ok((inds, coeff))
  }

  #[inline]
  pub(crate) fn get_qcoeffs_indices_build(&self, expr: &QuadExpr) -> Result<(Vec<i32>, Vec<i32>, Vec<f64>)> {
    let nqterms = expr.n_qterms();
    let mut rowinds = Vec::with_capacity(nqterms);
    let mut colinds = Vec::with_capacity(nqterms);
    let mut coeff = Vec::with_capacity(nqterms);
    for ((x,y), &c) in expr.iter_qterms() {
      rowinds.push(self.get_index_build(x)?);
      colinds.push(self.get_index_build(y)?);
      coeff.push(c);
    }
    Ok((rowinds, colinds, coeff))
  }


  /// Helper function to convert LinExpr objects into Compressed Sparse Row (CSR) format
  /// Assumes variables are ready in Build or Present state
  fn convert_coeff_to_csr_build(&self, expr: Vec<LinExpr>) -> Result<(Vec<i32>, Vec<i32>, Vec<f64>)> {
    let expr: Vec<(_, _)> = expr.into_iter().map(|e| e.into_parts()).collect();

    let mut constr_index_end = Vec::with_capacity(expr.len());
    let mut cumulative_nz = 0;

    for (coeff, _) in &expr {
      cumulative_nz += coeff.len();
      constr_index_end.push(cumulative_nz as i32);
    }

    let mut variable_indices = Vec::with_capacity(cumulative_nz);
    let mut coeff = Vec::with_capacity(cumulative_nz);

    for (coeffs, _) in expr {
      for (x, a) in coeffs {
        variable_indices.push(self.get_index_build(&x)?);
        coeff.push(a);
      }
    }
    Ok((constr_index_end, variable_indices, coeff))
  }

  /// Create an fixed model associated with the model.
  ///
  /// In fixed model, each integer variable is fixed to the value that it takes in the
  /// original MIP solution.
  /// Note that the model must be MIP and have a solution loaded.
  pub fn fixed(&mut self) -> Result<Model> {
    let mut fixed : *mut GRBmodel = null_mut();
    self.check_apicall(unsafe { ffi::GRBfixmodel(self.model, &mut fixed) })?;
    debug_assert!(!fixed.is_null());
    Model::from_raw(fixed)
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
    // Notice: Rust does not have appropriate mechanism which treats "null" C-style function
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
    Ok(self.vars.add_new(self.update_mode_lazy()?))
  }

  /// add decision variables to the model.
  pub fn add_vars(&mut self, names: &[&str], vtypes: &[VarType], objs: &[f64], lbs: &[f64], ubs: &[f64], colcoeff: &[Vec<(Constr, f64)>]) -> Result<Vec<Var>> {
    if names.len() != vtypes.len() || vtypes.len() != objs.len() || objs.len() != lbs.len() ||  lbs.len() != colcoeff.len() {
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
      let len_ind = colcoeff.iter().map(|c| c.len()).sum();
      let mut buf_beg = Vec::with_capacity(colcoeff.len());
      let mut buf_ind = Vec::with_capacity(len_ind);
      let mut buf_val: Vec<f64> = Vec::with_capacity(len_ind);

      let mut beg = 0i32;
      for coeff in colcoeff {
        buf_beg.push(beg);
        beg += coeff.len() as i32;

        for (constr, c) in coeff.iter() {
          buf_ind.push(self.get_index(constr)?);
          buf_val.push(*c);
        }
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

    let lazy = self.update_mode_lazy()?;
    Ok(vec![self.vars.add_new(lazy); names.len()])
  }


  /// add a linear constraint to the model.
  pub fn add_constr<Lhs, Rhs>(&mut self, name: &str, lhs: Lhs, sense: ConstrSense, rhs: Rhs) -> Result<Constr> where
    Lhs: Into<Expr>,
    Rhs: Into<Expr>,
  {

    let expr = (lhs.into() - rhs.into()).into_linexpr()?;
    let constrname = CString::new(name)?;
    let (vinds, cval) = self.get_coeffs_indices_build(&expr)?;
    self.check_apicall(unsafe {
      ffi::GRBaddconstr(self.model,
                        cval.len() as ffi::c_int,
                        vinds.as_ptr(),
                        cval.as_ptr(),
                        sense.into(),
                        -expr.get_offset(),
                        constrname.as_ptr())
    })?;

    Ok(self.constrs.add_new(self.update_mode_lazy()?))
  }


  /// add linear constraints to the model.
  pub fn add_constrs(&mut self, names: Vec<&str>, lhs: Vec<Expr>, sense: Vec<ConstrSense>, mut rhs: Vec<f64>) -> Result<Vec<Constr>> {
    if !(names.len() == lhs.len() && lhs.len() == sense.len() && sense.len() == rhs.len()) {
      return Err(Error::InconsistentDims);
    }
    let lhs : Result<Vec<LinExpr>>= lhs.into_iter().map(|e| e.into_linexpr()).collect();
    let lhs = lhs?;
    let sense : Vec<_> = sense.iter().map(|&s| s.into()).collect();
    rhs.iter_mut().zip(lhs.iter()).for_each(|(rhs, lhs)| *rhs -= lhs.get_offset());
    let constrnames = convert_to_cstring_ptrs(&names)?;
    let (cbeg, cind, cval) = self.convert_coeff_to_csr_build(lhs)?;

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

    let lazy = self.update_mode_lazy()?;
    Ok(vec![self.constrs.add_new(lazy); constrnames.len()])
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
    let (inds, coeff) = self.get_coeffs_indices_build(&expr)?;
    self.check_apicall(unsafe {
      ffi::GRBaddrangeconstr(self.model,
                             coeff.len() as ffi::c_int,
                             inds.as_ptr(),
                             coeff.as_ptr(),
                             lb - offset,
                             ub - offset,
                             constrname.as_ptr())
    })?;

    let lazy = self.update_mode_lazy()?;
    let var = self.vars.add_new(lazy);
    let cons = self.constrs.add_new(lazy);
    Ok((var, cons))
  }

  #[allow(unused_variables)]
  /// Add range constraints to the model.
  pub fn add_ranges(&mut self, names: Vec<&str>, expr: Vec<LinExpr>, mut lb: Vec<f64>, mut ub: Vec<f64>)
                    -> Result<(Vec<Var>, Vec<Constr>)> {
    let constrnames = convert_to_cstring_ptrs(&names)?;
    ub.iter_mut().zip(expr.iter()).for_each(|(x, e)| *x -= e.get_offset());
    lb.iter_mut().zip(expr.iter()).for_each(|(x, e)| *x -= e.get_offset());
    let (cbeg, cind, cval) = self.convert_coeff_to_csr_build(expr)?;

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
    let lazy = self.update_mode_lazy()?;
    let vars = vec![self.vars.add_new(lazy); ncons];
    let cons = vec![self.constrs.add_new(lazy); ncons];
    Ok((vars, cons))
  }

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self, constrname: &str, expr: Expr, sense: ConstrSense, rhs: f64) -> Result<QConstr> {
    let constrname = CString::new(constrname)?;
    let expr = expr.into_quadexpr();
    let (qrow, qcol, qval) = self.get_qcoeffs_indices_build(&expr)?;
    let (_, expr) = expr.into_parts();
    let (lvar, lval) = self.get_coeffs_indices_build(&expr)?;
    let offset = expr.get_offset();
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

    Ok(self.qconstrs.add_new(self.update_mode_lazy()?))
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

    Ok(self.sos.add_new(self.update_mode_lazy()?))
  }

  /// Set the objective function of the model.
  pub fn set_objective(&mut self, expr: impl Into<Expr>, sense: ModelSense) -> Result<()> {
    // if self.updatemode.is_some() {
    //   return Err(Error::FromAPI("The objective function cannot be set before any pending modifies existed".to_owned(),
    //                             50000));
    // }

    let expr : Expr = expr.into();
    self.del_qpterms()?;

    let expr = if expr.is_linear() {
      expr.into_linexpr().unwrap()
    } else {
      let qexpr = expr.into_quadexpr();
      let (qrow, qcol, qval) = self.get_qcoeffs_indices_build(&qexpr)?;
      self.add_qpterms(&qrow, &qcol, &qval)?;
      let (_, expr) = qexpr.into_parts();
      expr
    };

    let (coeff_map, _) = expr.into_parts();
    let mut vars = Vec::with_capacity(coeff_map.len());
    let mut coeff = Vec::with_capacity(coeff_map.len());
    for (var, c) in coeff_map {
      vars.push(var);
      coeff.push(c);
    }

    self.set_obj_attr_batch(attr::Obj, &vars, &coeff)?;
    self.set_attr(attr::ModelSense, sense.into())
  }

  /// Get a constraint by name.  Returns either a constraint if one was found, or `None` if none were found.
  /// If multiple constraints match, the method returns an arbitary one.
  ///
  /// # Errors
  /// Returns an error if the model requires an update,  the `name` cannot be converted to a C-string or a Gurobi error occurs.
  ///
  /// # Usage
  /// ```
  /// use gurobi::*;
  /// let mut m = Model::with_default_env("model").unwrap();
  /// let x = add_binvar!(m).unwrap();
  /// let y = add_binvar!(m).unwrap();
  /// let c = m.add_constr("constraint", x + y, Equal, 1.0).unwrap();
  /// assert_eq!(m.get_constr_by_name("constraint").unwrap_err(), Error::ModelUpdateNeeded);
  /// m.update().unwrap();
  /// assert_eq!(m.get_constr_by_name("constraint").unwrap(), Some(c));
  /// assert_eq!(m.get_constr_by_name("foo").unwrap(), None);
  /// ```

  pub fn get_constr_by_name(&self, name: &str) -> Result<Option<Constr>> {
    if self.constrs.model_update_needed() { return Err(Error::ModelUpdateNeeded) }
    let n = CString::new(name)?;
    let mut idx = i32::min_value();
    self.check_apicall(unsafe { ffi::GRBgetconstrbyname(self.model, n.as_ptr(), &mut idx)})?;
    if idx < 0 {
      Ok(None)
    } else {
      Ok(Some(self.constrs.objects()[idx as usize])) // should only panic if there's a bug in IdxManager
    }
  }

  /// Get a variable object by name.  See [`Model.get_constr_by_name`] for details
  ///
  /// [`Model.get_constr_by_name`]: struct.Model.html#methods.get_constr_by_name
  pub fn get_var_by_name(&self, name: &str) -> Result<Option<Var>> {
    if self.vars.model_update_needed() { return Err(Error::ModelUpdateNeeded) }
    let n = CString::new(name)?;
    let mut idx = i32::min_value();
    self.check_apicall(unsafe { ffi::GRBgetvarbyname(self.model, n.as_ptr(), &mut idx)})?;
    if idx < 0 {
      Ok(None)
    } else {
      Ok(Some(self.vars.objects()[idx as usize])) // should only panic if there's a bug in IdxManager
    }
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
    let index = self.get_index_build(elem)?;
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
    let indices = self.get_indices_build(elem)?;
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

    self.update()?;
    let n_old_vars = self.get_attr(attr::NumVars)? as usize;
    let n_old_constr = self.get_attr(attr::NumConstrs)? as usize;
    let n_old_qconstr = self.get_attr(attr::NumQConstrs)? as usize;


    let (pen_lb, pen_ub) = if vars.is_empty() {
      // Gurobi API spec allows for this optimisation
      (std::ptr::null(), std::ptr::null())
    } else {
      let mut pen_lb = vec![super::INFINITY; n_old_vars];
      let mut pen_ub = vec![super::INFINITY; n_old_vars];
      for (v, (&lb, &ub)) in vars.iter().zip(lbpen.iter().zip(ubpen)) {
        let idx = self.get_index(v)? as usize;
        pen_lb[idx] = lb;
        pen_ub[idx] = ub;
      }
      (pen_lb.as_ptr(), pen_ub.as_ptr())
    };

    let pen_rhs = if constrs.is_empty() {
      std::ptr::null()
    } else {
      let mut pen_rhs = vec![super::INFINITY; n_old_constr];
      for (c, &rhs) in constrs.iter().zip(rhspen) {
        let idx = self.get_index(c)? as usize;
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

    let lazy = self.update_mode_lazy()?;

    let n_vars = self.get_attr(attr::NumVars)? as usize;
    assert!(n_vars >= n_old_vars);
    let new_vars = (0..n_vars-n_old_vars).map(|_| self.vars.add_new(lazy)).collect();

    let n_cons = self.get_attr(attr::NumConstrs)? as usize;
    assert!(n_cons >= n_old_constr);
    let new_cons = (0..n_cons-n_old_constr).map(|_| self.constrs.add_new(lazy)).collect();

    let n_qcons = self.get_attr(attr::NumQConstrs)? as usize;
    assert!(n_qcons >= n_old_qconstr);
    let new_qcons = (0..n_cons-n_old_constr).map(|_| self.qconstrs.add_new(lazy)).collect();

    // FIXME: are SOS added here? are QConstr ever added?

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
                        self.get_index_build(var)?,
                        x.len() as ffi::c_int,
                        x.as_ptr(),
                        y.as_ptr())
    })
    // self.update()
  }

  /// Retrieve the status of the model.
  pub fn status(&self) -> Result<Status> { self.get_attr(attr::Status).map(|val| val.into()) }


  impl_object_list_getter!(get_vars, Var, vars, "variables");

  impl_object_list_getter!(get_constrs, Constr, constrs, "constraints");

  impl_object_list_getter!(get_qconstrs, QConstr, qconstrs, "quadratic constraints");

  impl_object_list_getter!(get_sos, SOS, sos, "SOS constraints");

  /// Remove a variable from the model.
  pub fn remove<O: ModelObject>(&mut self, item: O) -> Result<()> {
    let lazy = self.update_mode_lazy()?;
    let im = O::idx_manager_mut(self);
    let idx = im.get_index(&item)?;
    im.remove(item, lazy)?;
    self.check_apicall(unsafe {O::gurobi_remove(self.model, &[idx]) })
  }


  /// Retrieve a single constant matrix coefficient of the model.
  pub fn get_coeff(&self, var: &Var, constr: &Constr) -> Result<f64> {
    let mut value = 0.0;
    self.check_apicall(unsafe {
      ffi::GRBgetcoeff(self.model, self.get_index_build(constr)?,self.get_index_build(var)?, &mut value)
    })?;
    Ok(value)
  }

  /// Change a single constant matrix coefficient of the model.
  pub fn set_coeff(&mut self, var: &Var, constr: &Constr, value: f64) -> Result<()> {
    self.check_apicall(unsafe {
      ffi::GRBchgcoeffs(self.model, 1, &self.get_index_build(constr)?, &self.get_index_build(var)?, &value)
    })
  }

  /// Change a set of constant matrix coefficients of the model.
  pub fn set_coeffs(&mut self, vars: &[Var], constrs: &[Constr], values: &[f64]) -> Result<()> {
    if vars.len() != values.len() || constrs.len() != values.len() {
      return Err(Error::InconsistentDims);
    }

    let vars = self.get_indices_build(&vars)?;
    let constrs = self.get_indices_build(&constrs)?;

    self.check_apicall(unsafe {
      ffi::GRBchgcoeffs(self.model,
                        vars.len() as ffi::c_int,
                        constrs.as_ptr(),
                        vars.as_ptr(),
                        values.as_ptr())
    })
  }

  // add quadratic terms of objective function.
  fn add_qpterms(&mut self, qrow: &[i32], qcol: &[i32], qval: &[f64]) -> Result<()> {
    self.check_apicall(unsafe {
      ffi::GRBaddqpterms(self.model,
                         qrow.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr())
    })
  }

  // remove quadratic terms of objective function.
  fn del_qpterms(&mut self) -> Result<()> {
    self.check_apicall(unsafe { ffi::GRBdelq(self.model) })
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
    // Note: This method runs *before* the `drop()` method on the env inside the model
    // so we free the GRBModel before the GRBEnv, as per the Gurobi docs.
    unsafe { ffi::GRBfreemodel(self.model) };
  }
}


/// Convienence wrapper around [`Model.add_var`] Add a new variable to a `Model` object.  The macro keyword arguments are
/// optional, but their ordering is not.
///
/// # Arguments
/// The first argument is a `Model` object and the second argument is a [`VarType`] variant.  The `bounds` keyword argument
/// is of the format `LB..UB` where `LB` and `UB` are the upper and lower bounds of the variable.  `LB` and `UB` can be
/// left off as well, so `..UB`, `LB..` and `..` are also valid values.
///
/// [`Model.add_var`]: struct.Model.html#method.add_var
/// [`VarType`]: enum.VarType.html
/// ```
/// use gurobi::*;
/// let env = Env::new("gurobi.log").unwrap();
/// let mut model = Model::new("Model", &env).unwrap();
/// add_var!(model, Continuous, name="name", obj=0.0, bounds=-10..10).unwrap();
/// add_var!(model, Integer, bounds=0..).unwrap();
/// add_var!(model, Continuous, name=&format!("X[{}]", 42)).unwrap();
/// ```
#[macro_export]
macro_rules! add_var {
    ($model:ident, $t:ident, name=$name:expr, obj=$obj:expr, bounds=$($lb:literal)?..$($ub:literal)?) => {
      $model.add_var($name, $t, $obj as f64, add_var!(@LB, $($lb)*), add_var!(@UB, $($ub)*), &[], &[])
    };

    ($model:tt, $t:tt, name=$name:expr, obj=$obj:expr) => {
      add_var!($model, $t, name=$name, obj=$obj, bounds=0.0..)
    };

    ($model:tt, $t:tt, obj=$obj:expr,  bounds=$($lb:literal)?..$($ub:literal)?) => {
      add_var!($model, $t, name="", obj=$obj, bounds=$($lb)*..$($ub)*)
    };

    ($model:tt, $t:tt, name=$name:expr,  bounds=$($lb:literal)?..$($ub:literal)?) => {
      add_var!($model, $t, name=$name, obj=0.0, bounds=$($lb)*..$($ub)*)
    };

    ($model:tt, $t:tt, obj=$obj:expr) => {
      add_var!($model, $t, name="", obj=$obj)
    };

    ($model:tt, $t:tt, name=$name:expr) => {
      add_var!($model, $t, name=$name, obj=0.0)
    };

    ($model:tt, $t:tt, bounds=$($lb:literal)?..$($ub:literal)?) => {
      add_var!($model, $t, name="", bounds=$($lb)*..$($ub)*)
    };

    ($model:tt, $t:tt) => {
      add_var!($model, $t, name="")
    };

    (@UB, $x:literal) => { $x as f64 };
    (@UB, ) => { $crate::INFINITY };
    (@LB, $x:literal ) => { $x as f64 };
    (@LB, ) => { -$crate::INFINITY };
}

/// Add a binary variable to a Model object.  See [`add_var!`] for details.  The `bounds` keyword
/// is not available here.
///
/// # Errors
/// This macro will return a `Result<Var>`, forwarding any errors from the Gurobi API.
///
/// # Example
/// Usage with all keyword arguments supplied:
///
/// [`add_var!`]: macro.add_var.html
/// ```
/// use gurobi::*;
/// let env = Env::new("gurobi.log").unwrap();
/// let mut model = Model::new("Model", &env).unwrap();
/// add_binvar!(model, name="name", obj=0.0).unwrap();
/// ```
#[macro_export]
macro_rules! add_binvar {
    ($model:ident $(,$field:tt=$value:expr)*) => {
      add_var!($model, Binary $(,$field=$value)* , bounds=0..1)
    };
}


#[cfg(test)]
mod tests {
  use super::super::*;
  use gurobi_sys::IntParam::OutputFlag;

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

    let x1 = add_var!(m1, Binary,  name="x1").unwrap();
    let x2 = add_var!(m2, Binary,  name="x2").unwrap();
    assert_ne!(m1.id, m2.id);

    assert_eq!(x1.model_id, m1.id);
    assert_eq!(x1.id, 0);
    assert_eq!(x2.model_id, m2.id);
    assert_eq!(x2.id, 0);
  }


  #[test]
  fn eager_update() {
    let env = Env::empty().unwrap().start().unwrap();
    assert_eq!(env.get(param::UpdateMode).unwrap(), 1);

    let mut model = Model::new("test", &env).unwrap();
    assert!(!model.update_mode_lazy().unwrap());
    let x = add_binvar!(model, name="x").unwrap();
    let y = add_binvar!(model, name="y").unwrap();
    let c1 = model.add_constr("c1", x + y, Less, 1.0).unwrap(); // should work fine

    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 0);
    assert!(model.get_index(&x).is_err());
    assert_eq!(model.get_index_build(&x).unwrap(), 0);
    assert!(model.get_index(&y).is_err());
    assert_eq!(model.get_index_build(&y).unwrap(), 1);
    assert_eq!(model.get_index_build(&c1).unwrap(), 0);

    model.update().unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y).unwrap(), 1);

    let z = add_binvar!(model, name="z").unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y).unwrap(), 1);
    assert!(model.get_index(&z).is_err());
    assert_eq!(model.get_index_build(&z).unwrap(), 2);

    model.remove(y).unwrap();
    let c2 = model.add_constr("c2", z + y, Less, 1.0).unwrap(); // I know it's weird, because y is removed, but that's what Gurobi does
    assert_eq!(model.get_index_build(&c2).unwrap(), 1);
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y), Err(Error::ModelObjectRemoved)); // No longer available
    assert!(model.get_index(&z).is_err());
    assert_eq!(model.get_index_build(&z).unwrap(), 2);

    let w = add_binvar!(model, name="w").unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y), Err(Error::ModelObjectRemoved)); // No longer available
    assert!(model.get_index(&z).is_err());
    assert_eq!(model.get_index_build(&z).unwrap(), 2);
    assert!(model.get_index(&w).is_err());
    assert_eq!(model.get_index_build(&w).unwrap(), 3);

    model.update().unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 3);
    assert_eq!(model.get_attr(attr::NumNZs).unwrap(), 2); // start with 2 constraints with 2 vars = 2 x 2 = 4, minus where y appears twice = 4 - 2 = 2
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert!(model.get_index(&y).is_err());
    assert_eq!(model.get_index(&z).unwrap(), 1);
    assert_eq!(model.get_index(&w).unwrap(), 2);

    assert_eq!(model.get_obj_attr(attr::VarName, &x).unwrap(), "x".to_string());
    assert_eq!(model.get_obj_attr(attr::VarName, &z).unwrap(), "z".to_string());
    assert_eq!(model.get_obj_attr(attr::VarName, &w).unwrap(), "w".to_string());
  }

  #[test]
  fn lazy_update() {
    let mut env = Env::new("").unwrap();
    env.set(param::OutputFlag, 0).unwrap();
    env.set(param::UpdateMode, 0).unwrap();
    let mut model = Model::new("bug", &env).unwrap();
    assert!(model.update_mode_lazy().unwrap());

    let x = add_binvar!(model, name="x").unwrap();
    let y = add_binvar!(model, name="y").unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 0);
    assert_eq!(model.get_index_build(&x).unwrap_err(), Error::ModelObjectPending);
    assert_eq!(model.get_index_build(&y).unwrap_err(), Error::ModelObjectPending);
    model.add_constr("c1", x + y, Less, 1.0).unwrap_err();

    model.update().unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y).unwrap(), 1);
    let c1 = model.add_constr("c1", x + y, Less, 1.0).unwrap();

    model.remove(y).unwrap();
    let z = add_binvar!(model, name="z").unwrap();
    let w = add_binvar!(model, name="w").unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y).unwrap_err(), Error::ModelObjectRemoved); // this is updated instantly
    assert_eq!(model.get_index_build(&z).unwrap_err(), Error::ModelObjectPending);
    assert_eq!(model.get_index_build(&w).unwrap_err(), Error::ModelObjectPending);

    model.update().unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 3);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y).unwrap_err(), Error::ModelObjectRemoved);
    assert_eq!(model.get_index(&z).unwrap(), 1);
    assert_eq!(model.get_index(&w).unwrap(), 2);
    assert_eq!(model.get_index(&c1).unwrap(), 0);


    assert_eq!(model.get_obj_attr(attr::VarName, &x).unwrap(), "x".to_string());
    assert_eq!(model.get_obj_attr(attr::VarName, &z).unwrap(), "z".to_string());
    assert_eq!(model.get_obj_attr(attr::VarName, &w).unwrap(), "w".to_string());
  }

  #[test]
  fn model_obj_size() {
    assert_eq!(std::mem::size_of::<Var>(), 8);
    assert_eq!(std::mem::size_of::<QConstr>(), 8);
    assert_eq!(std::mem::size_of::<Constr>(), 8);
    assert_eq!(std::mem::size_of::<SOS>(), 8);
  }

  #[test]
  fn multiple_models() {
    let mut env = Env::new("").unwrap();
    env.set(param::OutputFlag, 0).unwrap();
    assert_eq!(env.get(param::UpdateMode).unwrap(), 1);

    let mut model1 = Model::new("model1", &env).unwrap();
    let mut model2 = Model::new("model1", &env).unwrap();

    let x1 = add_binvar!(model1, name="x1").unwrap();
    let y1 = add_binvar!(model1, name="y1").unwrap();
    let x2 = add_binvar!(model2, name="x2").unwrap();
    let y2 = add_binvar!(model2, name="y2").unwrap();

    model1.add_constr("", x1 - y1, Less, 0.0).unwrap();
    model2.add_constr("", x2 - y2, Less, 0.0).unwrap();
    assert_eq!(model1.add_constr("", x2 - y1, Less, 0.0).unwrap_err(), Error::ModelObjectMismatch);
    assert_eq!(model1.add_constr("", x1 - y2, Less, 0.0).unwrap_err(), Error::ModelObjectMismatch);

    model1.update().unwrap();
    model2.update().unwrap();


    assert_eq!(model1.get_obj_attr(attr::VarName, &x1).unwrap(), "x1".to_string());
    assert_eq!(model1.get_obj_attr(attr::VarName, &y1).unwrap(), "y1".to_string());
    assert_eq!(model2.get_obj_attr(attr::VarName, &x2).unwrap(), "x2".to_string());
    assert_eq!(model2.get_obj_attr(attr::VarName, &y2).unwrap(), "y2".to_string());

    assert_eq!(model1.get_obj_attr(attr::VarName, &y2).unwrap_err(), Error::ModelObjectMismatch);
    assert_eq!(model2.get_obj_attr(attr::VarName, &x1).unwrap_err(), Error::ModelObjectMismatch);
  }


  #[test]
  fn new_model_copies_env() -> Result<()> {
    let mut env = Env::new("")?;
    env.set(param::OutputFlag, 0)?;
    let mut model = Model::new("test", &env)?;
    let model_env = model.get_env_mut();
    // assert_eq!(model.get)

    model_env.set(param::OutputFlag, 1)?;
    assert_eq!(model_env.get(param::OutputFlag), Ok(1));
    assert_eq!(env.get(param::OutputFlag), Ok(0));

    assert_ne!(model_env.as_ptr(), env.as_ptr());
    Ok(())
  }

  #[test]
  fn new_model_copies_env_drop() -> Result<()> {
    let mut env = Env::new("")?;
    env.set(param::OutputFlag, 0)?;
    let mut model = Model::new("test", &env)?;
    drop(env); // frees underlying GRBEnv
    let model_env = model.get_env_mut();
    model_env.set(param::OutputFlag, 1)?;
    assert_eq!(model_env.get(param::OutputFlag), Ok(1));
    Ok(())
  }


  #[test]
  fn model_copy_copies_env() -> Result<()> {
    let mut env = Env::new("")?;
    env.set(param::OutputFlag, 0)?;
    let mut m1 = Model::new("m1", &env)?;
    let m2 = m1.copy()?;

    let m1_env = m1.get_env_mut();
    let m2_env = m2.get_env();

    m1_env.set(param::OutputFlag, 1)?;

    assert_eq!(m1_env.get(param::OutputFlag), Ok(1));
    assert_eq!(m2_env.get(param::OutputFlag), Ok(0));
    Ok(())
  }

  #[test]
  fn fixed_mip_model_copies_env() -> Result<()> {
    let mut m = {
      let mut env = Env::new("")?;
      env.set(OutputFlag, 0)?;
      Model::new("original", &env)?
    };

    let x = add_var!(m, Continuous, name="x")?;
    let y = add_binvar!(m, name="y")?;

    m.add_constr("c1", x + y ,Less, 1)?;
    m.add_constr("c2", x - y , Less, 2)?;

    m.optimize()?;
    let fixed = m.fixed()?;
    assert_eq!(fixed.get_attr(attr::IsMIP)?, 0);
    assert_eq!(fixed.get_env().get(param::OutputFlag)?, 0);

    m.get_env_mut().set(param::OutputFlag, 1)?;
    assert_eq!(fixed.get_env().get(param::OutputFlag)?, 0);

    assert_ne!(m.get_env().as_ptr(), fixed.get_env().as_ptr());

    Ok(())
  }

  #[test]
  fn read_model_copies_env() -> Result<()> {
    use std::fs::remove_file;
    let env = Env::new("")?;
    let m1 = Model::new("test", &env)?;
    let filename = "test_read_model_copies_env.lp";
    m1.write(filename);
    let m2 = Model::read_from(filename, &env)?;
    assert_ne!(m2.get_env().as_ptr(), m1.get_env().as_ptr());
    Ok(())
  }

  #[test]
  fn copy_env_model_to_model() -> Result<()> {
    let env = Env::new("")?;
    let m1 = Model::new("", &env)?;
    let m2 = Model::new("", m1.get_env())?;

    assert_ne!(m1.get_env().as_ptr(), m2.get_env().as_ptr());
    Ok(())
  }

  #[test]
  fn early_env_drop() -> Result<()> {
    let env = Env::new("")?;
    let p = env.as_ptr();
    let mut m = Model::new("", &env)?;
    drop(env); // this should not free the environment, as m is still in scope

    drop(m);
    let env = Env::from_raw(p);
    env.get(param::UpdateMode)?; // FIXME causes an error, because the GRBEnv has been freed
    Ok(())
  }
}
