use std::ffi::CString;
use std::mem::transmute;
use std::ptr::{null, null_mut};
use std::sync::atomic::{Ordering, AtomicU32};
use std::borrow::Borrow;

use gurobi_sys as ffi;
use gurobi_sys::GRBmodel;

use crate::prelude::*;
use crate::param::Param;
use crate::attr::Attr;
use crate::callback::{UserCallbackData, callback_wrapper};
use crate::{Result, Error};
use crate::model_object::IdxManager;
use crate::expr::{LinExpr, QuadExpr};
use crate::constr::{IneqExpr, RangeExpr};
use crate::env::AsPtr;

/// Gurobi Model object.
pub struct Model {
  ptr: *mut ffi::GRBmodel,
  #[allow(dead_code)]
  id: u32,
  env: Env,
  pub(crate) vars: IdxManager<Var>,
  pub(crate) constrs: IdxManager<Constr>,
  pub(crate) qconstrs: IdxManager<QConstr>,
  pub(crate) sos: IdxManager<SOS>,
}

macro_rules! impl_object_list_getter {
    ($name:ident, $t:ty, $attr:ident, $noun:literal) => {
      #[doc="Retrieve the "]
      #[doc=$noun]
      #[doc=" in the model. \n\n # Errors\nReturns an error if a model update is needed"]
      pub fn $name<'a>(&'a self) -> Result<&'a [$t]> {
        if self.$attr.model_update_needed() {  Err(Error::ModelUpdateNeeded)  }
        else  { Ok(self.$attr.objects()) }
      }
    };
}




impl Model {
  fn next_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
  }

  /// Create a new model with the given environment.  The original environment is
  /// copied by Gurobi.  To modify the environment of the model, use [`Model::get_env_mut`].
  ///
  /// # Examples
  /// ```
  /// # use grb::prelude::*;
  /// let mut env = Env::new("")?;
  /// env.set(param::OutputFlag, 0)?;
  ///
  /// let mut model = Model::with_env("Model", &env)?;
  /// assert_eq!(model.get_param(param::OutputFlag)?,  0);
  ///
  /// // Equivalent to model.set_param(param::OutputFlag, 1)?
  /// model.get_env_mut().set(param::OutputFlag, 1)?;
  ///
  /// assert_eq!(env.get(param::OutputFlag).unwrap(), 0); // original env is unchanged
  /// # Ok::<(), grb::Error>(())
  /// ```
  pub fn with_env(modelname: &str, env: impl Borrow<Env>) -> Result<Model> {
    let env = env.borrow();
    let modelname = CString::new(modelname)?;
    let mut model = null_mut();
    env.check_apicall(unsafe {
      ffi::GRBnewmodel(env.as_mut_ptr(),
                       &mut model,
                       modelname.as_ptr(),
                       0,
                       null(),
                       null(),
                       null(),
                       null(),
                       null())
    })?;
    Self::from_raw(env, model)
  }

  /// Create a new model with the default environment, which is lazily initialised.
  pub fn new(modelname: &str) -> Result<Model> {
    Env::GLOBAL_DEFAULT.with(|env|
      Model::with_env(modelname, env))
  }

  /// Create the `Model` object from a raw pointer returned by a Gurobi routine.
  ///
  /// # Safety
  /// Here we assume that the `GRBEnv` is tied to a specific `GRBModel`
  /// In other words, the pointer returned by GRBgetenv(model) is unique to
  /// that model.  It is explicitly stated in the docs for
  /// [`GRBnewmodel`](https://www.gurobi.com/documentation/9.1/refman/c_newmodel.html)
  /// that the environment the user supplies is copied,  but must be assumed for other
  /// Gurobi routines that create new `GRBmodel`s like
  /// [`GRBfeasrelax`](https://www.gurobi.com/documentation/9.1/refman/c_feasrelax.html),
  /// [`GRBfixmodel`](https://www.gurobi.com/documentation/9.1/refman/c_fixmodel.html)
  /// and [`GRBreadmodel`](https://www.gurobi.com/documentation/9.1/refman/c_readmodel.html)
  /// This assumption is necessary to prevent a double free when a `Model` object is dropped,
  /// which frees the `GRBModel` and triggers the drop of a `Env`, which in turn
  /// frees the `GRBEnv`.  The `*copies_env` tests in this module validate this assumption.
  fn from_raw(env: &Env, model: *mut ffi::GRBmodel) -> Result<Model> {
    let env_ptr = unsafe { ffi::GRBgetenv(model) };
    if env_ptr.is_null() {
      return Err(Error::FromAPI("Failed to retrieve GRBenv from given model".to_owned(),
                                2002));
    }
    let env = unsafe { Env::new_gurobi_allocated(env, env_ptr) };
    let id = Model::next_id();


    let mut model = Model {
      ptr: model,
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

  /// Create a copy of the model.  This method is fallible due to the lazy update approach and the underlying
  /// Gurobi C API, so a [`Clone`] implementation is not provided.
  ///
  /// # Errors
  ///  * [`Error::FromAPI`] if a Gurobi error occurs
  ///  * [`Error::ModelUpdateNeeded`] if model objects have been added to the model since the last update.
  pub fn try_clone(&self) -> Result<Model> {
    if self.model_update_needed() { return Err(Error::ModelUpdateNeeded); }

    let copied = unsafe { ffi::GRBcopymodel(self.ptr) };
    if copied.is_null() {
      return Err(Error::FromAPI("Failed to create a copy of the model".to_owned(), 20002));
    }

    Model::from_raw(&self.env, copied)
  }

  /// Read a model from a file.  See the [manual](https://www.gurobi.com/documentation/9.1/refman/c_readmodel.html) for accepted file formats.
  pub fn read_from(filename: &str, env: &Env) -> Result<Model> {
    let filename = CString::new(filename)?;
    let mut model = null_mut();
    env.check_apicall(unsafe { ffi::GRBreadmodel(env.as_mut_ptr(), filename.as_ptr(), &mut model) })?;
    Self::from_raw(env, model)
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
    for ((x, y), &c) in expr.iter_qterms() {
      rowinds.push(self.get_index_build(x)?);
      colinds.push(self.get_index_build(y)?);
      coeff.push(c);
    }
    Ok((rowinds, colinds, coeff))
  }


  /// Create the fixed model associated with the current MIP model.
  ///
  /// The model must be MIP and have a solution loaded. In the fixed model,
  /// each integer variable is fixed to the value that it takes in the current MIP solution.
  pub fn fixed(&mut self) -> Result<Model> {
    let mut fixed: *mut GRBmodel = null_mut();
    self.check_apicall(unsafe { ffi::GRBfixmodel(self.ptr, &mut fixed) })?;
    debug_assert!(!fixed.is_null());
    Model::from_raw(&self.env, fixed)
  }

  /// Get shared reference of the environment associated with the model.
  pub fn get_env(&self) -> &Env { &self.env }

  /// Get mutable reference of the environment associated with the model.
  pub fn get_env_mut(&mut self) -> &mut Env { &mut self.env }

  /// Apply all queued modification of the model and update internal lookups.
  ///
  /// Some operations like [`Model::try_clone`] require this method to be called.
  ///
  /// # Examples
  /// ```
  /// # use grb::prelude::*;
  /// let mut m = Model::new("model")?;
  /// let x = add_ctsvar!(m);
  ///
  /// assert_eq!(m.try_clone().err().unwrap(), grb::Error::ModelUpdateNeeded);
  ///
  /// m.update();
  /// assert!(m.try_clone().is_ok());
  ///
  /// # Ok::<(), grb::Error>(())
  /// ```
  pub fn update(&mut self) -> Result<()> {
    self.vars.update();
    self.constrs.update();
    self.qconstrs.update();
    self.sos.update();
    self.check_apicall(unsafe { ffi::GRBupdatemodel(self.ptr) })?;
    Ok(())
  }

  /// Query update mode. See [https://www.gurobi.com/documentation/9.1/refman/updatemode.html]
  fn update_mode_lazy(&self) -> Result<bool> {
    //  0 => pending until update() or optimize() called.
    //  1 => all changes are immediate
    Ok(self.env.get(param::UpdateMode)? == 0)
  }

  /// Optimize the model synchronously.  This method will always trigger a [`Model::update`].
  pub fn optimize(&mut self) -> Result<()> {
    self.update()?;
    self.check_apicall(unsafe { ffi::GRBoptimize(self.ptr) })
  }


  /// Optimize the model with a callback.  The callback is any type that implements the
  /// [`Callback`] trait.  Closures, and anything else that implements `FnMut(CbCtx) -> Result<()>`
  /// implement the `Callback` trait automatically.   This method will always trigger a [`Model::update`].
  /// See [`crate::callback`] for details on how to use callbacks.
  pub fn optimize_with_callback<F>(&mut self,  callback: &mut F) -> Result<()>
    where
      F: Callback
  {
    self.update()?;
    let nvars = self.get_attr(attr::NumVars)? as usize;
    let mut usrdata = UserCallbackData {
      model: self,
      cb_obj:  callback,
      nvars,
    };

    unsafe {
      self.check_apicall( ffi::GRBsetcallbackfunc(self.ptr, Some(callback_wrapper), transmute(&mut usrdata)))?;
      self.check_apicall(ffi::GRBoptimize(self.ptr))?;
      self.check_apicall(ffi::GRBsetcallbackfunc(self.ptr, None, null_mut()))?
    }

    Ok(())
  }

  // /// Wait for a optimization called asynchronously.
  // pub fn sync(&self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBsync(self.model) }) }

  /// Compute an Irreducible Inconsistent Subsystem (IIS) of the model.
  pub fn compute_iis(&mut self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBcomputeIIS(self.ptr) }) }

  /// Send a request to the model to terminate the current optimization process.
  pub fn terminate(&self) { unsafe { ffi::GRBterminate(self.ptr) } }

  /// Reset the model to an unsolved state.
  ///
  /// All solution information previously computed are discarded.
  pub fn reset(&self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBresetmodel(self.ptr) }) }

  /// Perform an automated search for parameter settings that improve performance on the model.
  /// See also references [on official
  /// manual](https://www.gurobi.com/documentation/6.5/refman/parameter_tuning_tool.html#sec:Tuning).
  pub fn tune(&self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBtunemodel(self.ptr) }) }

  /// Prepare to retrieve the results of `tune()`.
  /// See also references [on official
  /// manual](https://www.gurobi.com/documentation/6.5/refman/parameter_tuning_tool.html#sec:Tuning).
  pub fn get_tune_result(&self, n: i32) -> Result<()> {
    self.check_apicall(unsafe { ffi::GRBgettuneresult(self.ptr, n) })
  }

  /// Insert a message into log file.
  ///
  /// When **message** cannot convert to raw C string, a panic is occurred.
  pub fn message(&self, message: &str) { self.env.message(message); }

  /// Import optimization data of the model from a file.
  pub fn read(&mut self, filename: &str) -> Result<()> {
    let filename = CString::new(filename)?;
    self.check_apicall(unsafe { ffi::GRBread(self.ptr, filename.as_ptr()) })
  }

  /// Export optimization data of the model to a file.
  pub fn write(&self, filename: &str) -> Result<()> {
    let filename = CString::new(filename)?;
    self.check_apicall(unsafe { ffi::GRBwrite(self.ptr, filename.as_ptr()) })
  }


  /// Add a decision variable to the model.  This method allows the user to give the entire column (constraint coefficients).
  ///
  /// The [`add_var!`](crate::add_var) macro and its friends are usually preferred.
  pub fn add_var(&mut self, name: &str, vtype: VarType, obj: f64, lb: f64, ub: f64, colconstrs: &[Constr],
                 colvals: &[f64])
                 -> Result<Var> {
    if colconstrs.len() != colvals.len() {
      return Err(Error::InconsistentDims);
    }
    let colconstrs = self.get_indices(colconstrs)?;
    let name = CString::new(name)?;
    self.check_apicall(unsafe {
      ffi::GRBaddvar(self.ptr,
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

  /// Add multiple decision variables to the model in a single Gurobi API call.
  pub fn add_vars(&mut self, names: &[&str], vtypes: &[VarType], objs: &[f64], lbs: &[f64], ubs: &[f64], colcoeff: &[Vec<(Constr, f64)>]) -> Result<Vec<Var>> {
    if names.len() != vtypes.len() || vtypes.len() != objs.len() || objs.len() != lbs.len() || lbs.len() != colcoeff.len() {
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
      ffi::GRBaddvars(self.ptr,
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


  /// Add a Linear constraint to the model.
  ///
  /// The `con` argument is usually created with the [`c!`](crate::c) macro.
  ///
  /// # Examples
  /// ```
  /// # use grb::prelude::*;
  /// let mut m = Model::new("model")?;
  /// let x = add_ctsvar!(m)?;
  /// let y = add_ctsvar!(m)?;
  /// m.add_constr("c1", c!(x <= 1 - y))?;
  /// # Ok::<(), grb::Error>(())
  /// ```
  pub fn add_constr(&mut self, name: &str, con: IneqExpr) -> Result<Constr> where
  {
    let (lhs, sense, rhs) = con.into_normalised_linear()?;
    let constrname = CString::new(name)?;
    let (vinds, cval) = self.get_coeffs_indices_build(&lhs)?;
    self.check_apicall(unsafe {
      ffi::GRBaddconstr(self.ptr,
                        cval.len() as ffi::c_int,
                        vinds.as_ptr(),
                        cval.as_ptr(),
                        sense.into(),
                        rhs,
                        constrname.as_ptr())
    })?;

    Ok(self.constrs.add_new(self.update_mode_lazy()?))
  }


  /// Add multiple linear constraints to the model in a single Gurobi API call.
  ///
  /// Accepts anything that can be turned into an iterator of `(name, constraint)` pairs.
  ///
  /// # Examples
  /// ```
  /// # use grb::prelude::*;
  /// let mut m = Model::new("model")?;
  /// let x = add_ctsvar!(m)?;
  /// let y = add_ctsvar!(m)?;
  ///
  /// let constraints = vec![
  ///   ("c1", c!(x <= 1 - y )),
  ///   ("c2", c!(x == 0.5*y )),
  /// ];
  ///
  /// // store names in Vec to ensure they live long enough
  /// let more_constraints_names : Vec<_> =  (0..10).map(|i| format!("r{}", i)).collect();
  /// // A Map<_> iterator of (&String, IneqConstr)
  /// let more_constraints = (0..10).map(|i| (&more_constraints_names[i], c!(x >= i*y )));
  /// m.add_constrs(more_constraints)?;
  /// # Ok::<(), grb::Error>(())
  /// ```
  pub fn add_constrs<'a, I, N>(&mut self, constr_with_names: I) -> Result<Vec<Constr>> where
    N: AsRef<str> + 'a,
    I: Iterator<Item=(&'a N, IneqExpr)>
  {
    let constr_with_name = constr_with_names.into_iter();
    let (nconstr, _) = constr_with_name.size_hint();
    let mut names = Vec::with_capacity(nconstr); // needed to ensure CString lives long enough
    let mut cnames = Vec::with_capacity(nconstr);
    let mut rhs = Vec::with_capacity(nconstr);
    let mut cbeg = Vec::with_capacity(nconstr);
    let mut cind = Vec::with_capacity(nconstr);
    let mut cval = Vec::with_capacity(nconstr);
    let mut senses = Vec::with_capacity(nconstr);

    let mut c_start = 0;
    for (n, c) in constr_with_name {
      let n = CString::new(n.as_ref())?;
      cnames.push(n.as_ptr());
      names.push(n);
      let (lhs, sense, r) = c.into_normalised_linear()?;
      rhs.push(r);
      senses.push(sense.into());

      let (var_coeff, _) = lhs.into_parts();
      let nterms = var_coeff.len();
      cbeg.push(c_start);
      c_start += nterms as i32;

      cind.reserve(nterms);
      cval.reserve(nterms);
      for (var, coeff) in var_coeff {
        cind.push(self.get_index_build(&var)?);
        cval.push(coeff);
      }
    }

    self.check_apicall(unsafe {
      ffi::GRBaddconstrs(self.ptr,
                         cnames.len() as ffi::c_int,
                         cbeg.len() as ffi::c_int,
                         cbeg.as_ptr(),
                         cind.as_ptr(),
                         cval.as_ptr(),
                         senses.as_ptr(),
                         rhs.as_ptr(),
                         cnames.as_ptr())
    })?;

    let lazy = self.update_mode_lazy()?;
    Ok(vec![self.constrs.add_new(lazy); cnames.len()])
  }

  /// Add a range constraint to the model.
  ///
  /// This operation adds a decision variable with lower/upper bound, and a linear
  /// equality constraint which states that the value of variable must equal to `expr`.
  ///
  /// As with [`Model::add_constr`], the [`c!`](crate::c) macro is usually used to construct
  /// the second argument.
  ///
  /// # Errors
  ///  - [`Error::AlgebraicError`] if the expression in the range constraint is not linear.
  ///  - [`Error::ModelObjectPending`] if some variables haven't yet been added to the model.
  ///  - [`Error::ModelObjectRemoved`] if some variables have been removed from the model.
  ///  - [`Error::ModelObjectMismatch`] if some variables are from a different model.
  ///  - [`Error::FromAPI`]
  ///
  /// # Examples
  /// ```
  /// # use grb::prelude::*;
  /// let mut m = Model::new("model")?;
  /// let x = add_ctsvar!(m)?;
  /// let y = add_ctsvar!(m)?;
  /// m.add_range("", c!(x - y in 0..1))?;
  /// assert!(matches!(m.add_range("", c!(x*y in 0..1)).unwrap_err(), grb::Error::AlgebraicError(_)));
  /// # Ok::<(), grb::Error>(())
  /// ```
  pub fn add_range(&mut self, name: &str, expr: RangeExpr) -> Result<(Var, Constr)> {
    let constrname = CString::new(name)?;
    let (expr, lb, ub) = expr.into_normalised()?;
    let (inds, coeff) = self.get_coeffs_indices_build(&expr)?;
    self.check_apicall(unsafe {
      ffi::GRBaddrangeconstr(self.ptr,
                             coeff.len() as ffi::c_int,
                             inds.as_ptr(),
                             coeff.as_ptr(),
                             lb,
                             ub,
                             constrname.as_ptr())
    })?;

    let lazy = self.update_mode_lazy()?;
    let var = self.vars.add_new(lazy);
    let cons = self.constrs.add_new(lazy);
    Ok((var, cons))
  }

  #[allow(unused_variables)]
  /// Add range constraints to the model.
  pub fn add_ranges<'a, I, N>(&mut self, ranges_with_names: I) -> Result<(Vec<Var>, Vec<Constr>)> where
    N: AsRef<str> + 'a,
    I: IntoIterator<Item=(&'a N, RangeExpr)>
  {
    let ranges_with_names = ranges_with_names.into_iter();
    let (nconstr, _) = ranges_with_names.size_hint();
    let mut names = Vec::with_capacity(nconstr); // needed to ensure CString lives long enough
    let mut cnames = Vec::with_capacity(nconstr);
    let mut ubs = Vec::with_capacity(nconstr);
    let mut lbs = Vec::with_capacity(nconstr);
    let mut cbeg = Vec::with_capacity(nconstr);
    let mut cind = Vec::with_capacity(nconstr);
    let mut cval = Vec::with_capacity(nconstr);

    let mut c_start = 0;
    for (n, r) in ranges_with_names {
      let n = CString::new(n.as_ref())?;
      cnames.push(n.as_ptr());
      names.push(n);
      let (expr, lb, ub) = r.into_normalised()?;
      ubs.push(ub);
      lbs.push(lb);

      let (var_coeff, _) = expr.into_parts();
      let nterms = var_coeff.len();
      cbeg.push(c_start);
      c_start += nterms as i32;

      cind.reserve(nterms);
      cval.reserve(nterms);
      for (var, coeff) in var_coeff {
        cind.push(self.get_index_build(&var)?);
        cval.push(coeff);
      }
    }

    self.check_apicall(unsafe {
      ffi::GRBaddrangeconstrs(self.ptr,
                              cnames.len() as ffi::c_int,
                              cbeg.len() as ffi::c_int,
                              cbeg.as_ptr(),
                              cind.as_ptr(),
                              cval.as_ptr(),
                              lbs.as_ptr(),
                              ubs.as_ptr(),
                              cnames.as_ptr())
    })?;

    let ncons = names.len();
    let lazy = self.update_mode_lazy()?;
    let vars = vec![self.vars.add_new(lazy); ncons];
    let cons = vec![self.constrs.add_new(lazy); ncons];
    Ok((vars, cons))
  }

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self, name: &str, constraint: IneqExpr) -> Result<QConstr> {
    let (lhs, sense, rhs) = constraint.into_normalised_quad();
    let cname = CString::new(name)?;
    let (qrow, qcol, qval) = self.get_qcoeffs_indices_build(&lhs)?;
    let (_, lexpr) = lhs.into_parts();
    let (lvar, lval) = self.get_coeffs_indices_build(&lexpr)?;
    self.check_apicall(unsafe {
      ffi::GRBaddqconstr(self.ptr,
                         lval.len() as ffi::c_int,
                         lvar.as_ptr(),
                         lval.as_ptr(),
                         qval.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr(),
                         sense.into(),
                         rhs,
                         cname.as_ptr())
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
      ffi::GRBaddsos(self.ptr,
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
    let expr: Expr = expr.into();
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
  /// use grb::prelude::*;
  /// let mut m = Model::new("model").unwrap();
  /// let x = add_binvar!(m).unwrap();
  /// let y = add_binvar!(m).unwrap();
  /// let c = m.add_constr("constraint", c!(x + y == 1)).unwrap();
  /// assert_eq!(m.get_constr_by_name("constraint").unwrap_err(), grb::Error::ModelUpdateNeeded);
  /// m.update().unwrap();
  /// assert_eq!(m.get_constr_by_name("constraint").unwrap(), Some(c));
  /// assert_eq!(m.get_constr_by_name("foo").unwrap(), None);
  /// ```
  pub fn get_constr_by_name(&self, name: &str) -> Result<Option<Constr>> {
    if self.constrs.model_update_needed() { return Err(Error::ModelUpdateNeeded); }
    let n = CString::new(name)?;
    let mut idx = i32::min_value();
    self.check_apicall(unsafe { ffi::GRBgetconstrbyname(self.ptr, n.as_ptr(), &mut idx) })?;
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
    if self.vars.model_update_needed() { return Err(Error::ModelUpdateNeeded); }
    let n = CString::new(name)?;
    let mut idx = i32::min_value();
    self.check_apicall(unsafe { ffi::GRBgetvarbyname(self.ptr, n.as_ptr(), &mut idx) })?;
    if idx < 0 {
      Ok(None)
    } else {
      Ok(Some(self.vars.objects()[idx as usize])) // should only panic if there's a bug in IdxManager
    }
  }


  /// Query a Model attribute
  pub fn get_attr<A: Attr>(&self, attr: A) -> Result<A::Value> {
    unsafe { attr.get(self.ptr) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Query a model object attribute (Constr, Var, etc)
  pub fn get_obj_attr<A, E>(&self, attr: A, elem: &E) -> Result<A::Value>
    where
      A: Attr,
      E: ModelObject
  {
    let index = self.get_index(elem)?;
    unsafe { attr.get_element(self.ptr, index) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Query an attribute of multiple model objectis
  pub fn get_obj_attr_batch<A, E>(&self, attr: A, elem: &[E]) -> Result<Vec<A::Value>>
    where
      A: Attr,
      E: ModelObject
  {
    let index = self.get_indices(elem)?;
    unsafe { attr.get_elements(self.ptr, &index) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Set a Model attribute
  pub fn set_attr<A: Attr>(&self, attr: A, value: A::Value) -> Result<()> {
    unsafe { attr.set(self.ptr, value) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Set an attribute of a Model object (Const, Var, etc)
  pub fn set_obj_attr<A, E>(&self, attr: A, elem: &E, value: A::Value) -> Result<()>
    where
      A: Attr,
      E: ModelObject
  {
    let index = self.get_index_build(elem)?;
    unsafe { attr.set_element(self.ptr, index, value) }.map_err(|code| self.env.error_from_api(code))
  }

  /// Set an attribute of multiple Model objects (Const, Var, etc)
  pub fn set_obj_attr_batch<A, E>(&self, attr: A, elem: &[E], values: &[A::Value]) -> Result<()>
    where
      A: Attr,
      E: ModelObject
  {
    if elem.len() != values.len() {
      return Err(Error::InconsistentDims);
    }
    let indices = self.get_indices_build(elem)?;
    unsafe { attr.set_elements(self.ptr, &indices, values) }.map_err(|code| self.env.error_from_api(code))
  }

  pub fn set_param<P: Param>(&mut self, param: P, value: P::Value) -> Result<()> {
    self.get_env_mut().set(param, value)
  }

  pub fn get_param<P: Param>(&self, param: P) -> Result<P::Value> {
    self.get_env().get(param)
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
  /// this method with [`Model::try_clone()`].
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
      ffi::GRBfeasrelax(self.ptr,
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
    let new_vars = (0..n_vars - n_old_vars).map(|_| self.vars.add_new(lazy)).collect();

    let n_cons = self.get_attr(attr::NumConstrs)? as usize;
    assert!(n_cons >= n_old_constr);
    let new_cons = (0..n_cons - n_old_constr).map(|_| self.constrs.add_new(lazy)).collect();

    let n_qcons = self.get_attr(attr::NumQConstrs)? as usize;
    assert!(n_qcons >= n_old_qconstr);
    let new_qcons = (0..n_cons - n_old_constr).map(|_| self.qconstrs.add_new(lazy)).collect();

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
      ffi::GRBsetpwlobj(self.ptr,
                        self.get_index_build(var)?,
                        x.len() as ffi::c_int,
                        x.as_ptr(),
                        y.as_ptr())
    })
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
    self.check_apicall(unsafe { O::gurobi_remove(self.ptr, &[idx]) })
  }


  /// Retrieve a single constant matrix coefficient of the model.
  pub fn get_coeff(&self, var: &Var, constr: &Constr) -> Result<f64> {
    let mut value = 0.0;
    self.check_apicall(unsafe {
      ffi::GRBgetcoeff(self.ptr, self.get_index_build(constr)?, self.get_index_build(var)?, &mut value)
    })?;
    Ok(value)
  }

  /// Change a single constant matrix coefficient of the model.
  pub fn set_coeff(&mut self, var: &Var, constr: &Constr, value: f64) -> Result<()> {
    self.check_apicall(unsafe {
      ffi::GRBchgcoeffs(self.ptr, 1, &self.get_index_build(constr)?, &self.get_index_build(var)?, &value)
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
      ffi::GRBchgcoeffs(self.ptr,
                        vars.len() as ffi::c_int,
                        constrs.as_ptr(),
                        vars.as_ptr(),
                        values.as_ptr())
    })
  }

  // add quadratic terms of objective function.
  fn add_qpterms(&mut self, qrow: &[i32], qcol: &[i32], qval: &[f64]) -> Result<()> {
    self.check_apicall(unsafe {
      ffi::GRBaddqpterms(self.ptr,
                         qrow.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr())
    })
  }

  // remove quadratic terms of objective function.
  fn del_qpterms(&mut self) -> Result<()> {
    self.check_apicall(unsafe { ffi::GRBdelq(self.ptr) })
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
    unsafe { ffi::GRBfreemodel(self.ptr) };
  }
}

/// A handle to an [`AsyncModel`](crate::model::AsyncModel) which is currently solving.
pub struct AsyncHandle(Model);

impl AsyncHandle {
  /// Retrieve current the `attr::Status` of the model.
  pub fn status(&self) -> Result<Status> {
    self.0.status()
  }

  /// Retrieve the current `attr::ObjVal` of the model.
  pub fn obj_val(&self) -> Result<f64> {
    self.0.get_attr(attr::ObjVal)
  }

  /// Retrieve the current  `attr::ObjBound` of the model.
  pub fn obj_bnd(&self) -> Result<f64> {
    self.0.get_attr(attr::ObjBound)
  }

  /// Retrieve the current `attr::IterCount` of the model.
  pub fn iter_cnt(&self) -> Result<f64> {
    self.0.get_attr(attr::IterCount)
  }

  /// Retrieve the current `attr::BarIterCount` of the model.
  pub fn bar_iter_cnt(&self) -> Result<i32> {
    self.0.get_attr(attr::BarIterCount)
  }

  /// Retrieve the current `attr::NodeCount` of the model.
  pub fn node_cnt(&self) -> Result<f64> {
    self.0.get_attr(attr::NodeCount)
  }

  /// Wait for optimisation to finish.
  ///
  /// # Errors
  /// An [`Error::FromAPI`] may occur during optimisation, in which case it is stored in the `Result`.
  pub fn join(self) -> (AsyncModel, Result<()>) {
    let errors = self.0.check_apicall(unsafe { ffi::GRBsync(self.0.ptr) });
    (AsyncModel(self.0), errors)
  }
}

/// A wrapper around [`Model`] that supports async optimisation in the background.
///
///  From the Gurobi [manual](https://www.gurobi.com/documentation/9.1/refman/c_optimizeasync.html), regarding solving models asynchronously:
///
/// *"[modifying or performing non-permitted] calls on the running model, **or on any other models that were built within the same Gurobi environment**,
///  will fail with error code `OPTIMIZATION_IN_PROGRESS`."*
///
/// For this reason, creating an `AsyncModel` requires a [`Model`] whose [`Env`] wasn't previously been used to construct other models.
///
/// `Model` implements `From<AsyncModel>`, so you can recover the `Model` using `.into()` (see examples).
pub struct AsyncModel(Model);

impl AsyncModel {
  /// # Panics
  /// This function will panic if the `model` does not have sole ownership over its `Env`.  This means
  /// the `Model` cannot be created with [`Model::new`], instead you must use [`Model::with_env`].
  /// # Examples
  ///
  /// This example panics because `env` has two references - inside `m` and the bound variable in the current scope
  /// ```should_panic
  /// use grb::prelude::*;
  /// use grb::AsyncModel;
  ///
  /// let env = Env::new("")?;
  /// let mut m = Model::with_env("model", &env)?;
  /// let mut m =  AsyncModel::new(m); // panic - env is still in scope
  /// # Ok::<(), grb::Error>(())
  /// ```
  /// This is easily resolved by ensuring `env` is no longer in scope when the `AsyncModel` is created.
  /// ```
  /// # use grb::prelude::*;
  /// # use grb::AsyncModel;
  /// # let env = Env::new("")?;
  /// let mut m = Model::with_env("model", &env)?;
  /// drop(env);
  /// let mut m =  AsyncModel::new(m); // ok
  /// # Ok::<(), grb::Error>(())
  /// ```
  /// This example panics because `m` uses the default `Env`, which is also stored globally.
  /// `Model`s created with [`Model::new`] can never be made into `AsyncModel`s for this reason.
  /// ```should_panic
  /// # use grb::prelude::*;
  /// # use grb::AsyncModel;
  /// let m = Model::new("model1")?;
  /// let m =  AsyncModel::new(m); // panic
  /// # Ok::<(), grb::Error>(())
  /// ```
  ///
  pub fn new(model: Model) -> AsyncModel {
    if model.env.is_shared() {
      panic!("Cannot create async model - environment is used in other models");
    }
    AsyncModel(model)
  }

  /// Optimize the model on another thread.  This method will always trigger a [`Model::update`] on the underlying `Model`.
  ///
  /// On success, returns an [`AsyncHandle`](crate::model::AsyncHandle) that provides a limited API for model queries.
  /// The `AsyncModel` can be retrieved by calling [`AsyncHandle::join`](crate::model::AsyncHandle::join).
  ///
  /// # Errors
  /// An `grb::Error::FromAPI` may occur.  In this case, the `Err` variant contains this error
  /// and gives back ownership of this `AsyncModel`.
  ///
  ///
  /// # Examples
  /// ```
  /// use grb::prelude::*;
  /// use grb::AsyncModel;
  ///
  /// let mut m = Model::with_env("model", &Env::new("")?)?;
  /// let x = add_ctsvar!(m, obj: 2)?;
  /// let y = add_intvar!(m, bounds: 0..100)?;
  /// m.add_constr("c0", c!(x <= y - 0.5 ))?;
  /// let m = AsyncModel::new(m);
  ///
  /// let handle = match m.optimize() {
  ///   Err((_, e)) => panic!("{}", e),
  ///   Ok(h) => h
  /// };
  ///
  /// println!("The model has explored {} MIP nodes so far", handle.node_cnt()?);
  /// let (m, errors) = handle.join(); // the AsyncModel is always returned
  /// errors?; // optimisation errors - as if Model::optimize were called.
  /// let m: Model = m.into(); // get original Model back
  /// # Ok::<(), grb::Error>(())
  /// ```
  pub fn optimize(mut self) -> std::result::Result<AsyncHandle, (Self, Error)> {
    match self.0.update()
      .and_then(|_| self.0.check_apicall(unsafe { ffi::GRBoptimizeasync(self.0.ptr) }))
    {
      Ok(()) => Ok(AsyncHandle(self.0)),
      Err(e) => Err((self, e)),
    }
  }
}


impl std::convert::From<AsyncModel> for Model {
  fn from(model: AsyncModel) -> Model {
    model.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::prelude::*;

  extern crate self as grb;

  #[test]
  fn model_id_factory() {
    let mut env = Env::new("").unwrap();
    env.set(param::OutputFlag, 0).unwrap();

    let mut m1 = Model::with_env("test1", &env).unwrap();
    let mut m2 = Model::with_env("test2", &env).unwrap();

    let x1 = add_var!(m1, Binary,  name:"x1").unwrap();
    let x2 = add_var!(m2, Binary,  name:"x2").unwrap();
    assert_ne!(m1.id, m2.id);

    assert_eq!(x1.model_id, m1.id);
    assert_eq!(x1.id, 0);
    assert_eq!(x2.model_id, m2.id);
    assert_eq!(x2.id, 0);
  }


  #[test]
  fn eager_update() {
    let mut model = Model::new("test").unwrap();
    assert_eq!(model.get_param(param::UpdateMode).unwrap(), 1);
    assert!(!model.update_mode_lazy().unwrap());
    let x = add_binvar!(model, name:"x").unwrap();
    let y = add_binvar!(model, name:"y").unwrap();
    let c1 = model.add_constr("c1", c!(x + y <= 1)).unwrap(); // should work fine

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

    let z = add_binvar!(model, name:"z").unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y).unwrap(), 1);
    assert!(model.get_index(&z).is_err());
    assert_eq!(model.get_index_build(&z).unwrap(), 2);

    model.remove(y).unwrap();
    let c2 = model.add_constr("c2", c!(z + y <= 1)).unwrap(); // I know it's weird, because y is removed, but that's what Gurobi does
    assert_eq!(model.get_index_build(&c2).unwrap(), 1);
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y), Err(Error::ModelObjectRemoved)); // No longer available
    assert!(model.get_index(&z).is_err());
    assert_eq!(model.get_index_build(&z).unwrap(), 2);

    let w = add_binvar!(model, name:"w").unwrap();
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
    let mut model = Model::with_env("bug", &env).unwrap();
    assert!(model.update_mode_lazy().unwrap());

    let x = add_binvar!(model, name:"x").unwrap();
    let y = add_binvar!(model, name:"y").unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 0);
    assert_eq!(model.get_index_build(&x).unwrap_err(), Error::ModelObjectPending);
    assert_eq!(model.get_index_build(&y).unwrap_err(), Error::ModelObjectPending);
    model.add_constr("c1", c!(x + y <= 1)).unwrap_err();

    model.update().unwrap();
    assert_eq!(model.get_attr(attr::NumVars).unwrap(), 2);
    assert_eq!(model.get_index(&x).unwrap(), 0);
    assert_eq!(model.get_index(&y).unwrap(), 1);
    let c1 = model.add_constr("c1", c!(x + y <= 1)).unwrap();

    model.remove(y).unwrap();
    let z = add_binvar!(model, name:"z").unwrap();
    let w = add_binvar!(model, name:"w").unwrap();
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

    let mut model1 = Model::with_env("model1", &env).unwrap();
    let mut model2 = Model::with_env("model1", &env).unwrap();

    let x1 = add_binvar!(model1, name:"x1").unwrap();
    let y1 = add_binvar!(model1, name:"y1").unwrap();
    let x2 = add_binvar!(model2, name:"x2").unwrap();
    let y2 = add_binvar!(model2, name:"y2").unwrap();

    model1.add_constr("", c!(x1 <= y1)).unwrap();
    model2.add_constr("", c!(x2 <= y2)).unwrap();
    assert_eq!(model1.add_constr("", c!(x2 <= y1)).unwrap_err(), Error::ModelObjectMismatch);
    assert_eq!(model1.add_constr("", c!(x1 <= y2)).unwrap_err(), Error::ModelObjectMismatch);

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
    let mut model = Model::with_env("test", &env)?;
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
    let mut model = Model::with_env("test", &env)?;
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
    let mut m1 = Model::with_env("m1", &env)?;
    let m2 = m1.try_clone()?;

    let m1_env = m1.get_env_mut();
    let m2_env = m2.get_env();

    m1_env.set(param::OutputFlag, 1)?;

    assert_eq!(m1_env.get(param::OutputFlag), Ok(1));
    assert_eq!(m2_env.get(param::OutputFlag), Ok(0));
    Ok(())
  }

  #[test]
  fn fixed_mip_model_copies_env() -> Result<()> {
    let mut m = Model::new("")?;
    m.set_param(param::OutputFlag, 0)?;
    let x = add_var!(m, Continuous, name:"x")?;
    let y = add_binvar!(m, name:"y")?;

    m.add_constr("c1", c!(x + y <= 1))?;
    m.add_constr("c1", c!(x - y <= 2))?;

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
    let env = Env::new("")?;
    let m1 = Model::with_env("test", &env)?;
    let filename = "test_read_model_copies_env.lp";
    m1.write(filename)?;
    let m2 = Model::read_from(filename, &env)?;
    assert_ne!(m2.get_env().as_ptr(), m1.get_env().as_ptr());
    Ok(())
  }

  #[test]
  fn copy_env_model_to_model() -> Result<()> {
    let env = Env::new("")?;
    let m1 = Model::with_env("", &env)?;
    let m2 = Model::with_env("", m1.get_env())?;

    assert_ne!(m1.get_env().as_ptr(), m2.get_env().as_ptr());
    Ok(())
  }
}
