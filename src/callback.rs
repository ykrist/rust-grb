// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.


use gurobi_sys as ffi;
use std::ops::Deref;
use std::ptr::null;
use std::os::raw;

use crate::{Error, Result, Model, Var, ConstrSense, INFINITY};
use crate::expr::LinExpr;
use crate::util;
use crate::constants::callback::*;
// used for setting a partial solution in a callback
use crate::constants::GRB_UNDEFINED;

/// Location where the callback called
///
/// If you want to get more information, see [official
/// manual](https://www.gurobi.com/documentation/6.5/refman/callback_codes.html).
#[derive(Debug, Clone)]
pub enum Where {
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
    coecfg: i32
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
    ispert: i32
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
    itrcnt: f64
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
    solcnt: f64
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
    solcnt: i32
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
    compl: f64
  }
}

impl Into<i32> for Where {
  fn into(self) -> i32 {
    match self {
      Where::Polling => POLLING,
      Where::PreSolve { .. } => PRESOLVE,
      Where::Simplex { .. } => SIMPLEX,
      Where::MIP { .. } => MIP,
      Where::MIPSol { .. } => MIPSOL,
      Where::MIPNode { .. } => MIPNODE,
      Where::Message(_) => MESSAGE,
      Where::Barrier { .. } => BARRIER,
    }
  }
}


/// The context object for Gurobi callback.
pub struct Callback<'a> {
  cbdata: *mut ffi::c_void,
  where_: Where,
  model: &'a Model
}


pub trait New<'a> {
  #[allow(clippy::new_ret_no_self)]
  fn new(cbdata: *mut ffi::c_void, where_: i32, model: &'a Model) -> Result<Callback<'a>>;
}

impl<'a> New<'a> for Callback<'a> {
  fn new(cbdata: *mut ffi::c_void, where_: i32, model: &'a Model) -> Result<Callback<'a>> {
    let mut callback = Callback {
      cbdata,
      where_: Where::Polling,
      model
    };

    let where_ = match where_ {
      POLLING => Where::Polling,
      PRESOLVE => {
        Where::PreSolve {
          coldel: callback.get_int(PRESOLVE, PRE_COLDEL)?,
          rowdel: callback.get_int(PRESOLVE, PRE_ROWDEL)?,
          senchg: callback.get_int(PRESOLVE, PRE_SENCHG)?,
          bndchg: callback.get_int(PRESOLVE, PRE_BNDCHG)?,
          coecfg: callback.get_int(PRESOLVE, PRE_COECHG)?
        }
      }

      SIMPLEX => {
        Where::Simplex {
          itrcnt: callback.get_double(SIMPLEX, SPX_ITRCNT)?,
          objval: callback.get_double(SIMPLEX, SPX_OBJVAL)?,
          priminf: callback.get_double(SIMPLEX, SPX_PRIMINF)?,
          dualinf: callback.get_double(SIMPLEX, SPX_DUALINF)?,
          ispert: callback.get_int(SIMPLEX, SPX_ISPERT)?
        }
      }
      MIP => {
        Where::MIP {
          objbst: callback.get_double(MIP, MIP_OBJBST)?,
          objbnd: callback.get_double(MIP, MIP_OBJBND)?,
          nodcnt: callback.get_double(MIP, MIP_NODCNT)?,
          solcnt: callback.get_double(MIP, MIP_SOLCNT)?,
          cutcnt: callback.get_int(MIP, MIP_CUTCNT)?,
          nodleft: callback.get_double(MIP, MIP_NODLFT)?,
          itrcnt: callback.get_double(MIP, MIP_ITRCNT)?
        }
      }
      MIPSOL => {
        Where::MIPSol {
          obj: callback.get_double(MIPSOL, MIPSOL_OBJ)?,
          objbst: callback.get_double(MIPSOL, MIPSOL_OBJBST)?,
          objbnd: callback.get_double(MIPSOL, MIPSOL_OBJBND)?,
          nodcnt: callback.get_double(MIPSOL, MIPSOL_NODCNT)?,
          solcnt: callback.get_double(MIPSOL, MIPSOL_SOLCNT)?
        }
      }
      MIPNODE => {
        Where::MIPNode {
          status: callback.get_int(MIPNODE, MIPNODE_STATUS)?,
          objbst: callback.get_double(MIPNODE, MIPNODE_OBJBST)?,
          objbnd: callback.get_double(MIPNODE, MIPNODE_OBJBND)?,
          nodcnt: callback.get_double(MIPNODE, MIPNODE_NODCNT)?,
          solcnt: callback.get_int(MIPNODE, MIPNODE_SOLCNT)?
        }
      }
      MESSAGE => Where::Message(callback.get_string(MESSAGE, MSG_STRING)?.trim().to_owned()),
      BARRIER => {
        Where::Barrier {
          itrcnt: callback.get_int(BARRIER, BARRIER_ITRCNT)?,
          primobj: callback.get_double(BARRIER, BARRIER_PRIMOBJ)?,
          dualobj: callback.get_double(BARRIER, BARRIER_DUALOBJ)?,
          priminf: callback.get_double(BARRIER, BARRIER_PRIMINF)?,
          dualinf: callback.get_double(BARRIER, BARRIER_DUALINF)?,
          compl: callback.get_double(BARRIER, BARRIER_COMPL)?
        }
      }
      _ => panic!("Invalid callback location. {}", where_)
    };

    callback.where_ = where_;
    Ok(callback)
  }
}


impl<'a> Callback<'a> {
  /// Retrieve the location where the callback called.
  pub fn get_where(&self) -> Where { self.where_.clone() }

  /// Retrive node relaxation solution values at the current node.
  pub fn get_node_rel(&self, vars: &[Var]) -> Result<Vec<f64>> {
    // memo: only MIPNode && status == Optimal
    // note that this MUST be after a call to model.update(), so the indices in model.vars are Added and the unwrap() is ok
    let vals = self.get_double_array(MIPNODE, MIPNODE_REL)?;
    vars.iter().map(|v|  Ok(vals[self.model.get_index(v)? as usize])).collect()
  }

  /// Retrieve values from the current solution vector.
  pub fn get_solution(&self, vars: &[Var]) -> Result<Vec<f64>> {
    let inds = self.model.get_indices(vars)?;
    let buf = self.get_double_array(MIPSOL, MIPSOL_SOL)?;
    Ok(inds.into_iter().map(|i| buf[i as usize]).collect())
  }

  /// Provide a new feasible solution for a MIP model.
  pub fn set_solution(&self, vars: &[Var], solution: &[f64]) -> Result<f64> {
    if vars.len() != solution.len() || vars.len() < self.model.vars.len() {
      return Err(Error::InconsistentDims);
    }

    let inds = self.model.get_indices(vars)?;
    let mut soln = vec![GRB_UNDEFINED; self.model.vars.len()];
    for (i, &val) in inds.into_iter().zip(solution.iter()) {
      soln[i as usize] = val;
    }
    let mut obj = INFINITY as raw::c_double;
    self.check_apicall(unsafe { ffi::GRBcbsolution(self.cbdata, soln.as_ptr(), &mut obj as *mut raw::c_double) })?;
    Ok(obj)
  }

  /// Retrieve the elapsed solver runtime [sec].
  pub fn get_runtime(&self) -> Result<f64> {
    if let Where::Polling = self.get_where()  {
      return Err(Error::FromAPI("bad call in callback".to_owned(), 40001));
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


  fn get_int(&self, where_: i32, what: i32) -> Result<i32> {
    let mut buf = 0i32;
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, &mut buf as *mut i32 as *mut raw::c_void) }).and(Ok(buf))
  }

  fn get_double(&self, where_: i32, what: i32) -> Result<f64> {
    let mut buf = 0.0f64;
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, &mut buf as *mut f64 as *mut raw::c_void) }).and(Ok(buf))
  }

  fn get_double_array(&self, where_: i32, what: i32) -> Result<Vec<f64>> {
    let mut buf = vec![0.0; self.model.vars.len()];
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, buf.as_mut_ptr() as *mut  raw::c_void) }).and(Ok(buf))
  }

  fn get_string(&self, where_: i32, what: i32) -> Result<String> {
    let mut buf = null();
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what,  &mut buf as *mut *const i8 as *mut raw::c_void) })
      .and(Ok(unsafe { util::copy_c_str(buf) }))
  }

  fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(())
  }
}


impl<'a> Deref for Callback<'a> {
  type Target = Model;
  fn deref(&self) -> &Model { self.model }
}
