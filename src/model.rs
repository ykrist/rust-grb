use grb_sys2 as ffi;
use grb_sys2::{c_int, GRBmodel};
use std::borrow::Borrow;
use std::ffi::CString;
use std::mem::transmute;
use std::path::Path;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::attribute::{ModelAttrGet, ModelAttrSet, ObjAttrGet, ObjAttrSet};
use crate::callback::{callback_wrapper, UserCallbackData};
use crate::constr::{IneqExpr, RangeExpr};
use crate::expr::{LinExpr, QuadExpr};
use crate::model_object::IdxManager;
use crate::parameter::{ParamGet, ParamSet};
use crate::prelude::*;
use crate::util::AsPtr;
use crate::{Error, Result};

/// Gurobi Model object.
///
/// This will be where the bulk of interactions with Gurobi occur.
pub struct Model {
    ptr: *mut GRBmodel,
    #[allow(dead_code)]
    id: u32,
    env: Env,
    pub(crate) vars: IdxManager<Var>,
    pub(crate) constrs: IdxManager<Constr>,
    pub(crate) genconstrs: IdxManager<GenConstr>,
    pub(crate) qconstrs: IdxManager<QConstr>,
    pub(crate) sos: IdxManager<SOS>,
}

macro_rules! impl_object_list_getter {
    ($name:ident, $t:ty, $attr:ident, $noun:literal) => {
        #[doc = "Retrieve the "]
        #[doc=$noun]
        #[doc = " in the model. \n\n # Errors\nReturns an error if a model update is needed"]
        pub fn $name(&self) -> Result<&[$t]> {
            if self.$attr.model_update_needed() {
                Err(Error::ModelUpdateNeeded)
            } else {
                Ok(self.$attr.objects())
            }
        }
    };
}

impl AsPtr for Model {
    type Ptr = GRBmodel;
    unsafe fn as_mut_ptr(&self) -> *mut GRBmodel {
        self.ptr
    }
}

/// Set the strategy used for performing a piecewise-linear approximation of a function constraint
///
/// # Examples
/// ```
/// # use grb::prelude::*;
/// let mut m = Model::new("model")?;
/// let x = add_ctsvar!(m)?;
/// let y = add_ctsvar!(m)?;
/// m.add_genconstr_sin(\"c1\", x, y, Some(FuncConstrOptions::PieceWidth(2.5))?;");
/// # Ok::<(), grb::Error>(())
#[non_exhaustive]
pub enum FuncConstrOptions {
    /// Ignore the attribute settings for this function constraint and use the parameter settings ([`FuncPieces`](`crate::param::FuncPieces`), etc.) instead
    Ignore,
    /// Set the number of pieces; pieces are equal width
    /// The value must be greater or equal to 2.
    NumPieces(usize),
    /// Use a fixed width for each piece
    PieceWidth(f64),
    /// Bound the absolute error of the approximation
    MaxAbsoluteError(f64),
    /// Bound the relative error of the approximation
    MaxRelativeError(f64),
}

impl From<FuncConstrOptions> for CString {
    fn from(value: FuncConstrOptions) -> Self {
        use FuncConstrOptions as O;
        let res = match value {
            O::Ignore => "FuncPieces=0".to_owned(),

            // HACK: I know this is not the perfect place to validate `FunConstrOptions`, but I couldn't
            // find any earlier opportunities to do so.
            O::NumPieces(0 | 1) => {
                panic!("Invalid number of pieces. The value must be above or equal 2.")
            }
            O::NumPieces(n) => format!("FuncPieces={n}"),

            O::PieceWidth(w) => format!("FuncPieces=1 FuncPieceLength={w}"),
            O::MaxAbsoluteError(e) => format!("FuncPieces=-1 FuncPieceError={e}"),
            O::MaxRelativeError(e) => format!("FuncPieces=-2 FuncPieceError={e}"),
        };
        CString::new(res).expect("the strings above don't contain NULs")
    }
}

macro_rules! impl_func_constr {
    ($name:literal, $formula:literal, $fn_name:ident, $ffi_fn_name:path) => {
        #[doc = concat!("Add ", $name, " function constraint to the model.")]
        ///
        #[doc = $formula]
        ///
        /// # Examples
        /// ```
        /// # use grb::prelude::*;
        /// let mut m = Model::new("model")?;
        /// let x = add_ctsvar!(m)?;
        /// let y = add_ctsvar!(m)?;
        #[doc = concat!("m.", stringify!($fn_name), "(\"c1\", x, y, None)?;")]
        /// # Ok::<(), grb::Error>(())
        /// ```
        pub fn $fn_name(
            &mut self,
            name: &str,
            x: Var,
            y: Var,
            options: Option<FuncConstrOptions>,
        ) -> Result<GenConstr> {
            let constrname = CString::new(name)?;
            let x_idx = self.get_index_build(&x)?;
            let y_idx = self.get_index_build(&y)?;
            let options = options.map(CString::from).unwrap_or_default();

            self.check_apicall(unsafe {
                $ffi_fn_name(
                    self.ptr,
                    constrname.as_ptr(),
                    x_idx,
                    y_idx,
                    options.as_ptr(),
                )
            })?;

            Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
        }
    };
}

macro_rules! impl_funca_constr {
    ($name:literal, $formula:literal, $fn_name:ident, $ffi_fn_name:path) => {
        #[doc = concat!("Add ", $name, " function constraint to the model.")]
        ///
        #[doc = $formula]
        ///
        /// # Examples
        /// ```
        /// # use grb::prelude::*;
        /// let mut m = Model::new("model")?;
        /// let x = add_ctsvar!(m)?;
        /// let y = add_ctsvar!(m)?;
        /// let a = 5.0;
        #[doc=concat!("m.", stringify!($fn_name), "(\"c1\", x, y, a, None)?;")]
        /// # Ok::<(), grb::Error>(())
        /// ```
        pub fn $fn_name(
            &mut self,
            name: &str,
            x: Var,
            y: Var,
            a: f64,
            options: Option<FuncConstrOptions>,
        ) -> Result<GenConstr> {
            let constrname = CString::new(name)?;
            let x_idx = self.get_index_build(&x)?;
            let y_idx = self.get_index_build(&y)?;
            let options = options.map(CString::from).unwrap_or_default();

            self.check_apicall(unsafe {
                $ffi_fn_name(
                    self.ptr,
                    constrname.as_ptr(),
                    x_idx,
                    y_idx,
                    a,
                    options.as_ptr(),
                )
            })?;

            Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
        }
    };
}

impl Model {
    fn next_id() -> u32 {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }

    fn model_update_needed(&self) -> bool {
        self.vars.model_update_needed()
            || self.constrs.model_update_needed()
            || self.genconstrs.model_update_needed()
            || self.qconstrs.model_update_needed()
            || self.sos.model_update_needed()
    }

    fn build_idx_arrays_obj_obj<O1, O2, T>(
        &self,
        iter: impl Iterator<Item = (O1, O2, T)>,
    ) -> Result<(Vec<i32>, Vec<i32>, Vec<T>)>
    where
        O1: ModelObject,
        O2: ModelObject,
    {
        let n = iter.size_hint().0;
        let mut ov1 = Vec::with_capacity(n);
        let mut ov2 = Vec::with_capacity(n);
        let mut tv = Vec::with_capacity(n);

        for (o1, o2, t) in iter {
            ov1.push(self.get_index_build(&o1)?);
            ov2.push(self.get_index_build(&o2)?);
            tv.push(t);
        }

        Ok((ov1, ov2, tv))
    }

    fn build_idx_arrays_obj<O, T>(
        &self,
        iter: impl Iterator<Item = (O, T)>,
    ) -> Result<(Vec<i32>, Vec<T>)>
    where
        O: ModelObject,
    {
        let n = iter.size_hint().0;
        let mut ov = Vec::with_capacity(n);
        let mut tv = Vec::with_capacity(n);

        for (o, t) in iter {
            ov.push(self.get_index_build(&o)?);
            tv.push(t);
        }

        Ok((ov, tv))
    }

    #[inline]
    pub(crate) fn get_index<O: ModelObject>(&self, item: &O) -> Result<i32> {
        O::idx_manager(self).get_index(item)
    }

    #[inline]
    pub(crate) fn get_index_build<O: ModelObject>(&self, item: &O) -> Result<i32> {
        O::idx_manager(self).get_index_build(item)
    }

    #[inline]
    pub(crate) fn get_coeffs_indices_build(&self, expr: &LinExpr) -> Result<(Vec<i32>, Vec<f64>)> {
        let nterms = expr.num_terms();
        let mut inds = Vec::with_capacity(nterms);
        let mut coeff = Vec::with_capacity(nterms);
        for (x, &c) in expr.iter_terms() {
            inds.push(self.get_index_build(x)?);
            coeff.push(c);
        }
        Ok((inds, coeff))
    }

    #[inline]
    pub(crate) fn get_qcoeffs_indices_build(
        &self,
        expr: &QuadExpr,
    ) -> Result<(Vec<i32>, Vec<i32>, Vec<f64>)> {
        let nqterms = expr.num_qterms();
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
    fn from_raw(env: &Env, model: *mut GRBmodel) -> Result<Model> {
        assert!(!model.is_null());
        let env_ptr = unsafe { ffi::GRBgetenv(model) };
        if env_ptr.is_null() {
            return Err(Error::FromAPI(
                "Failed to retrieve GRBenv from given model".to_owned(),
                2002,
            ));
        }
        let env = unsafe { Env::new_gurobi_allocated(env, env_ptr) };
        let id = Model::next_id();

        let mut model = Model {
            ptr: model,
            id,
            env,
            vars: IdxManager::new(id),
            constrs: IdxManager::new(id),
            genconstrs: IdxManager::new(id),
            qconstrs: IdxManager::new(id),
            sos: IdxManager::new(id),
        };

        let nvars = model.get_attr(attr::NumVars)?;
        let nconstr = model.get_attr(attr::NumConstrs)?;
        let ngenconstr = model.get_attr(attr::NumGenConstrs)?;
        let nqconstr = model.get_attr(attr::NumQConstrs)?;
        let sos = model.get_attr(attr::NumSOS)?;

        model.vars = IdxManager::new_with_existing_obj(id, nvars as usize);
        model.constrs = IdxManager::new_with_existing_obj(id, nconstr as usize);
        model.genconstrs = IdxManager::new_with_existing_obj(id, ngenconstr as usize);
        model.qconstrs = IdxManager::new_with_existing_obj(id, nqconstr as usize);
        model.sos = IdxManager::new_with_existing_obj(id, sos as usize);

        Ok(model)
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
            ffi::GRBnewmodel(
                env.as_mut_ptr(),
                &mut model,
                modelname.as_ptr(),
                0,
                null(),
                null(),
                null(),
                null(),
                null(),
            )
        })?;
        Self::from_raw(env, model)
    }

    /// Create a new model with the default environment, which is lazily initialised.
    pub fn new(modelname: &str) -> Result<Model> {
        Env::DEFAULT_ENV.with(|env| Model::with_env(modelname, env))
    }

    /// Create a copy of the model.  This method is fallible due to the lazy update approach and the underlying
    /// Gurobi C API, so a [`Clone`] implementation is not provided.
    ///
    /// # Errors
    ///  * [`Error::FromAPI`] if a Gurobi error occurs
    ///  * [`Error::ModelUpdateNeeded`] if model objects have been added to the model since the last update.
    pub fn try_clone(&self) -> Result<Model> {
        if self.model_update_needed() {
            return Err(Error::ModelUpdateNeeded);
        }

        let copied = unsafe { ffi::GRBcopymodel(self.ptr) };
        if copied.is_null() {
            return Err(Error::FromAPI(
                "Failed to create a copy of the model".to_owned(),
                20002,
            ));
        }

        Model::from_raw(&self.env, copied)
    }

    #[deprecated(note = "use `Model::from_file_with_env` instead")]
    /// This function has been deprecated in favour of [`Model::from_file_with_env`] and [`Model::from_file`]
    pub fn read_from(filename: &str, env: &Env) -> Result<Model> {
        Model::from_file_with_env(filename, env)
    }

    /// Read a model from a file using the default `Env`.
    pub fn from_file(filename: impl AsRef<Path>) -> Result<Model> {
        Env::DEFAULT_ENV.with(|env| Model::from_file_with_env(filename, env))
    }

    /// Read a model from a file.  See the [manual](https://www.gurobi.com/documentation/9.1/refman/c_readmodel.html) for accepted file formats.
    pub fn from_file_with_env(filename: impl AsRef<Path>, env: &Env) -> Result<Model> {
        let filename = crate::util::path_to_cstring(filename)?;
        let mut model = null_mut();
        env.check_apicall(unsafe {
            ffi::GRBreadmodel(env.as_mut_ptr(), filename.as_ptr(), &mut model)
        })?;
        Self::from_raw(env, model)
    }

    /// Create the fixed model associated with the current MIP model.
    ///
    /// The model must be MIP and have a solution loaded. In the fixed model,
    /// each integer variable is fixed to the value that it takes in the current MIP solution.
    pub fn fixed(&mut self) -> Result<Model> {
        let mut fixed: *mut GRBmodel = null_mut();
        self.check_apicall(unsafe { ffi::GRBfixmodel(self.ptr, &mut fixed) })?;
        Model::from_raw(&self.env, fixed)
    }

    /// Get shared reference to the environment associated with the model.
    pub fn get_env(&self) -> &Env {
        &self.env
    }

    /// Get mutable reference to the environment associated with the model.
    pub fn get_env_mut(&mut self) -> &mut Env {
        &mut self.env
    }

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
        self.genconstrs.update();
        self.qconstrs.update();
        self.sos.update();
        self.check_apicall(unsafe { ffi::GRBupdatemodel(self.ptr) })?;
        Ok(())
    }

    /// Query update mode. See <https://docs.gurobi.com/projects/optimizer/en/current/reference/parameters.html#parameterupdatemode>
    fn update_mode_lazy(&self) -> Result<bool> {
        //  0 => pending until update() or optimize() called.
        //  1 => all changes are immediate
        Ok(self.env.get(param::UpdateMode)? == 0)
    }

    fn call_with_callback<F>(
        &mut self,
        gurobi_routine: unsafe extern "C" fn(*mut GRBmodel) -> c_int,
        callback: &mut F,
    ) -> Result<()>
    where
        F: Callback,
    {
        self.update()?;
        let nvars = self.get_attr(attr::NumVars)? as usize;
        let mut usrdata = UserCallbackData {
            model: self,
            cb_obj: callback,
            nvars,
        };

        unsafe {
            let res = self
                .check_apicall(ffi::GRBsetcallbackfunc(
                    self.ptr,
                    Some(callback_wrapper),
                    transmute(&mut usrdata),
                ))
                .and_then(|()| self.check_apicall(gurobi_routine(self.ptr)));
            self.check_apicall(ffi::GRBsetcallbackfunc(self.ptr, None, null_mut()))
                .expect("failed to clear callback function");
            res
        }
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
    ///
    /// # Panics
    /// This function panics if Gurobi errors on clearing the callback.
    pub fn optimize_with_callback<F>(&mut self, callback: &mut F) -> Result<()>
    where
        F: Callback,
    {
        self.call_with_callback(ffi::GRBoptimize, callback)
    }

    /// Compute an Irreducible Inconsistent Subsystem (IIS) of the model.  The constraints in the IIS can be identified
    /// by checking their `IISConstr` attribute
    ///
    /// # Example
    /// ```
    /// # use grb::prelude::*;
    ///
    /// fn compute_iis_constraints(m: &mut Model) -> grb::Result<Vec<Constr>> {
    ///    m.compute_iis()?;
    ///    let constrs = m.get_constrs()?; // all constraints in model
    ///    let iis_constrs = m.get_obj_attr_batch(attr::IISConstr, constrs.iter().copied())?
    ///     .into_iter()
    ///     .zip(constrs)
    ///     // IISConstr is 1 if constraint is in the IIS, 0 otherwise
    ///     .filter_map(|(is_iis, c)| if is_iis > 0 { Some(*c)} else { None })
    ///     .collect();
    ///     Ok(iis_constrs)
    /// }
    /// ```
    pub fn compute_iis(&mut self) -> Result<()> {
        self.check_apicall(unsafe { ffi::GRBcomputeIIS(self.ptr) })
    }

    /// Compute an IIS of the model with a callback.  Only the only variant of [`Where`] will be [`Where::IIS`].
    pub fn compute_iis_with_callback<F>(&mut self, callback: &mut F) -> Result<()>
    where
        F: Callback,
    {
        self.call_with_callback(ffi::GRBcomputeIIS, callback)
    }

    /// Send a request to the model to terminate the current optimization process.
    pub fn terminate(&self) {
        unsafe { ffi::GRBterminate(self.ptr) }
    }

    /// Reset the model to an unsolved state.
    ///
    /// All solution information previously computed are discarded.
    pub fn reset(&self) -> Result<()> {
        self.check_apicall(unsafe { ffi::GRBresetmodel(self.ptr) })
    }

    /// Perform an automated search for parameter settings that improve performance on the model.
    /// See also references [on official
    /// manual](https://www.gurobi.com/documentation/6.5/refman/parameter_tuning_tool.html#sec:Tuning).
    pub fn tune(&self) -> Result<()> {
        self.check_apicall(unsafe { ffi::GRBtunemodel(self.ptr) })
    }

    /// Prepare to retrieve the results of `tune()`.
    /// See also references [on official
    /// manual](https://www.gurobi.com/documentation/6.5/refman/parameter_tuning_tool.html#sec:Tuning).
    pub fn get_tune_result(&self, n: i32) -> Result<()> {
        self.check_apicall(unsafe { ffi::GRBgettuneresult(self.ptr, n) })
    }

    /// Insert a message into log file.
    ///
    /// # Panics
    /// Panics when `message` cannot be converted to a nul-terminated C string.
    pub fn message(&self, message: &str) {
        self.env.message(message);
    }

    // FIXME: this should accept AsRef<Path> types (breaking change)
    /// Import optimization data from a file. This routine is the general entry point for importing
    /// data from a file into a model. It can be used to read start vectors for MIP models,
    /// basis files for LP models, or parameter settings. The type of data read is determined by the file suffix.
    /// File formats are described in the [manual](https://www.gurobi.com/documentation/9.1/refman/model_file_formats.html#sec:FileFormats).
    ///
    /// If you wish to construct a model from an format like `MPS` or `LP`, use [`Model::from_file`].
    pub fn read(&mut self, filename: &str) -> Result<()> {
        let filename = CString::new(filename)?;
        self.check_apicall(unsafe { ffi::GRBread(self.ptr, filename.as_ptr()) })
    }

    // FIXME: this should accept AsRef<Path> types (breaking change)
    /// Export a model to a file.
    ///
    /// The file type is encoded in the file name suffix. Valid suffixes are `.mps`, `.rew`, `.lp`, or `.rlp` for
    /// writing the model itself, `.ilp` for writing just the IIS associated with an infeasible model,
    /// `.sol` for writing the current solution, `.mst` for writing
    /// a start vector, `.hnt` for writing a hint file, `.bas` for writing an LP basis, `.prm` for writing modified
    /// parameter settings, `.attr` for writing model attributes, or `.json` for writing solution information in
    /// JSON format. If your system has compression utilities installed (e.g., 7z or zip for Windows, and gzip,
    /// bzip2, or unzip for Linux or Mac OS), then the files can be compressed, so additional suffixes of `.gz`,
    /// `.bz2`, or `.7z` are accepted.
    pub fn write(&self, filename: &str) -> Result<()> {
        let filename = CString::new(filename)?;
        self.check_apicall(unsafe { ffi::GRBwrite(self.ptr, filename.as_ptr()) })
    }

    /// Add a decision variable to the model.  This method allows the user to give the entire column (constraint coefficients).
    ///
    /// The [`add_var!`](crate::add_var) macro and its friends are usually easier to use.
    #[allow(clippy::too_many_arguments)]
    pub fn add_var(
        &mut self,
        name: &str,
        vtype: VarType,
        obj: f64,
        lb: f64,
        ub: f64,
        col_coeff: impl IntoIterator<Item = (Constr, f64)>,
    ) -> Result<Var> {
        let name = CString::new(name)?;
        let mut col_coeff = col_coeff.into_iter().peekable();
        let (numnz, _vind, _vval, vind, vval) = if col_coeff.peek().is_some() {
            let (constrs, vals) = self.build_idx_arrays_obj(col_coeff)?;
            let c_ptr = constrs.as_ptr();
            let v_ptr = vals.as_ptr();
            (
                constrs.len() as c_int,
                Some(constrs),
                Some(vals),
                c_ptr,
                v_ptr,
            )
        } else {
            (0, None, None, std::ptr::null(), std::ptr::null())
        };
        self.check_apicall(unsafe {
            ffi::GRBaddvar(
                self.ptr,
                numnz,
                vind,
                vval,
                obj,
                lb,
                ub,
                vtype.into(),
                name.as_ptr(),
            )
        })?;
        Ok(self.vars.add_new(self.update_mode_lazy()?))
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
    pub fn add_constr(&mut self, name: &str, con: IneqExpr) -> Result<Constr> where {
        let (lhs, sense, rhs) = con.into_normalised_linear()?;
        let constrname = CString::new(name)?;
        let (vinds, cval) = self.get_coeffs_indices_build(&lhs)?;
        self.check_apicall(unsafe {
            ffi::GRBaddconstr(
                self.ptr,
                cval.len() as ffi::c_int,
                vinds.as_ptr(),
                cval.as_ptr(),
                sense as ffi::c_char,
                rhs,
                constrname.as_ptr(),
            )
        })?;

        Ok(self.constrs.add_new(self.update_mode_lazy()?))
    }

    /// Add multiple linear constraints to the model in a single Gurobi API call.
    ///
    /// Accepts anything that can be turned into an iterator of `(name, constraint)` pairs
    /// where `name : AsRef<str>` (eg `&str` or `String`) and `constraint` is a *linear* [`IneqExpr`].
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x = add_ctsvar!(m)?;
    /// let y = add_ctsvar!(m)?;
    ///
    /// let constraints = vec![
    ///   (&"c1", c!(x <= 1 - y )),
    ///   (&"c2", c!(x == 0.5*y )),
    /// ];
    ///
    /// m.add_constrs(constraints)?;
    ///
    /// // store owned names in Vec to ensure they live long enough
    /// let more_constraints_names : Vec<_> =  (0..10).map(|i| format!("r{}", i)).collect();
    /// // A Map iterator of (&String, IneqConstr)
    /// let more_constraints = (0..10).map(|i| (&more_constraints_names[i], c!(x >= i*y )));
    /// m.add_constrs(more_constraints)?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    ///
    /// # Errors
    /// - [`Error::AlgebraicError`] if a nonlinear constraint is given.
    /// - [`Error::ModelObjectPending`] if some variables haven't yet been added to the model.
    /// - [`Error::ModelObjectRemoved`] if some variables have been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if some variables are from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    pub fn add_constrs<'a, I, S>(&mut self, constr_with_names: I) -> Result<Vec<Constr>>
    where
        I: IntoIterator<Item = (&'a S, IneqExpr)>,
        S: AsRef<str> + 'a,
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
            senses.push(sense as ffi::c_char);

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
            ffi::GRBaddconstrs(
                self.ptr,
                cnames.len() as ffi::c_int,
                cbeg.len() as ffi::c_int,
                cbeg.as_ptr(),
                cind.as_ptr(),
                cval.as_ptr(),
                senses.as_ptr(),
                rhs.as_ptr(),
                cnames.as_ptr(),
            )
        })?;

        let lazy = self.update_mode_lazy()?;
        Ok(vec![self.constrs.add_new(lazy); cnames.len()])
    }

    /// Add a MIN constraint to the model.
    ///
    /// A MIN constraint $r = \min\{x_1,\ldots,x_n,c\}$ states that
    /// the resultant variable $r$ should be equal to the minimum of
    /// the operand variables $x_1,\ldots,x_n$ and the constant $c$.
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x1 = add_ctsvar!(m, bounds: 2..)?;
    /// let x2 = add_ctsvar!(m, bounds: 3..)?;
    /// let x3 = add_ctsvar!(m, bounds: 1..)?;
    /// let y = add_ctsvar!(m)?;
    /// m.add_genconstr_min("c1", y, [x1, x2, x3], None)?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_min(
        &mut self,
        name: &str,
        resultant_var: Var,
        operand_vars: impl IntoIterator<Item = Var>,
        constant: Option<f64>,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let resvar_idx = self.get_index_build(&resultant_var)?;
        let vars: Vec<_> = operand_vars
            .into_iter()
            .map(|v| self.get_index_build(&v))
            .collect::<Result<_>>()?;
        let constant = constant.unwrap_or(INFINITY);

        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrMin(
                self.ptr,
                constrname.as_ptr(),
                resvar_idx as ffi::c_int,
                vars.len() as ffi::c_int,
                vars.as_ptr(),
                constant as ffi::c_double,
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add a MAX constraint to the model.
    ///
    /// A MAX constraint $r = \max\{x_1,\ldots,x_n,c\}$ states that
    /// the resultant variable $r$ should be equal to the maximum of
    /// the operand variables $x_1,\ldots,x_n$ and the constant $c$.
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x1 = add_ctsvar!(m, bounds: ..2)?;
    /// let x2 = add_ctsvar!(m, bounds: ..3)?;
    /// let x3 = add_ctsvar!(m, bounds: ..1)?;
    /// let y = add_ctsvar!(m)?;
    /// m.add_genconstr_max("c1", y, [x1, x2, x3], None)?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_max(
        &mut self,
        name: &str,
        resultant_var: Var,
        operand_vars: impl IntoIterator<Item = Var>,
        constant: Option<f64>,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let resvar_idx = self.get_index_build(&resultant_var)?;
        let vars: Vec<_> = operand_vars
            .into_iter()
            .map(|v| self.get_index_build(&v))
            .collect::<Result<_>>()?;
        let constant = constant.unwrap_or_default();

        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrMax(
                self.ptr,
                constrname.as_ptr(),
                resvar_idx as ffi::c_int,
                vars.len() as ffi::c_int,
                vars.as_ptr(),
                constant as ffi::c_double,
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add an ABS constraint to the model.
    ///
    /// An ABS constraint $r = \mbox{abs}\{x\}$ states that
    /// the resultant variable $r$ should be equal to
    /// the absolute value of the argument variable $x$.
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x = add_ctsvar!(m, bounds: -2..2)?;
    /// let y = add_ctsvar!(m)?;
    /// m.add_genconstr_abs("c1", y, x)?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_abs(
        &mut self,
        name: &str,
        resultant_var: Var,
        argument_var: Var,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let resvar_idx = self.get_index_build(&resultant_var)?;
        let argvar_idx = self.get_index_build(&argument_var)?;

        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrAbs(
                self.ptr,
                constrname.as_ptr(),
                resvar_idx as ffi::c_int,
                argvar_idx as ffi::c_int,
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add an AND constraint to the model.
    ///
    /// An AND constraint $r = \and\{x_1,\ldots,x_n,c\}$ states that
    /// the binary variable $r$ should be 1 if and only if
    /// all the operand variables $x_1,\ldots,x_n$ are equal to 1.
    /// If any of the operand variables is $0$, then the resultant should be $0$ as well.
    ///
    /// Note that all variables participating in such a constraint will be forced to be binary,
    /// independent of how they were created.
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x1 = add_binvar!(m)?;
    /// let x2 = add_binvar!(m)?;
    /// let x3 = add_binvar!(m)?;
    /// let y = add_binvar!(m)?;
    /// m.add_genconstr_and("c1", y, [x1, x2, x3])?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_and(
        &mut self,
        name: &str,
        resultant_var: Var,
        operand_vars: impl IntoIterator<Item = Var>,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let resvar_idx = self.get_index_build(&resultant_var)?;
        let vars: Vec<_> = operand_vars
            .into_iter()
            .map(|v| self.get_index_build(&v))
            .collect::<Result<_>>()?;

        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrAnd(
                self.ptr,
                constrname.as_ptr(),
                resvar_idx as ffi::c_int,
                vars.len() as ffi::c_int,
                vars.as_ptr(),
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add an OR constraint to the model.
    ///
    /// An OR constraint $r = \and\{x_1,\ldots,x_n,c\}$ states that
    /// the binary variable $r$ should be 1 if and only if
    /// any of the operand variables $x_1,\ldots,x_n$ is equal to 1.
    /// If all of the operand variables is $0$, then the resultant should be $0$ as well.
    ///
    /// Note that all variables participating in such a constraint will be forced to be binary,
    /// independent of how they were created.
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x1 = add_binvar!(m)?;
    /// let x2 = add_binvar!(m)?;
    /// let x3 = add_binvar!(m)?;
    /// let y = add_binvar!(m)?;
    /// m.add_genconstr_or("c1", y, [x1, x2, x3])?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_or(
        &mut self,
        name: &str,
        resultant_var: Var,
        operand_vars: impl IntoIterator<Item = Var>,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let resvar_idx = self.get_index_build(&resultant_var)?;
        let vars: Vec<_> = operand_vars
            .into_iter()
            .map(|v| self.get_index_build(&v))
            .collect::<Result<_>>()?;

        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrOr(
                self.ptr,
                constrname.as_ptr(),
                resvar_idx,
                vars.len() as ffi::c_int,
                vars.as_ptr(),
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add a NORM constraint to the model.
    ///
    /// A NORM constraint $r = \mbox{norm}\{x_1,\ldots,x_n,c\}$ states that
    /// the resultant variable $r$ should be equal to
    /// the vector norm of the argument vector $x_1,\ldots,x_n$.
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x1 = add_ctsvar!(m)?;
    /// let x2 = add_ctsvar!(m)?;
    /// let x3 = add_ctsvar!(m)?;
    /// let y = add_ctsvar!(m)?;
    /// m.add_genconstr_norm("c1", y, [x1, x2, x3], Norm::L1)?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_norm(
        &mut self,
        name: &str,
        resultant_var: Var,
        operand_vars: impl IntoIterator<Item = Var>,
        norm: Norm,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let resvar_idx = self.get_index_build(&resultant_var)?;
        let vars: Vec<_> = operand_vars
            .into_iter()
            .map(|v| self.get_index_build(&v))
            .collect::<Result<_>>()?;
        let norm = match norm {
            Norm::L0 => 0.,
            Norm::L1 => 1.,
            Norm::L2 => 2.,
            Norm::LInfinity => INFINITY,
        };

        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrNorm(
                self.ptr,
                constrname.as_ptr(),
                resvar_idx,
                vars.len() as ffi::c_int,
                vars.as_ptr(),
                norm as ffi::c_double,
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add an indicator constraint to the model.
    ///
    /// The `con` argument is usually created with the [`c!`](crate::c) macro.
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let b = add_binvar!(m)?;
    /// let x = add_ctsvar!(m)?;
    /// let y = add_ctsvar!(m)?;
    /// m.add_genconstr_indicator("c1", b, true, c!(x <= 1 - y))?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_indicator(
        &mut self,
        name: &str,
        ind: Var,
        ind_val: bool,
        con: IneqExpr,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let (lhs, sense, rhs) = con.into_normalised_linear()?;
        let (vinds, cval) = self.get_coeffs_indices_build(&lhs)?;
        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrIndicator(
                self.ptr,
                constrname.as_ptr(),
                self.get_index_build(&ind)?,
                ind_val as ffi::c_int,
                cval.len() as ffi::c_int,
                vinds.as_ptr(),
                cval.as_ptr(),
                sense as ffi::c_char,
                rhs,
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add a piecewise-linear constraint to the model.
    ///
    /// A piecewise-linear constraint $y = f(x)$ states that
    /// the point $(x, y)$ must lie on the piecewise-linear function $f()$ defined by
    /// a set of points $(x_1, y_1), (x_2, y_2), ..., (x_n, y_n)$.
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x = add_ctsvar!(m)?;
    /// let y = add_ctsvar!(m)?;
    /// let points= [(1., 1.), (3., 2.), (5., 4.)];
    /// m.add_genconstr_pwl("c1", x, y, points)?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_pwl(
        &mut self,
        name: &str,
        x: Var,
        y: Var,
        points: impl IntoIterator<Item = (f64, f64)>,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let x_idx = self.get_index_build(&x)?;
        let y_idx = self.get_index_build(&y)?;
        let (x_points, y_points): (Vec<_>, Vec<_>) = points.into_iter().unzip();

        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrPWL(
                self.ptr,
                constrname.as_ptr(),
                x_idx,
                y_idx,
                x_points.len() as ffi::c_int,
                x_points.as_ptr(),
                y_points.as_ptr(),
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add a polynomial function constraint to the model.
    ///
    /// $y = p_0 x^n + p_1 x^{n-1} + \ldots + p_n x + p_{n+1}$
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x = add_ctsvar!(m)?;
    /// let y = add_ctsvar!(m)?;
    /// let coeffs = [3., 0., 0., 7., 3.];
    /// m.add_genconstr_poly("c1", x, y, coeffs.to_vec(), "")?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn add_genconstr_poly(
        &mut self,
        name: &str,
        x: Var,
        y: Var,
        mut coeffs: Vec<f64>,
        options: Option<FuncConstrOptions>,
    ) -> Result<GenConstr> {
        let constrname = CString::new(name)?;
        let x_idx = self.get_index_build(&x)?;
        let y_idx = self.get_index_build(&y)?;
        let options = options.map(CString::from).unwrap_or_default();

        self.check_apicall(unsafe {
            ffi::GRBaddgenconstrPoly(
                self.ptr,
                constrname.as_ptr(),
                x_idx,
                y_idx,
                coeffs.len() as ffi::c_int,
                coeffs.as_mut_ptr(),
                options.as_ptr(),
            )
        })?;

        Ok(self.genconstrs.add_new(self.update_mode_lazy()?))
    }

    impl_func_constr!(
        "a natural exponent",
        r"$y = \exp(x) or e^x$",
        add_genconstr_natural_exp,
        ffi::GRBaddgenconstrExp
    );

    impl_funca_constr!(
        "an exponent",
        r"$y = a^x$ where $a \gt 0$ is the base for the exponential function",
        add_genconstr_exp,
        ffi::GRBaddgenconstrExpA
    );

    impl_func_constr!(
        "a natural logarithm",
        r"$y = \log_e(x) or \ln(x)$",
        add_genconstr_natural_log,
        ffi::GRBaddgenconstrLog
    );

    impl_funca_constr!(
        "a logarithm",
        r"$y = \log_a(x)$ where $a \gt 0$ is the base for the logarithm function",
        add_genconstr_log,
        ffi::GRBaddgenconstrLogA
    );

    impl_func_constr!(
        "a logistic",
        r"$y=\frac{1}{1+e^{-x}}$",
        add_genconstr_logistic,
        ffi::GRBaddgenconstrLogistic
    );

    impl_funca_constr!(
        "a power",
        r"$y=x^a$, where $a$ is the (constant) exponent",
        add_genconstr_pow,
        ffi::GRBaddgenconstrPow
    );

    impl_func_constr!(
        "a sine",
        r"$y=\sin(x)$",
        add_genconstr_sin,
        ffi::GRBaddgenconstrSin
    );

    impl_func_constr!(
        "a cosine",
        r"$y=\cos(x)$",
        add_genconstr_cos,
        ffi::GRBaddgenconstrCos
    );

    impl_func_constr!(
        "a tangent",
        r"$y=\tan(x)$",
        add_genconstr_tan,
        ffi::GRBaddgenconstrTan
    );

    /// Add a range constraint to the model.
    ///
    /// This operation adds a decision variable with lower/upper bound, and a linear
    /// equality constraint which states that the value of variable must equal to `expr`.
    ///
    /// As with [`Model::add_constr`], the [`c!`](crate::c) macro is usually used to construct
    /// the second argument.
    ///
    /// # Errors
    /// - [`Error::AlgebraicError`] if the expression in the range constraint is not linear.
    /// - [`Error::ModelObjectPending`] if some variables haven't yet been added to the model.
    /// - [`Error::ModelObjectRemoved`] if some variables have been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if some variables are from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    ///
    /// # Examples
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x = add_ctsvar!(m)?;
    /// let y = add_ctsvar!(m)?;
    /// m.add_range("", c!(x - y in 0..1))?;
    /// let r = m.add_range("", c!(x*y in 0..1));
    /// assert!(matches!(r, Err(grb::Error::AlgebraicError(_))));
    /// # Ok::<(), grb::Error>(())
    /// ```
    ///
    ///
    pub fn add_range(&mut self, name: &str, expr: RangeExpr) -> Result<(Var, Constr)> {
        let constrname = CString::new(name)?;
        let (expr, lb, ub) = expr.into_normalised()?;
        let (inds, coeff) = self.get_coeffs_indices_build(&expr)?;
        self.check_apicall(unsafe {
            ffi::GRBaddrangeconstr(
                self.ptr,
                coeff.len() as ffi::c_int,
                inds.as_ptr(),
                coeff.as_ptr(),
                lb,
                ub,
                constrname.as_ptr(),
            )
        })?;

        let lazy = self.update_mode_lazy()?;
        let var = self.vars.add_new(lazy);
        let cons = self.constrs.add_new(lazy);
        Ok((var, cons))
    }

    #[allow(unused_variables)]
    /// Add multiple range constraints to the model in a single API call, analagous to
    /// [`Model::add_constrs`].
    ///
    /// # Errors
    /// - [`Error::AlgebraicError`] if the expression a the range constraint is not linear.
    /// - [`Error::ModelObjectPending`] if some variables haven't yet been added to the model.
    /// - [`Error::ModelObjectRemoved`] if some variables have been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if some variables are from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    pub fn add_ranges<'a, I, N>(&mut self, ranges_with_names: I) -> Result<(Vec<Var>, Vec<Constr>)>
    where
        N: AsRef<str> + 'a,
        I: IntoIterator<Item = (&'a N, RangeExpr)>,
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
            ffi::GRBaddrangeconstrs(
                self.ptr,
                cnames.len() as ffi::c_int,
                cbeg.len() as ffi::c_int,
                cbeg.as_ptr(),
                cind.as_ptr(),
                cval.as_ptr(),
                lbs.as_ptr(),
                ubs.as_ptr(),
                cnames.as_ptr(),
            )
        })?;

        let ncons = names.len();
        let lazy = self.update_mode_lazy()?;
        let vars = vec![self.vars.add_new(lazy); ncons];
        let cons = vec![self.constrs.add_new(lazy); ncons];
        Ok((vars, cons))
    }

    /// Add a quadratic constraint to the model.  See the [manual](https://www.gurobi.com/documentation/9.1/refman/c_addqconstr.html)
    /// for which quadratic expressions are accepted by Gurobi.
    ///
    /// # Errors
    /// - [`Error::ModelObjectPending`] if some variables haven't yet been added to the model.
    /// - [`Error::ModelObjectRemoved`] if some variables have been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if some variables are from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    pub fn add_qconstr(&mut self, name: &str, constraint: IneqExpr) -> Result<QConstr> {
        let (lhs, sense, rhs) = constraint.into_normalised_quad();
        let cname = CString::new(name)?;
        let (qrow, qcol, qval) = self.get_qcoeffs_indices_build(&lhs)?;
        let (_, lexpr) = lhs.into_parts();
        let (lvar, lval) = self.get_coeffs_indices_build(&lexpr)?;
        self.check_apicall(unsafe {
            ffi::GRBaddqconstr(
                self.ptr,
                lval.len() as ffi::c_int,
                lvar.as_ptr(),
                lval.as_ptr(),
                qval.len() as ffi::c_int,
                qrow.as_ptr(),
                qcol.as_ptr(),
                qval.as_ptr(),
                sense as ffi::c_char,
                rhs,
                cname.as_ptr(),
            )
        })?;

        Ok(self.qconstrs.add_new(self.update_mode_lazy()?))
    }

    /// Add a single [Special Order Set (SOS)](https://www.gurobi.com/documentation/9.1/refman/constraints.html#subsubsection:SOSConstraints)
    /// constraint to the model.
    ///
    /// # Errors
    /// - [`Error::ModelObjectPending`] if some variables haven't yet been added to the model.
    /// - [`Error::ModelObjectRemoved`] if some variables have been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if some variables are from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    pub fn add_sos(
        &mut self,
        var_weight_pairs: impl IntoIterator<Item = (Var, f64)>,
        sostype: SOSType,
    ) -> Result<SOS> {
        let var_weight_pairs = var_weight_pairs.into_iter();
        let n = var_weight_pairs.size_hint().0;
        let mut ind = Vec::with_capacity(n);
        let mut weight = Vec::with_capacity(n);

        for (var, w) in var_weight_pairs {
            ind.push(self.get_index_build(&var)?);
            weight.push(w);
        }

        let beg = 0;
        let sostype = sostype as c_int;
        self.check_apicall(unsafe {
            ffi::GRBaddsos(
                self.ptr,
                1,
                ind.len() as ffi::c_int,
                &sostype,
                &beg,
                ind.as_ptr(),
                weight.as_ptr(),
            )
        })?;

        Ok(self.sos.add_new(self.update_mode_lazy()?))
    }

    /// Delete a list of general constraints from an existing model.
    ///
    /// # Errors
    /// TODO: is this actually the case?
    /// - [`Error::ModelObjectPending`] if some variables haven't yet been added to the model.
    /// - [`Error::ModelObjectRemoved`] if some variables have been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if some variables are from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    pub fn del_genconstrs(&mut self, constrs: impl IntoIterator<Item = GenConstr>) -> Result<()> {
        let constr_indices: Vec<_> = constrs
            .into_iter()
            .map(|c| self.get_index_build(&c))
            .collect::<Result<_>>()?;

        self.check_apicall(unsafe {
            ffi::GRBdelgenconstrs(
                self.ptr,
                constr_indices.len() as ffi::c_int,
                constr_indices.as_ptr(),
            )
        })?;

        Ok(())
    }

    /// Set the objective function of the model and optimisation direction (min or max).
    /// Because this requires setting a [`Var`] attribute (the `Obj` attribute), this method
    /// always triggers a model update.
    ///
    /// # Errors
    /// - [`Error::ModelObjectPending`] if some variables haven't yet been added to the model.
    /// - [`Error::ModelObjectRemoved`] if some variables have been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if some variables are from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    pub fn set_objective(&mut self, expr: impl Into<Expr>, sense: ModelSense) -> Result<()> {
        self.update()?;
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

        let (coeff_map, obj_cons) = expr.into_parts();

        self.set_obj_attr_batch(
            attr::Obj,
            self.get_vars()?
                .iter()
                .map(|v| (*v, coeff_map.get(v).copied().unwrap_or(0.))),
        )?;
        self.set_attr(attr::ObjCon, obj_cons)?;
        self.set_attr(attr::ModelSense, sense)
    }

    /// Get a constraint by name.  Returns either a constraint if one was found, or `None` if none were found.
    /// If multiple constraints match, the method returns an arbitary one.
    ///
    /// # Usage
    /// ```
    /// # use grb::prelude::*;
    /// let mut m = Model::new("model")?;
    /// let x = add_binvar!(m)?;
    /// let y = add_binvar!(m)?;
    /// let c = m.add_constr("constraint", c!(x + y == 1))?;
    /// assert_eq!(m.get_constr_by_name("constraint").unwrap_err(), grb::Error::ModelUpdateNeeded);
    /// m.update()?;
    /// assert_eq!(m.get_constr_by_name("constraint")?, Some(c));
    /// assert_eq!(m.get_constr_by_name("foo")?, None);
    /// # Ok::<(), grb::Error>(())
    /// ```
    ///
    /// # Errors
    /// - [`Error::NulError`] if the `name` cannot be converted to a C-string
    /// - [`Error::ModelUpdateNeeded`] if a model update is needed.
    /// - [`Error::ModelObjectRemoved`] if the constraint has been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if the constraint is from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    pub fn get_constr_by_name(&self, name: &str) -> Result<Option<Constr>> {
        if self.constrs.model_update_needed() {
            return Err(Error::ModelUpdateNeeded);
        }
        let n = CString::new(name)?;
        let mut idx = i32::MIN;
        self.check_apicall(unsafe { ffi::GRBgetconstrbyname(self.ptr, n.as_ptr(), &mut idx) })?;
        if idx < 0 {
            Ok(None)
        } else {
            Ok(Some(self.constrs.objects()[idx as usize])) // should only panic if there's a bug in IdxManager
        }
    }

    /// Get a variable object by name.  See [`Model::get_constr_by_name`] for details
    ///
    /// # Errors
    /// - [`Error::NulError`] if the `name` cannot be converted to a C-string
    /// - [`Error::ModelUpdateNeeded`] if a model update is needed.
    /// - [`Error::ModelObjectRemoved`] if the variable has been removed from the model.
    /// - [`Error::ModelObjectMismatch`] if the variable is from a different model.
    /// - [`Error::FromAPI`] if a Gurobi API error occurs.
    pub fn get_var_by_name(&self, name: &str) -> Result<Option<Var>> {
        if self.vars.model_update_needed() {
            return Err(Error::ModelUpdateNeeded);
        }
        let n = CString::new(name)?;
        let mut idx = i32::MIN;
        self.check_apicall(unsafe { ffi::GRBgetvarbyname(self.ptr, n.as_ptr(), &mut idx) })?;
        if idx < 0 {
            Ok(None)
        } else {
            Ok(Some(self.vars.objects()[idx as usize])) // should only panic if there's a bug in IdxManager
        }
    }

    /// Query a Model attribute.    Model attributes (objects with the `ModelAttr` trait) can be found in the [`attr`] module.
    pub fn get_attr<A: ModelAttrGet<V>, V>(&self, attr: A) -> Result<V> {
        attr.get(self)
    }

    /// Query a model object attribute (Constr, Var, etc).  Available attributes can be found
    /// in the [`attr`] module, which is imported in the [prelude](crate::prelude).
    pub fn get_obj_attr<A, O, V>(&self, attr: A, obj: &O) -> Result<V>
    where
        A: ObjAttrGet<O, V>,
        O: ModelObject,
    {
        attr.get(self, self.get_index(obj)?)
    }

    /// Query an attribute of multiple model objects.   Available attributes can be found
    /// in the [`attr`] module, which is imported in the [prelude](crate::prelude).
    pub fn get_obj_attr_batch<A, I, O, V>(&self, attr: A, objs: I) -> Result<Vec<V>>
    where
        A: ObjAttrGet<O, V>,
        I: IntoIterator<Item = O>,
        O: ModelObject,
    {
        attr.get_batch(self, objs.into_iter().map(|obj| self.get_index(&obj)))
    }

    /// Set a model attribute.  Attributes (objects with the `Attr` trait) can be found in the [`attr`] module.
    ///
    /// # Example
    /// ```
    /// # use grb::prelude::*;
    /// let mut model = Model::new("")?;
    /// model.set_attr(attr::ModelName, "model".to_string())?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn set_attr<A: ModelAttrSet<V>, V>(&self, attr: A, value: V) -> Result<()> {
        attr.set(self, value)
    }

    /// Set an attribute of a Model object (Const, Var, etc).   Attributes (objects with the `Attr` trait) can be found
    /// in the [`attr`] module.
    ///
    /// # Example
    /// ```
    /// # use grb::prelude::*;
    /// let mut model = Model::new("")?;
    /// let x = add_ctsvar!(model)?;
    /// let c = model.add_constr("", c!(x <= 1))?;
    /// model.set_obj_attr(attr::VarName, &x, "x")?;
    /// model.set_obj_attr(attr::ConstrName, &c, "c")?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    /// Trying to set an attribute on a model object that belongs to another model object type will fail to compile:
    /// ```compile_fail
    /// # use grb::prelude::*;
    /// # let mut model = Model::new("")?;
    /// # let x = add_ctsvar!(model)?;
    /// model.set_obj_attr2(attr::ConstrName, &x, "c")?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn set_obj_attr<A, O, V>(&self, attr: A, obj: &O, val: V) -> Result<()>
    where
        A: ObjAttrSet<O, V>,
        O: ModelObject,
    {
        attr.set(self, self.get_index_build(obj)?, val)
    }

    /// Set an attribute of multiple Model objects (Const, Var, etc).   Attributes (objects with the `Attr` trait) can be
    /// found in the [`attr`] module.
    pub fn set_obj_attr_batch<A, O, I, V>(&self, attr: A, obj_val_pairs: I) -> Result<()>
    where
        A: ObjAttrSet<O, V>,
        I: IntoIterator<Item = (O, V)>,
        O: ModelObject,
    {
        attr.set_batch(
            self,
            obj_val_pairs
                .into_iter()
                .map(|(obj, val)| (self.get_index(&obj), val)),
        )
    }

    /// Set a model parameter.  Parameters (objects with the `Param` trait) can be found in the [`param`] module.
    ///
    /// # Example
    /// ```
    /// # use grb::prelude::*;
    /// let mut model = Model::new("")?;
    /// model.set_param(param::OutputFlag, 0)?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn set_param<P: ParamSet<V>, V>(&mut self, param: P, value: V) -> Result<()> {
        self.get_env_mut().set(param, value)
    }

    /// Query a model parameter.  Parameters (objects with the `Param` trait) can be found in the [`param`] module.
    ///
    /// # Example
    /// ```
    /// # use grb::prelude::*;
    /// let mut model = Model::new("")?;
    /// assert_eq!(model.get_param(param::LazyConstraints)?, 0);
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn get_param<P: ParamGet<V>, V>(&self, param: P) -> Result<V> {
        self.get_env().get(param)
    }

    /// Modify the model to create a feasibility relaxation.
    ///
    /// Given a `Model` whose objective function is $f(x)$, the feasibility relaxation seeks to minimise
    /// $$
    ///   \text{min}\quad f(x) + \sum_{j} w_j \cdot p(s_j)
    /// $$
    /// where $s\_j > 0$ is the slack variable of $j$ -th constraint or bound, $w_j$ is the $j$-th weight
    /// and $p(s)$ is the penalty function.
    ///
    /// The `ty` argument sets the penalty function:
    ///
    /// | [`RelaxType`] variant | Penalty function                                                                    |
    /// | --------------------- | ----------------------------------------------------------------------------------- |
    /// | `Quadratic`           | $ p(s) = {s}^2 $                                                                    |
    /// | `Linear`              | $ p(s) = {s} $                                                                      |
    /// | `Cardinality`         | $ p(s) = \begin{cases} 1 & \text{if } s > 0 \\\\ 0 & \text{otherwise} \end{cases} $ |
    ///
    /// This method will modify the model - if this is not desired copy the model before invoking
    /// this method with [`Model::try_clone()`].
    ///
    /// ## Arguments
    /// * `ty` : The type of cost function used when finding the minimum cost relaxation.
    /// * `minrelax` : How the objective should be minimised.
    ///
    ///   If `false`, optimizing the returned model gives a solution that minimizes the cost of the
    ///   violation. If `true`, optimizing the returned model finds a solution that minimizes the original objective,
    ///   but only from among those solutions that minimize the cost of the violation. Note that this method must solve an
    ///   optimization problem to find the minimum possible relaxation when set to `true`, which can be quite expensive.
    ///
    /// * `lb_pen` : Variables whose lower bounds are allowed to be violated, and their penalty weights.
    /// * `ub_pen` : Variables whose upper bounds are allowed to be violated, and their penalty weights.
    /// * `constr_pen` :  Constraints which are allowed to be violated, and their penalty weights.
    ///
    /// ## Returns
    /// * The objective value for the relaxation performed (if `minrelax` is `true`).
    /// * Slack variables for relaxation and related linear/general/quadratic constraints.
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub fn feas_relax(
        &mut self,
        ty: RelaxType,
        minrelax: bool,
        lb_pen: impl IntoIterator<Item = (Var, f64)>,
        ub_pen: impl IntoIterator<Item = (Var, f64)>,
        constr_pen: impl IntoIterator<Item = (Constr, f64)>,
    ) -> Result<(Option<f64>, Vec<Var>, Vec<Constr>, Vec<QConstr>)> {
        self.update()?;
        let n_old_vars = self.get_attr(attr::NumVars)? as usize;
        let n_old_constr = self.get_attr(attr::NumConstrs)? as usize;
        let n_old_genconstr = self.get_attr(attr::NumGenConstrs)? as usize;
        let n_old_qconstr = self.get_attr(attr::NumQConstrs)? as usize;

        fn build_array<T, O>(
            model: &Model,
            iter: T,
            buf_size: usize,
        ) -> Result<(Option<Vec<f64>>, *const f64)>
        where
            T: IntoIterator<Item = (O, f64)>,
            O: ModelObject,
        {
            let mut iter = iter.into_iter().peekable();
            if iter.peek().is_some() {
                let mut vec = vec![super::INFINITY; buf_size];
                for (obj, weight) in iter {
                    vec[model.get_index(&obj)? as usize] = weight;
                }
                let ptr = vec.as_ptr();
                Ok((Some(vec), ptr))
            } else {
                Ok((None, std::ptr::null()))
            }
        }

        let (_lbpen, lbpen) = build_array(self, lb_pen, n_old_vars)?;
        let (_ubpen, ubpen) = build_array(self, ub_pen, n_old_vars)?;
        let (_rhspen, rhspen) = build_array(self, constr_pen, n_old_constr)?;

        let mut feasobj = crate::constants::GRB_UNDEFINED;
        self.check_apicall(unsafe {
            ffi::GRBfeasrelax(
                self.ptr,
                ty as c_int,
                minrelax as c_int,
                lbpen,
                ubpen,
                rhspen,
                &mut feasobj,
            )
        })?;
        let feasobj = if minrelax { Some(feasobj) } else { None };
        self.update()?;

        let lazy = self.update_mode_lazy()?;

        let n_vars = self.get_attr(attr::NumVars)? as usize;
        assert!(n_vars >= n_old_vars);
        let new_vars = (0..n_vars - n_old_vars)
            .map(|_| self.vars.add_new(lazy))
            .collect();

        let n_cons = self.get_attr(attr::NumConstrs)? as usize;
        assert!(n_cons >= n_old_constr);
        let new_cons = (0..n_cons - n_old_constr)
            .map(|_| self.constrs.add_new(lazy))
            .collect();

        let n_gencons = self.get_attr(attr::NumGenConstrs)? as usize;
        assert_eq!(n_gencons, n_old_genconstr);

        let n_qcons = self.get_attr(attr::NumQConstrs)? as usize;
        assert!(n_qcons >= n_old_qconstr);
        let new_qcons = (0..n_cons - n_old_constr)
            .map(|_| self.qconstrs.add_new(lazy))
            .collect();

        Ok((feasobj, new_vars, new_cons, new_qcons))
    }

    /// Capture a single scenario from a multi-scenario model. Use the `ScenarioNumber` parameter to indicate which
    /// scenario to capture. See the
    /// [manual](https://www.gurobi.com/documentation/9.5/refman/multiple_scenarios.html#sec:MultipleScenarios)
    /// for details on multi-scenario models.
    pub fn single_scenario_model(&mut self) -> Result<Model> {
        let mut model_ptr: *mut GRBmodel = std::ptr::null_mut();
        self.check_apicall(unsafe {
            ffi::GRBsinglescenariomodel(self.as_mut_ptr(), &mut model_ptr)
        })?;
        assert!(!model_ptr.is_null());
        Model::from_raw(self.get_env(), model_ptr)
    }

    /// Set a piecewise-linear objective function for the variable.
    ///
    /// Given a sequence of points $(x_1, y_1), \dots, (x_n, y_n)$, the piecewise-linear objective function
    /// $f(x)$ is defined as follows:
    /// $$
    ///   f(x) =
    ///   \begin{cases}
    ///     y_1 + \dfrac{y_2 - y_1}{x_2 - x_1} \\, (x - x_1)         & \text{if $x \leq x_1$}, \\\\
    ///   \\\\
    ///     y_i + \dfrac{y_{i+1} - y_i}{x_{i+1}-x_i} \\, (x - x_i)   & \text{if $x_i \leq x \leq x_{i+1}$}, \\\\
    ///   \\\\
    ///     y_n + \dfrac{y_n - y_{n-1}}{x_n-x_{n-1}} \\, (x - x_n)   & \text{if $x \geq x_n$},
    ///   \end{cases}
    /// $$
    ///
    /// The `Obj` attribute of the [`Var`] object will be set to 0.   To delete the piecewise-linear function on the
    /// variable, set the value of `Obj` attribute to non-zero.
    ///
    /// The `points` argument contains the pairs $(x_i,y_i)$ and must satisfy $x_i < x_{i+1}$.
    pub fn set_pwl_obj(
        &mut self,
        var: &Var,
        points: impl IntoIterator<Item = (f64, f64)>,
    ) -> Result<()> {
        let points = points.into_iter();
        let n = points.size_hint().0;
        let mut xvals = Vec::with_capacity(n);
        let mut yvals = Vec::with_capacity(n);
        for (x, y) in points {
            xvals.push(x);
            yvals.push(y);
        }

        self.check_apicall(unsafe {
            ffi::GRBsetpwlobj(
                self.ptr,
                self.get_index_build(var)?,
                xvals.len() as ffi::c_int,
                xvals.as_ptr(),
                yvals.as_ptr(),
            )
        })
    }

    /// Retrieve the status of the model.
    pub fn status(&self) -> Result<Status> {
        self.get_attr(attr::Status)
    }

    impl_object_list_getter!(get_vars, Var, vars, "variables");

    impl_object_list_getter!(get_constrs, Constr, constrs, "constraints");

    impl_object_list_getter!(get_genconstrs, GenConstr, genconstrs, "general constraints");

    impl_object_list_getter!(get_qconstrs, QConstr, qconstrs, "quadratic constraints");

    impl_object_list_getter!(get_sos, SOS, sos, "SOS constraints");

    /// Remove a variable or constraint from the model.
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
            ffi::GRBgetcoeff(
                self.ptr,
                self.get_index_build(constr)?,
                self.get_index_build(var)?,
                &mut value,
            )
        })?;
        Ok(value)
    }

    /// Change a single constant matrix coefficient of the model.
    pub fn set_coeff(&mut self, var: &Var, constr: &Constr, value: f64) -> Result<()> {
        self.check_apicall(unsafe {
            ffi::GRBchgcoeffs(
                self.ptr,
                1,
                &self.get_index_build(constr)?,
                &self.get_index_build(var)?,
                &value,
            )
        })
    }

    /// Change a set of constant matrix coefficients of the model.
    pub fn set_coeffs(
        &mut self,
        coeffs: impl IntoIterator<Item = (Var, Constr, f64)>,
    ) -> Result<()> {
        let (vind, cind, val) = self.build_idx_arrays_obj_obj(coeffs.into_iter())?;
        self.check_apicall(unsafe {
            ffi::GRBchgcoeffs(
                self.as_mut_ptr(),
                vind.len() as ffi::c_int,
                cind.as_ptr(),
                vind.as_ptr(),
                val.as_ptr(),
            )
        })
    }

    // add quadratic terms of objective function.
    fn add_qpterms(&mut self, qrow: &[i32], qcol: &[i32], qval: &[f64]) -> Result<()> {
        self.check_apicall(unsafe {
            ffi::GRBaddqpterms(
                self.ptr,
                qrow.len() as ffi::c_int,
                qrow.as_ptr(),
                qcol.as_ptr(),
                qval.as_ptr(),
            )
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

/// A handle to an [`AsyncModel`] which is currently solving.
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

    /// Send a request to Gurobi to terminate optimization.  Optimization may not finish immediately.
    ///
    /// # Example
    /// ```
    /// # use grb::prelude::*;
    /// use grb::AsyncModel;
    /// let e = Env::new("")?;
    /// let m = Model::with_env("async", e)?;
    /// # /*
    ///   ...
    /// # */
    /// let m = AsyncModel::new(m);
    /// // discard `AsyncModel` on failure and panic
    /// let handle = m.optimize().map_err(|(_, e)| e).unwrap();
    /// # /*
    ///   ...
    /// # */
    /// handle.terminate();
    /// # let (m, errors) = handle.join();
    /// # errors?;
    /// # Ok::<(), grb::Error>(())
    /// ```
    pub fn terminate(&self) {
        self.0.terminate();
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
    /// You can also pass an owned `Env` to `Model::with_env`:
    /// ```
    /// # use grb::prelude::*;
    /// # use grb::AsyncModel;
    /// # let env = Env::new("")?;
    /// let mut m = Model::with_env("model", env)?;
    /// let mut m =  AsyncModel::new(m); // also ok
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
        assert!(
            !model.env.is_shared(),
            "Cannot create async model - environment is used in other models"
        );
        AsyncModel(model)
    }

    /// Optimize the model on another thread.  This method will always trigger a [`Model::update`] on the underlying `Model`.
    ///
    /// On success, returns an [`AsyncHandle`] that provides a limited API for model queries.
    /// The `AsyncModel` can be retrieved by calling [`AsyncHandle::join`](crate::model::AsyncHandle::join).
    ///
    /// # Errors
    /// An `grb::Error::FromAPI` may occur.  In this case, the `Err` variant contains this error
    /// and gives back ownership of this `AsyncModel`.
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
        match self.0.update().and_then(|()| {
            self.0
                .check_apicall(unsafe { ffi::GRBoptimizeasync(self.0.ptr) })
        }) {
            Ok(()) => Ok(AsyncHandle(self.0)),
            Err(e) => Err((self, e)),
        }
    }
}

// TODO: check that multi-objective and scenario optimisation work/are usable

impl std::convert::From<AsyncModel> for Model {
    fn from(model: AsyncModel) -> Model {
        model.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        assert_eq!(
            model.get_obj_attr(attr::VarName, &x).unwrap(),
            "x".to_string()
        );
        assert_eq!(
            model.get_obj_attr(attr::VarName, &z).unwrap(),
            "z".to_string()
        );
        assert_eq!(
            model.get_obj_attr(attr::VarName, &w).unwrap(),
            "w".to_string()
        );
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
        assert_eq!(
            model.get_index_build(&x).unwrap_err(),
            Error::ModelObjectPending
        );
        assert_eq!(
            model.get_index_build(&y).unwrap_err(),
            Error::ModelObjectPending
        );
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
        assert_eq!(
            model.get_index_build(&z).unwrap_err(),
            Error::ModelObjectPending
        );
        assert_eq!(
            model.get_index_build(&w).unwrap_err(),
            Error::ModelObjectPending
        );

        model.update().unwrap();
        assert_eq!(model.get_attr(attr::NumVars).unwrap(), 3);
        assert_eq!(model.get_index(&x).unwrap(), 0);
        assert_eq!(model.get_index(&y).unwrap_err(), Error::ModelObjectRemoved);
        assert_eq!(model.get_index(&z).unwrap(), 1);
        assert_eq!(model.get_index(&w).unwrap(), 2);
        assert_eq!(model.get_index(&c1).unwrap(), 0);

        assert_eq!(
            model.get_obj_attr(attr::VarName, &x).unwrap(),
            "x".to_string()
        );
        assert_eq!(
            model.get_obj_attr(attr::VarName, &z).unwrap(),
            "z".to_string()
        );
        assert_eq!(
            model.get_obj_attr(attr::VarName, &w).unwrap(),
            "w".to_string()
        );
    }

    #[test]
    fn model_obj_size() {
        assert_eq!(std::mem::size_of::<Var>(), 8);
        assert_eq!(std::mem::size_of::<QConstr>(), 8);
        assert_eq!(std::mem::size_of::<Constr>(), 8);
        assert_eq!(std::mem::size_of::<GenConstr>(), 8);
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
        assert_eq!(
            model1.add_constr("", c!(x2 <= y1)).unwrap_err(),
            Error::ModelObjectMismatch
        );
        assert_eq!(
            model1.add_constr("", c!(x1 <= y2)).unwrap_err(),
            Error::ModelObjectMismatch
        );

        model1.update().unwrap();
        model2.update().unwrap();

        assert_eq!(
            model1.get_obj_attr(attr::VarName, &x1).unwrap(),
            "x1".to_string()
        );
        assert_eq!(
            model1.get_obj_attr(attr::VarName, &y1).unwrap(),
            "y1".to_string()
        );
        assert_eq!(
            model2.get_obj_attr(attr::VarName, &x2).unwrap(),
            "x2".to_string()
        );
        assert_eq!(
            model2.get_obj_attr(attr::VarName, &y2).unwrap(),
            "y2".to_string()
        );

        assert_eq!(
            model1.get_obj_attr(attr::VarName, &y2).unwrap_err(),
            Error::ModelObjectMismatch
        );
        assert_eq!(
            model2.get_obj_attr(attr::VarName, &x1).unwrap_err(),
            Error::ModelObjectMismatch
        );
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
        let m2 = Model::from_file_with_env(filename, &env)?;
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

    #[test]
    fn objective_constant() -> Result<()> {
        let mut m = Model::new("")?;
        let x = add_ctsvar!(m)?;
        m.set_objective(x + 1, Minimize)?; // x has ub of 0
        m.optimize()?;
        assert_eq!(m.get_attr(attr::ObjVal)?.round() as usize, 1); // obj = x^* + 1 = 0 + 1
        Ok(())
    }
}
