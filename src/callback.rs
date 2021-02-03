use gurobi_sys as ffi;
use std::ptr::null;
use std::os::raw;
use std::iter::{Iterator, IntoIterator};
use std::borrow::Borrow;

use crate::{Error, Result, Model, Var, INFINITY, Status};
use crate::util;
use crate::constants::{callback::*, GRB_UNDEFINED, ERROR_CALLBACK};
use crate::constr::IneqExpr; // used for setting a partial solution in a callback

pub type CbResult = anyhow::Result<()>;

pub trait Callback {
  fn callback(&mut self, w: Where) -> CbResult;
}

impl<F: FnMut(Where) -> CbResult> Callback for F {
  fn callback(&mut self, w: Where) -> CbResult {
    self(w)
  }
}

/// The C function given to the Gurobi API with `GRBsetcallbackfunc`
#[allow(unused_variables)]
pub(crate) extern "C" fn callback_wrapper(model: *mut ffi::GRBmodel,
                                          cbdata: *mut ffi::c_void,
                                          where_: ffi::c_int,
                                          usrdata: *mut ffi::c_void) -> ffi::c_int {
  use std::panic::{catch_unwind, AssertUnwindSafe};
  let usrdata = unsafe { &mut *(usrdata as *mut UserCallbackData) };
  let (cb_obj, model, nvars) = (&mut usrdata.cb_obj, usrdata.model, usrdata.nvars);

  let where_ = Where::new(CbCtx::new(cbdata, where_, model, nvars));

  let callback_result = catch_unwind(AssertUnwindSafe(|| {
    let w = match where_ {
      Ok(w) => w,
      Err(e @ Error::NotYetSupported(_)) => { return Ok(()) }
      Err(_) => unreachable!(),
    };
    cb_obj.callback(w)
  }));

  match callback_result {
    Ok(Ok(())) => 0,
    Ok(Err(e)) => {
      eprintln!("Callback returned error:\n{:#?}", e);
      ERROR_CALLBACK
    }
    Err(_) => {
      eprintln!("Callback panicked! You should return an error instead.");
      ERROR_CALLBACK
    }
  }
}


/// The `usrdata` struct passed to [`callback_wrapper`]
pub(crate) struct UserCallbackData<'a> {
  pub(crate) model: &'a Model,
  pub(crate) nvars: usize,
  pub(crate) cb_obj: &'a mut dyn Callback,
}


// TODO add MULTIOBJ

macro_rules! impl_getter {
    ($name:ident, i32, $wher:path, $what:path, $help:literal) => {
      #[doc = $help]
      pub fn $name(&self) -> Result<i32> {
        self.0.get_int($wher, $what)
      }
    };

    ($name:ident, f64, $wher:path, $what:path, $help:literal) => {
      #[doc = $help]
      pub fn $name(&self) -> Result<f64> {
        self.0.get_double($wher, $what)
      }
    };
}

macro_rules! impl_runtime {
    () => {
      /// Retrieve the elapsed solver runtime in seconds.
      pub fn runtime(&self) -> Result<f64> {
        self.0.get_runtime()
      }
    };
}

macro_rules! impl_terminate {
    () => {
      /// Signal Gurobi to terminate the optimisation.  Will not take effect immediately
      pub fn terminate(&self) {
        self.0.terminate()
      }
    };
}

macro_rules! impl_add_lazy {
    () => {
        /// Add a new lazy constraint to the model
        ///
        /// *Important*: Requires that the `LazyConstraints` parameter is set to 1
        pub fn add_lazy(&self, constr: IneqExpr) -> Result<()> {
          self.0.add_lazy(constr)
        }
    };
}


pub struct PollingCtx<'a>(CbCtx<'a>);
impl<'a> PollingCtx<'a> {
  impl_terminate! {}
}

pub struct PreSolveCtx<'a>(CbCtx<'a>);
impl<'a> PreSolveCtx<'a> {
  impl_terminate! {}
  impl_runtime! {}
  impl_getter! { col_del, i32, PRESOLVE, PRE_COLDEL, "Number of columns removed so far." }
  impl_getter! { row_del, i32, PRESOLVE, PRE_ROWDEL, "Number of rows removed so far." }
  impl_getter! { sense_chg, i32, PRESOLVE, PRE_SENCHG, "Number of constraint senses changed so far." }
  impl_getter! { bnd_chg, i32, PRESOLVE, PRE_BNDCHG, "Number of variable bounds changed so far." }
  impl_getter! { coeff_chg, i32, PRESOLVE, PRE_COECHG, "Number of coefficients changed so far." }
}

pub struct SimplexCtx<'a>(CbCtx<'a>);
impl<'a> SimplexCtx<'a> {
  impl_terminate! {}
  impl_runtime! {}
  impl_getter! { iter_cnt, f64, SIMPLEX, SPX_ITRCNT, "Current simplex iteration count." }
  impl_getter! { obj_val, f64, SIMPLEX, SPX_OBJVAL, "Current simplex objective value." }
  impl_getter! { prim_inf, f64, SIMPLEX, SPX_PRIMINF, "Current primal infeasibility." }
  impl_getter! { dual_inf, f64, SIMPLEX, SPX_DUALINF, "Current primal infeasibility." }
  impl_getter! { is_perturbed, i32, SIMPLEX, SPX_ISPERT, "Is problem currently perturbed?" }
}

pub struct MIPCtx<'a>(CbCtx<'a>);
impl<'a> MIPCtx<'a> {
  impl_terminate! {}
  impl_runtime! {}
  impl_getter! { obj_best, f64, MIP, MIP_OBJBST, "Current best objective." }
  impl_getter! { obj_bnd, f64, MIP, MIP_OBJBND, "Current best objective bound." }
  impl_getter! { node_cnt, f64, MIP, MIP_NODCNT, "Current explored node count." }
  impl_getter! { sol_cnt, i32, MIP, MIP_SOLCNT, "Current count of feasible solutions found." }
  impl_getter! { cut_cnt, i32, MIP, MIP_CUTCNT, "Current count of cutting planes applied." }
  impl_getter! { node_left, f64, MIP, MIP_NODLFT, "Current unexplored node count." }
  impl_getter! { iter_cnt, f64, MIP, MIP_ITRCNT, "Current simplex iteration count." }
}

pub struct MIPSolCtx<'a>(CbCtx<'a>);
impl<'a> MIPSolCtx<'a> {
  /// Add a new (linear) cutting plane to the MIP model.
  pub fn add_cut(&self, constr: IneqExpr) -> Result<()> { self.0.add_cut(constr) }

  pub fn get_solution<I, V>(&self, vars: I) -> Result<Vec<f64>> where
    V: Borrow<Var>,
    I: IntoIterator<Item=V>
  {
    self.0.get_mip_solution(vars)
  }

  impl_terminate! {}
  impl_runtime! {}
  impl_add_lazy! {}
  impl_getter! { obj, f64, MIPSOL, MIPSOL_OBJ, "Objective value for the new solution." }
  impl_getter! { obj_best, f64, MIPSOL, MIPSOL_OBJBST, "Current best objective." }
  impl_getter! { obj_bnd, f64, MIPSOL, MIPSOL_OBJBND, "Current best objective bound." }
  impl_getter! { node_cnt, f64, MIPSOL, MIPSOL_NODCNT, "Current explored node count." }
  impl_getter! { sol_cnt, i32, MIPSOL, MIPSOL_SOLCNT, "Current count of feasible solutions found." }
}

pub struct MIPNodeCtx<'a>(CbCtx<'a>);
impl<'a> MIPNodeCtx<'a> {
  // Optimization status of current MIP node
  pub fn status(&self) -> Result<Status> {
    self.0.get_int(MIPNODE, MIPNODE_STATUS).map(Status::from)
  }

  pub fn get_solution<I,V>(&self, vars: I) -> Result<Vec<f64>> where
    V: Borrow<Var>,
    I: IntoIterator<Item=V>
  {
    self.0.get_node_rel(vars)
  }

  /// Provide a new feasible solution for a MIP model.  Not all variables need to be given.
  ///
  /// On success, if the solution was feasible the method returns the computed objective value,
  /// otherwise returns `None`.
  pub fn set_solution<I, V, T>(&self, solution: I) -> Result<Option<f64>> where
    V: Borrow<Var>,
    T: Borrow<f64>,
    I: IntoIterator<Item=(V, T)> {
    self.0.set_solution(solution)
  }

  impl_terminate! {}
  impl_runtime! {}
  impl_add_lazy! {}
  impl_getter! { obj_best, f64, MIPNODE, MIPNODE_OBJBST, "Current best objective." }
  impl_getter! { obj_bnd, f64, MIPNODE, MIPNODE_OBJBND, "Current best objective bound." }
  impl_getter! { node_cnt, f64, MIPNODE, MIPNODE_NODCNT, "Current explored node count." }
  impl_getter! { sol_cnt, i32, MIPNODE, MIPNODE_SOLCNT, "Current count of feasible solutions found." }
}

pub struct MessageCtx<'a>(CbCtx<'a>);
impl<'a> MessageCtx<'a> {
  /// The message about to be logged
  pub fn message(&self) -> Result<String> {
    self.0.get_string(MESSAGE, MSG_STRING).map(|s| s.trim().to_owned())
  }

  impl_terminate! {}
}

pub struct BarrierCtx<'a>(CbCtx<'a>);

impl<'a> BarrierCtx<'a> {
  impl_terminate! {}
  impl_runtime! {}
  impl_getter! { iter_cnt, i32, BARRIER, BARRIER_ITRCNT, "Current simplex iteration count." }
  impl_getter! { prim_obj, f64, BARRIER, BARRIER_PRIMOBJ, "Primal objective value for current barrier iterate." }
  impl_getter! { dual_obj, f64, BARRIER, BARRIER_DUALOBJ, "Dual objective value for current barrier iterate." }
  impl_getter! { prim_inf, f64, BARRIER, BARRIER_PRIMINF, "Primal infeasibility for current barrier iterate." }
  impl_getter! { dual_inf, f64, BARRIER, BARRIER_DUALINF, "Dual infeasibility for current barrier iterate." }
  impl_getter! { compl_viol, f64, BARRIER, BARRIER_COMPL, "Complementarity violation for current barrier iterate." }
}


pub enum Where<'a> {
  Polling(PollingCtx<'a>),
  PreSolve(PreSolveCtx<'a>),
  Simplex(SimplexCtx<'a>),
  MIP(MIPCtx<'a>),
  MIPSol(MIPSolCtx<'a>),
  MIPNode(MIPNodeCtx<'a>),
  Message(MessageCtx<'a>),
  Barrier(BarrierCtx<'a>),
}

impl Where<'_> {
  fn new<'a>(ctx: CbCtx<'a>) -> Result<Where<'a>> {
    let w = match ctx.where_raw {
      POLLING => Where::Polling(PollingCtx(ctx)),
      PRESOLVE => Where::PreSolve(PreSolveCtx(ctx)),
      SIMPLEX => Where::Simplex(SimplexCtx(ctx)),
      MIP => Where::MIP(MIPCtx(ctx)),
      MIPNODE => Where::MIPNode(MIPNodeCtx(ctx)),
      MIPSOL => Where::MIPSol(MIPSolCtx(ctx)),
      MESSAGE => Where::Message(MessageCtx(ctx)),
      BARRIER => Where::Barrier(BarrierCtx(ctx)),
      _ => {
        return Err(Error::NotYetSupported(format!("WHERE = {}", ctx.where_raw)));
      }
    };
    Ok(w)
  }
}

/// The context object for Gurobi callback.
struct CbCtx<'a> {
  where_raw: i32,
  cbdata: *mut ffi::c_void,
  model: &'a Model,
  nvars: usize,
}


impl<'a> CbCtx<'a> {
  pub(crate) fn new(cbdata: *mut ffi::c_void, where_raw: i32, model: &'a Model, nvars: usize) -> Self {
    CbCtx {
      cbdata,
      where_raw,
      model,
      nvars,
    }
  }

  /// Retreive node relaxation solution values at the current node.
  pub fn get_node_rel<I, V>(&self, vars: I) -> Result<Vec<f64>> where
    V: Borrow<Var>,
    I: IntoIterator<Item=V>
  {
    // memo: only MIPNode && status == Optimal
    // note that this MUST be after a call to model.update(), so the indices in model.vars are Added and the unwrap() is ok
    let vals = self.get_double_array_vars(MIPNODE, MIPNODE_REL)?;
    vars.into_iter().map(|v| Ok(vals[self.model.get_index(v.borrow())? as usize])).collect()
  }

  /// Retrieve values from the current solution vector.
  pub fn get_mip_solution<I, V>(&self, vars: I) -> Result<Vec<f64>> where
    V: Borrow<Var>,
    I: IntoIterator<Item=V>
  {
    let vals = self.get_double_array_vars(MIPSOL, MIPSOL_SOL)?;
    vars.into_iter().map(|v| Ok(vals[self.model.get_index(v.borrow())? as usize])).collect()
  }

  /// Provide a new feasible solution for a MIP model.  Not all variables need to be given.
  pub fn set_solution<I, V, T>(&self, solution: I) -> Result<Option<f64>> where
    V: Borrow<Var>,
    T: Borrow<f64>,
    I: IntoIterator<Item=(V, T)>
  {
    let mut soln = vec![GRB_UNDEFINED; self.model.get_attr(crate::attr::NumVars)? as usize];
    for (i, val) in solution {
      soln[self.model.get_index_build(i.borrow())? as usize] = *val.borrow();
    }
    let mut obj = INFINITY as raw::c_double;
    self.check_apicall(unsafe { ffi::GRBcbsolution(self.cbdata, soln.as_ptr(), &mut obj as *mut raw::c_double) })?;

    Ok(if obj == INFINITY { None } else { Some(obj) })
  }

  /// Retrieve the elapsed solver runtime in seconds.
  pub fn get_runtime(&self) -> Result<f64> {
    self.get_double(self.where_raw, RUNTIME)
  }

  /// Add a new cutting plane to the MIP model.
  pub fn add_cut(&self, constr: IneqExpr) -> Result<()> {
    // note the user can still provide a LinExpr containing vars from a different model, so unwrap() won't work
    let (lhs, sense, rhs) = constr.into_normalised_linear()?;
    let (inds, coeff) = self.model.get_coeffs_indices_build(&lhs)?;

    self.check_apicall(unsafe {
      ffi::GRBcbcut(self.cbdata,
                    coeff.len() as ffi::c_int,
                    inds.as_ptr(),
                    coeff.as_ptr(),
                    sense.into(),
                    rhs)
    })
  }

  /// Add a new lazy constraint to the MIP model.
  pub fn add_lazy(&self, constr: IneqExpr) -> Result<()> {
    let (lhs, sense, rhs) = constr.into_normalised_linear()?;
    let (inds, coeff) = self.model.get_coeffs_indices_build(&lhs)?;
    self.check_apicall(unsafe {
      ffi::GRBcblazy(self.cbdata,
                     coeff.len() as ffi::c_int,
                     inds.as_ptr(),
                     coeff.as_ptr(),
                     sense.into(),
                     rhs)
    })
  }

  pub fn terminate(&self) {
    self.model.terminate()
  }

  fn get_int(&self, where_: i32, what: i32) -> Result<i32> {
    let mut buf = 0i32;
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, &mut buf as *mut i32 as *mut raw::c_void) }).and(Ok(buf))
  }

  fn get_double(&self, where_: i32, what: i32) -> Result<f64> {
    let mut buf = 0.0f64;
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, &mut buf as *mut f64 as *mut raw::c_void) }).and(Ok(buf))
  }

  fn get_double_array_vars(&self, where_: i32, what: i32) -> Result<Vec<f64>> {
    let mut buf = vec![0.0; self.nvars];
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, buf.as_mut_ptr() as *mut raw::c_void) }).and(Ok(buf))
  }

  fn get_string(&self, where_: i32, what: i32) -> Result<String> {
    let mut buf = null();
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, &mut buf as *mut *const i8 as *mut raw::c_void) })
      .and(Ok(unsafe { util::copy_c_str(buf) }))
  }

  fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(())
  }
}
