// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.


use ffi;
use itertools::{Itertools, Zip};

use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null;

use error::{Error, Result};
use model::{Model, Var, LinExpr, ConstrSense, ProxyBase};
use util;

// Location where the callback called.
const POLLING: i32 = 0;
const PRESOLVE: i32 = 1;
const SIMPLEX: i32 = 2;
const MIP: i32 = 3;
const MIPSOL: i32 = 4;
const MIPNODE: i32 = 5;
const MESSAGE: i32 = 6;
const BARRIER: i32 = 7;


const PRE_COLDEL: i32 = 1000;
const PRE_ROWDEL: i32 = 1001;
const PRE_SENCHG: i32 = 1002;
const PRE_BNDCHG: i32 = 1003;
const PRE_COECHG: i32 = 1004;

const SPX_ITRCNT: i32 = 2000;
const SPX_OBJVAL: i32 = 2001;
const SPX_PRIMINF: i32 = 2002;
const SPX_DUALINF: i32 = 2003;
const SPX_ISPERT: i32 = 2004;

const MIP_OBJBST: i32 = 3000;
const MIP_OBJBND: i32 = 3001;
const MIP_NODCNT: i32 = 3002;
const MIP_SOLCNT: i32 = 3003;
const MIP_CUTCNT: i32 = 3004;
const MIP_NODLFT: i32 = 3005;
const MIP_ITRCNT: i32 = 3006;

#[allow(dead_code)]
const MIP_OBJBNDC: i32 = 3007;

const MIPSOL_SOL: i32 = 4001;
const MIPSOL_OBJ: i32 = 4002;
const MIPSOL_OBJBST: i32 = 4003;
const MIPSOL_OBJBND: i32 = 4004;
const MIPSOL_NODCNT: i32 = 4005;
const MIPSOL_SOLCNT: i32 = 4006;
#[allow(dead_code)]
const MIPSOL_OBJBNDC: i32 = 4007;

const MIPNODE_STATUS: i32 = 5001;
const MIPNODE_REL: i32 = 5002;
const MIPNODE_OBJBST: i32 = 5003;
const MIPNODE_OBJBND: i32 = 5004;
const MIPNODE_NODCNT: i32 = 5005;
const MIPNODE_SOLCNT: i32 = 5006;
#[allow(dead_code)]
const MIPNODE_BRVAR: i32 = 5007;
#[allow(dead_code)]
const MIPNODE_OBJBNDC: i32 = 5008;

const MSG_STRING: i32 = 6001;
const RUNTIME: i32 = 6002;

const BARRIER_ITRCNT: i32 = 7001;
const BARRIER_PRIMOBJ: i32 = 7002;
const BARRIER_DUALOBJ: i32 = 7003;
const BARRIER_PRIMINF: i32 = 7004;
const BARRIER_DUALINF: i32 = 7005;
const BARRIER_COMPL: i32 = 7006;


#[derive(Debug, Clone)]
pub enum Where {
  /// b
  Polling,

  /// a
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

  /// c
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

  /// d
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

  /// e
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

  /// f
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

  /// f
  Message(String),

  /// d
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


/// Gurobi callback object
pub struct Callback<'a> {
  cbdata: *mut ffi::c_void,
  where_: Where,
  model: &'a Model<'a>
}


pub trait New<'a> {
  fn new(cbdata: *mut ffi::c_void, where_: i32, model: &'a Model<'a>) -> Result<Callback<'a>>;
}

impl<'a> New<'a> for Callback<'a> {
  fn new(cbdata: *mut ffi::c_void, where_: i32, model: &'a Model<'a>) -> Result<Callback<'a>> {
    let mut callback = Callback {
      cbdata: cbdata,
      where_: Where::Polling,
      model: model
    };

    let where_ = match where_ {
      POLLING => Where::Polling,
      PRESOLVE => {
        Where::PreSolve {
          coldel: try!(callback.get_int(PRESOLVE, PRE_COLDEL)),
          rowdel: try!(callback.get_int(PRESOLVE, PRE_ROWDEL)),
          senchg: try!(callback.get_int(PRESOLVE, PRE_SENCHG)),
          bndchg: try!(callback.get_int(PRESOLVE, PRE_BNDCHG)),
          coecfg: try!(callback.get_int(PRESOLVE, PRE_COECHG))
        }
      }

      SIMPLEX => {
        Where::Simplex {
          itrcnt: try!(callback.get_double(SIMPLEX, SPX_ITRCNT)),
          objval: try!(callback.get_double(SIMPLEX, SPX_OBJVAL)),
          priminf: try!(callback.get_double(SIMPLEX, SPX_PRIMINF)),
          dualinf: try!(callback.get_double(SIMPLEX, SPX_DUALINF)),
          ispert: try!(callback.get_int(SIMPLEX, SPX_ISPERT))
        }
      }
      MIP => {
        Where::MIP {
          objbst: try!(callback.get_double(MIP, MIP_OBJBST)),
          objbnd: try!(callback.get_double(MIP, MIP_OBJBND)),
          nodcnt: try!(callback.get_double(MIP, MIP_NODCNT)),
          solcnt: try!(callback.get_double(MIP, MIP_SOLCNT)),
          cutcnt: try!(callback.get_int(MIP, MIP_CUTCNT)),
          nodleft: try!(callback.get_double(MIP, MIP_NODLFT)),
          itrcnt: try!(callback.get_double(MIP, MIP_ITRCNT))
        }
      }
      MIPSOL => {
        Where::MIPSol {
          obj: try!(callback.get_double(MIPSOL, MIPSOL_OBJ)),
          objbst: try!(callback.get_double(MIPSOL, MIPSOL_OBJBST)),
          objbnd: try!(callback.get_double(MIPSOL, MIPSOL_OBJBND)),
          nodcnt: try!(callback.get_double(MIPSOL, MIPSOL_NODCNT)),
          solcnt: try!(callback.get_double(MIPSOL, MIPSOL_SOLCNT)),
        }
      }
      MIPNODE => {
        Where::MIPNode {
          status: try!(callback.get_int(MIPNODE, MIPNODE_STATUS)),
          objbst: try!(callback.get_double(MIPNODE, MIPNODE_OBJBST)),
          objbnd: try!(callback.get_double(MIPNODE, MIPNODE_OBJBND)),
          nodcnt: try!(callback.get_double(MIPNODE, MIPNODE_NODCNT)),
          solcnt: try!(callback.get_int(MIPNODE, MIPNODE_SOLCNT)),
        }
      }
      MESSAGE => Where::Message(try!(callback.get_string(MESSAGE, MSG_STRING)).trim().to_owned()),
      BARRIER => {
        Where::Barrier {
          itrcnt: try!(callback.get_int(BARRIER, BARRIER_ITRCNT)),
          primobj: try!(callback.get_double(BARRIER, BARRIER_PRIMOBJ)),
          dualobj: try!(callback.get_double(BARRIER, BARRIER_DUALOBJ)),
          priminf:try!(callback.get_double(BARRIER, BARRIER_PRIMINF)),
          dualinf: try!(callback.get_double(BARRIER, BARRIER_DUALINF)),
          compl: try!(callback.get_double(BARRIER, BARRIER_COMPL)),
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
    self.get_double_array(MIPNODE, MIPNODE_REL).map(|buf| vars.iter().map(|v| buf[v.index() as usize]).collect_vec())
  }

  /// Retrieve values from the current solution vector.
  pub fn get_solution(&self, vars: &[Var]) -> Result<Vec<f64>> {
    self.get_double_array(MIPSOL, MIPSOL_SOL).map(|buf| vars.iter().map(|v| buf[v.index() as usize]).collect_vec())
  }

  /// Provide a new feasible solution for a MIP model.
  pub fn set_solution(&self, vars: &[Var], solution: &[f64]) -> Result<()> {
    if vars.len() != solution.len() || vars.len() < self.model.vars.len() {
      return Err(Error::InconsitentDims);
    }

    let mut buf = vec![0.0; self.model.vars.len()];
    for (v, &sol) in Zip::new((vars.iter(), solution.iter())) {
      let i = v.index() as usize;
      buf[i] = sol;
    }

    self.check_apicall(unsafe { ffi::GRBcbsolution(self.cbdata, buf.as_ptr()) })
  }

  ///
  pub fn get_runtime(&self) -> Result<f64> {
    match self.get_where() {
      Where::Polling => return Err(Error::FromAPI("bad call in callback".to_owned(), 40001)),
      _ => ()
    }
    self.get_double(self.get_where().into(), RUNTIME)
  }

  /// Add a new cutting plane to the MIP model.
  pub fn add_cut(&self, lhs: LinExpr, sense: ConstrSense, rhs: f64) -> Result<()> {
    self.check_apicall(unsafe {
      ffi::GRBcbcut(self.cbdata,
                    lhs.coeff.len() as ffi::c_int,
                    lhs.vars.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                    lhs.coeff.as_ptr(),
                    sense.into(),
                    rhs - lhs.offset)
    })
  }

  /// Add a new lazy constraint to the MIP model.
  pub fn add_lazy(&self, lhs: LinExpr, sense: ConstrSense, rhs: f64) -> Result<()> {
    self.check_apicall(unsafe {
      ffi::GRBcblazy(self.cbdata,
                     lhs.coeff.len() as ffi::c_int,
                     lhs.vars.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                     lhs.coeff.as_ptr(),
                     sense.into(),
                     rhs - lhs.offset)
    })
  }


  fn get_int(&self, where_: i32, what: i32) -> Result<i32> {
    let mut buf = 0;
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, transmute(&mut buf)) }).and(Ok(buf.into()))
  }

  fn get_double(&self, where_: i32, what: i32) -> Result<f64> {
    let mut buf = 0.0;
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, transmute(&mut buf)) }).and(Ok(buf.into()))
  }

  fn get_double_array(&self, where_: i32, what: i32) -> Result<Vec<f64>> {
    let mut buf = vec![0.0; self.model.vars.len()];
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, transmute(buf.as_mut_ptr())) }).and(Ok(buf))
  }

  fn get_string(&self, where_: i32, what: i32) -> Result<String> {
    let mut buf = null();
    self.check_apicall(unsafe { ffi::GRBcbget(self.cbdata, where_, what, transmute(&mut buf)) })
      .and(Ok(unsafe { util::from_c_str(buf) }))
  }

  fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(())
  }
}


impl<'a> Deref for Callback<'a> {
  type Target = Model<'a>;
  fn deref(&self) -> &Model<'a> { self.model }
}
