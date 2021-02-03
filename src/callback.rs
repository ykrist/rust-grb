use gurobi_sys as ffi;
use std::ptr::null;
use std::os::raw;
use std::convert::TryFrom;

use crate::{Error, Result, Model, Var, ConstrSense, INFINITY, Status};
use crate::expr::LinExpr;
use crate::util;
use crate::constants::callback::*;

use crate::constants::GRB_UNDEFINED;
use crate::callback::WhereData::MIPNode; // used for setting a partial solution in a callback

pub trait Callback {
  fn callback(&mut self, ctx: CbCtx, w: Where) -> Result<()>;
}

impl<F: FnMut(CbCtx) -> Result<()>> Callback for F {
  fn callback(&mut self, ctx: CbCtx, w: Where) -> Result<()> {
    self(ctx)
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

  #[allow(clippy::useless_conversion)]
  match CbCtx::new(cbdata, where_.into(), model, nvars) {
    Err(err) => {
      println!("failed to create context: {:?}", err);
      -3
    }
    Ok(context) => {
      match catch_unwind(AssertUnwindSafe(|| if cb_obj.callback(context).is_ok() { 0 } else { -1 })) {
        Ok(ret) => ret,
        Err(_e) => -3000,
      }
    }
  }
}


/// The `usrdata` struct passed to [`callback_wrapper`]
pub(crate) struct UserCallbackData<'a> {
  pub(crate) model: &'a Model,
  pub(crate) nvars: usize,
  pub(crate) cb_obj: &'a mut dyn Callback,
}


/// Location where the callback was called.
///
/// If you want to get more information, see the [official
/// manual](https://www.gurobi.com/documentation/6.5/refman/callback_codes.html).
#[derive(Debug, Clone)]
pub enum WhereData {
  /// Periodic polling callback
  Polling,
  /// Currently performing presolve
  PreSolve {
    /// The number of columns removed by presolve to this point.
    coldel: i32,
    /// The number of rows removed by presolve to this point.
    rowdel: i32,
    /// The number of constraint senses changed by presolve to this point.
    senchg: i32,
    /// The number of variable bounds changed by presolve to this point.
    bndchg: i32,
    /// The number of coefficients changed by presolve to this point.
    coecfg: i32,
  },
  /// Currently in simplex
  Simplex {
    /// Current simplex iteration count.
    itrcnt: f64,
    /// Current simplex objective value.
    objval: f64,
    /// Current primal infeasibility.
    priminf: f64,
    /// Current dual infeasibility.
    dualinf: f64,
    /// Is problem current perturbed?
    ispert: i32,
  },
  /// Currently in MIP
  MIP {
    /// Current best objective.
    objbst: f64,
    /// Current best objective bound.
    objbnd: f64,
    /// Current explored node count.
    nodcnt: f64,
    /// Current count of feasible solutions found.
    solcnt: f64,
    /// Current count of cutting planes applied.
    cutcnt: i32,
    /// Current unexplored node count.
    nodleft: f64,
    /// Current simplex iteration count.
    itrcnt: f64,
  },
  /// Found a new MIP incumbent
  MIPSol {
    /// Objective value for new solution.
    obj: f64,
    /// Current best objective.
    objbst: f64,
    /// Current best objective bound.
    objbnd: f64,
    /// Current explored node count.
    nodcnt: f64,
    /// Current count of feasible solutions found.
    solcnt: f64,
  },
  /// Currently exploring a MIP node
  MIPNode {
    /// Optimization status of current MIP node (see the Status Code section for further information).
    status: i32,
    /// Current best objective.
    objbst: f64,
    /// Current best objective bound.
    objbnd: f64,
    /// Current explored node count.
    nodcnt: f64,
    /// Current count of feasible solutions found.
    solcnt: i32,
  },
  /// Printing a log message
  Message(String),
  /// Currently in barrier.
  Barrier {
    /// Current barrier iteration count.
    itrcnt: i32,
    /// Primal objective value for current barrier iterate.
    primobj: f64,
    /// Dual objective value for current barrier iterate.
    dualobj: f64,
    /// Primal infeasibility for current barrier iterate.
    priminf: f64,
    /// Dual infeasibility for current barrier iterate.
    dualinf: f64,
    /// Complementarity violation for current barrier iterate.
    compl: f64,
  },
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
struct PreSolveData<'a>(&'a CbCtx<'a>);

impl<'a> PreSolveData<'a> {
  impl_getter! { col_del, i32, PRESOLVE, PRE_COLDEL, "Number of columns removed so far." }
  impl_getter! { row_del, i32, PRESOLVE, PRE_ROWDEL, "Number of rows removed so far." }
  impl_getter! { sense_chg, i32, PRESOLVE, PRE_SENCHG, "Number of constraint senses changed so far." }
  impl_getter! { bnd_chg, i32, PRESOLVE, PRE_BNDCHG, "Number of variable bounds changed so far." }
  impl_getter! { coeff_chg, i32, PRESOLVE, PRE_COECHG, "Number of coefficients changed so far." }
}

struct SimplexData<'a>(&'a CbCtx<'a>);

impl<'a> SimplexData<'a> {
  impl_getter! { iter_cnt, f64, SIMPLEX, SPX_ITRCNT, "Current simplex iteration count." }
  impl_getter! { obj_val, f64, SIMPLEX, SPX_OBJVAL, "Current simplex objective value." }
  impl_getter! { prim_inf, f64, SIMPLEX, SPX_PRIMINF, "Current primal infeasibility." }
  impl_getter! { dual_inf, f64, SIMPLEX, SPX_DUALINF, "Current primal infeasibility." }
  impl_getter! { is_perturbed, i32, SIMPLEX, SPX_ISPERT, "Is problem currently perturbed?" }
}

struct MIPData<'a>(&'a CbCtx<'a>);

impl<'a> MIPData<'a> {
  impl_getter! { obj_best, f64, MIP, MIP_OBJBST, "Current best objective." }
  impl_getter! { obj_bnd, f64, MIP, MIP_OBJBND, "Current best objective bound." }
  impl_getter! { node_cnt, f64, MIP, MIP_NODCNT, "Current explored node count." }
  impl_getter! { sol_cnt, i32, MIP, MIP_SOLCNT, "Current count of feasible solutions found." }
  impl_getter! { cut_cnt, i32, MIP, MIP_CUTCNT, "Current count of cutting planes applied." }
  impl_getter! { node_left, f64, MIP, MIP_NODLFT, "Current unexplored node count." }
  impl_getter! { iter_cnt, f64, MIP, MIP_ITRCNT, "Current simplex iteration count." }

}

struct MIPSolData<'a>(&'a CbCtx<'a>);

impl<'a> MIPSolData<'a> {
  impl_getter! { obj, f64, MIPSOL, MIPSOL_OBJ, "Objective value for the new solution." }
  impl_getter! { obj_best, f64, MIPSOL, MIPSOL_OBJBST, "Current best objective." }
  impl_getter! { obj_bnd, f64, MIPSOL, MIPSOL_OBJBND, "Current best objective bound." }
  impl_getter! { node_cnt, f64, MIPSOL, MIPSOL_NODCNT, "Current explored node count." }
  impl_getter! { sol_cnt, i32, MIPSOL, MIPSOL_SOLCNT, "Current count of feasible solutions found." }
}

struct MIPNodeData<'a>(&'a CbCtx<'a>);

impl<'a> MIPNodeData<'a> {
  // Optimization status of current MIP node
  pub fn status(&self) -> Result<Status> {
    self.0.get_int(MIPNODE, MIPNODE_STATUS).map(Status::from)
  }

  impl_getter! { obj_best, f64, MIPNODE, MIPNODE_OBJBST, "Current best objective." }
  impl_getter! { obj_bnd, f64, MIPNODE, MIPNODE_OBJBND, "Current best objective bound." }
  impl_getter! { node_cnt, f64, MIPNODE, MIPNODE_NODCNT, "Current explored node count." }
  impl_getter! { sol_cnt, i32, MIPNODE, MIPNODE_SOLCNT, "Current count of feasible solutions found." }
}

struct MessageData<'a>(&'a CbCtx<'a>);

impl<'a> MessageData<'a> {
  pub fn message(&self) -> Result<String> {
    callback.get_string(MESSAGE, MSG_STRING)?.trim().to_owned()
  }
}

struct BarrierData<'a>(&'a CbCtx<'a>);

impl<'a> BarrierData<'a> {
  impl_getter! { iter_cnt, i32, BARRIER, BARRIER_ITRCNT, "Current simplex iteration count." }
  impl_getter! { prim_obj, f64, BARRIER, BARRIER_PRIMOBJ, "Primal objective value for current barrier iterate." }
  impl_getter! { dual_obj, f64, BARRIER, BARRIER_DUALOBJ, "Dual objective value for current barrier iterate." }
  impl_getter! { prim_inf, f64, BARRIER, BARRIER_PRIMINF, "Primal infeasibility for current barrier iterate." }
  impl_getter! { dual_inf, f64, BARRIER, BARRIER_DUALINF, "Dual infeasibility for current barrier iterate." }
  impl_getter! { compl_viol, f64, BARRIER, BARRIER_COMPL, "Complementarity violation for current barrier iterate." }
}


pub enum Where<'a> {
  Polling,
  PreSolve(PreSolveData<'a>),
  Simplex(SimplexData<'a>),
  MIP(MIPData<'a>),
  MIPSol(MIPSolData<'a>),
  MIPNode(MIPNodeData<'a>),
  Message(MessageData<'a>),
  Barrier(BarrierData<'a>),
}

impl Where {
  fn new<'a>(ctx: &'a CbCtx) -> Result<Where<'a>> {
    let where_ = match ctx.where_raw {
      POLLING => Where::Polling,
      PRESOLVE => Where::PreSolve(PreSolveData(ctx)),
      SIMPLEX => Where::Simplex(SimplexData(ctx)),
      MIP => Where::MIP(MIPData(ctx)),
      MIPNODE => Where::MIPNode(MIPNodeData(ctx)),
      MIPSOL => Where::MIPSol(MIPSolData(ctx)),
      MESSAGE => Where::Message(MessageData(ctx)),
      BARRIER => Where::Barrier(BarrierData(ctx)),
      _ => {
        return Err(Error::NotYetSupported(format!("WHERE = {}", ctx.where_raw)))
      },
    };
    Ok(where_)
  }
}


/// The context object for Gurobi callback.
pub struct CbCtx<'a> {
  cbdata: *mut ffi::c_void,
  where_raw : i32,
  model: &'a Model,
  nvars: usize,
}


impl<'a> CbCtx<'a> {
  pub(crate) fn new(cbdata: *mut ffi::c_void, where_: i32, model: &'a Model, nvars: usize) -> Result<Self> {
    let mut callback = CbCtx {
      cbdata,
      where_raw : where_,
      model,
      nvars,
    };
    let where_: Where = where_.try_into().unwrap();
  
    callback.where_ = where_;
    Ok(callback)
  }

  /// Retrieve the location where the callback called.
  pub fn get_where(&self) -> Result<Where> {
    Where::new(self)
  }

  /// Retrive node relaxation solution values at the current node.
  pub fn get_node_rel(&self, vars: &[Var]) -> Result<Vec<f64>> {
// memo: only MIPNode && status == Optimal
// note that this MUST be after a call to model.update(), so the indices in model.vars are Added and the unwrap() is ok
    let vals = self.get_double_array_vars(MIPNODE, MIPNODE_REL)?;
    vars.iter().map(|v| Ok(vals[self.model.get_index(v)? as usize])).collect()
  }

  /// Retrieve values from the current solution vector.
  pub fn get_solution(&self, vars: &[Var]) -> Result<Vec<f64>> {
    let inds = self.model.get_indices(vars)?;
    let buf = self.get_double_array_vars(MIPSOL, MIPSOL_SOL)?;
    Ok(inds.into_iter().map(|i| buf[i as usize]).collect())
  }

  /// Provide a new feasible solution for a MIP model.  Not all variables need to be given.
  pub fn set_solution(&self, vars: &[Var], solution: &[f64]) -> Result<f64> {
    if vars.len() != solution.len() {
      return Err(Error::InconsistentDims);
    }

    let inds = self.model.get_indices(vars)?;
    let mut soln = vec![GRB_UNDEFINED; self.model.get_attr(crate::attr::NumVars)? as usize];
    for (i, &val) in inds.into_iter().zip(solution.iter()) {
      soln[i as usize] = val;
    }
    let mut obj = INFINITY as raw::c_double;
    self.check_apicall(unsafe { ffi::GRBcbsolution(self.cbdata, soln.as_ptr(), &mut obj as *mut raw::c_double) })?;
    Ok(obj)
  }

  /// Retrieve the elapsed solver runtime in seconds.
  pub fn get_runtime(&self) -> Result<f64> {
    if let WhereData::Polling = self.get_where() {
      return Err(Error::FromAPI("Cannot query runtime in Polling state".to_owned(), 40001));
    }
    self.get_double(self.get_where().into(), RUNTIME)
  }

  /// Add a new cutting plane to the MIP model.
  pub fn add_cut(&self, lhs: LinExpr, sense: ConstrSense, rhs: f64) -> Result<()> {
    let offset = lhs.get_offset();
// note the user can still provide a LinExpr containing vars from a different model, so unwrap() won't work
    let (inds, coeff) = self.model.get_coeffs_indices_build(&lhs)?;
    self.check_apicall(unsafe {
      ffi::GRBcbcut(self.cbdata,
                    coeff.len() as ffi::c_int,
                    inds.as_ptr(),
                    coeff.as_ptr(),
                    sense.into(),
                    rhs - offset)
    })
  }

  /// Add a new lazy constraint to the MIP model.
  pub fn add_lazy(&self, lhs: LinExpr, sense: ConstrSense, rhs: f64) -> Result<()> {
    let offset = lhs.get_offset();
    let (inds, coeff) = self.model.get_coeffs_indices_build(&lhs)?;
    self.check_apicall(unsafe {
      ffi::GRBcblazy(self.cbdata,
                     coeff.len() as ffi::c_int,
                     inds.as_ptr(),
                     coeff.as_ptr(),
                     sense.into(),
                     rhs - offset)
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
// let self.model.vars
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
