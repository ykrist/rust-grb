// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use std::ops::Deref;
use std::mem::transmute;
use super::super::ffi;

use model::Model;
use error::{Error, Result};
use std::ptr::null;
use util;
use model::Var;
use model::LinExpr;
use model::ConstrSense;
use model::ProxyBase;
use itertools::Itertools;


/// Defines callback codes. See also
/// [the reference of official site](https://www.gurobi.com/documentation/6.5/refman/callback_codes.html).
#[allow(non_camel_case_types)]
pub mod exports {
  /// Location where the callback called.
  #[derive(Debug, Copy, Clone)]
  pub enum Where {
    Polling = 0,
    PreSolve,
    Simplex,
    MIP,
    MIPSol,
    MIPNode,
    Message,
    Barrier
  }

  /// Name of integer attributes in callback
  #[derive(Debug, Copy, Clone)]
  pub enum WhatInt {
    Pre_ColDel = 1000,
    Pre_RowDel = 1001,
    Pre_SenChg = 1002,
    Pre_BndChg = 1003,
    Pre_CoeChg = 1004,

    Spx_IsPert = 2004,

    MIP_SolCnt = 3003,
    MIP_CutCnt = 3004,

    MIPSol_SolCnt = 4001,

    MIP_NodeStatus = 5001,
    MIP_NodeSolCnt = 5006,

    BarrierItrCnt = 7001
  }

  /// Name of floating attributes in callback
  #[derive(Debug, Copy, Clone)]
  pub enum WhatDouble {
    Runtime = 6001,

    Spx_ItrCnt = 2000,
    Spx_ObjVal = 2001,
    Spx_PrimInf = 2002,
    Spx_DualInf = 2003,

    MIP_ObjBst = 3000,
    MIP_ObjBnd = 3001,
    MIP_NodCnt = 3002,
    MIP_NodLeft = 3005,
    MIP_ItrCnt = 3006,

    MIPSol_Obj = 4002,
    MIPSol_ObjBst = 4003,
    MIPSol_ObjBnd = 4004,
    MIPSol_NodCnt = 4005,

    MIPNode_ObjBst = 5003,
    MIPNode_ObjBnd = 5004,
    MIPNode_NodCnt = 5005,

    Barrier_PrimObj = 7002,
    Barrier_DualObj = 7003,
    Barrier_PrimInf = 7004,
    Barrier_DualInf = 7005,
    Barrier_Compl = 7006
  }

  // re-exports
  pub use self::Where::*;
  pub use self::WhatInt::*;
  pub use self::WhatDouble::*;
}
use self::exports::{Where, WhatInt, WhatDouble};

pub trait What: Into<i32> {
  type Out;
  type Buf: Default + Into<Self::Out>;
}

impl From<i32> for Where {
  fn from(val: i32) -> Where {
    match val {
      0...7 => unsafe { transmute(val as u8) },
      _ => panic!("invalid conversion")
    }
  }
}

impl Into<i32> for Where {
  fn into(self) -> i32 { self as i32 }
}


impl Into<i32> for WhatInt {
  fn into(self) -> i32 { self as i32 }
}

impl Into<i32> for WhatDouble {
  fn into(self) -> i32 { self as i32 }
}

impl What for WhatInt {
  type Out = i32;
  type Buf = ffi::c_int;
}

impl What for WhatDouble {
  type Out = f64;
  type Buf = ffi::c_double;
}


/// a
pub struct Context<'a> {
  cbdata: *mut ffi::c_void,
  where_: Where,
  model: &'a Model<'a>
}

impl<'a> Context<'a> {
  /// a
  pub fn get_where(&self) -> Where { self.where_ }

  /// a
  pub fn get_model(&self) -> &Model { self.model }

  /// a
  pub fn get<C: What>(&self, what: C) -> Result<C::Out> {
    let mut buf = C::Buf::default();
    self.check_apicall(unsafe {
        ffi::GRBcbget(self.cbdata,
                      self.where_.clone().into(),
                      what.into(),
                      transmute(&mut buf))
      })
      .and(Ok(buf.into()))
  }

  /// a
  pub fn get_msg_string(&self) -> Result<String> {
    const MSG_STRING: i32 = 6002;

    let mut buf = null();
    self.check_apicall(unsafe {
        ffi::GRBcbget(self.cbdata,
                      self.where_.clone().into(),
                      MSG_STRING,
                      transmute(&mut buf))
      })
      .and(Ok(unsafe { util::from_c_str(buf) }))
  }

  /// a
  pub fn get_node_rel(&self, vars: &[Var]) -> Result<Vec<f64>> {
    const MIPNODE_REL: i32 = 5002;
    self.get_double_array(vars, MIPNODE_REL)
  }

  /// a
  pub fn get_solution(&self, vars: &[Var]) -> Result<Vec<f64>> {
    const MIPSOL_SOL: i32 = 4001;
    self.get_double_array(vars, MIPSOL_SOL)
  }

  fn get_double_array(&self, vars: &[Var], what: i32) -> Result<Vec<f64>> {
    let mut buf = vec![0.0; self.model.vars.len()];

    self.check_apicall(unsafe {
        ffi::GRBcbget(self.cbdata,
                      self.where_.clone().into(),
                      what,
                      transmute(buf.as_mut_ptr()))
      })
      .and(Ok(vars.iter().map(|v| buf[v.index() as usize]).collect_vec()))
  }

  /// Provide a new feasible solution for a MIP model.
  pub fn set_solution(&self, solution: &[f64]) -> Result<()> {
    if solution.len() < self.model.vars.len() {
      return Err(Error::InconsitentDims);
    }

    self.check_apicall(unsafe { ffi::GRBcbsolution(self.cbdata, solution.as_ptr()) })
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

  fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(())
  }
}

pub trait New<'a> {
  fn new(cbdata: *mut ffi::c_void, where_: Where, model: &'a Model<'a>) -> Context<'a>;
}

impl<'a> New<'a> for Context<'a> {
  fn new(cbdata: *mut ffi::c_void, where_: Where, model: &'a Model<'a>) -> Context<'a> {
    Context {
      cbdata: cbdata,
      where_: where_,
      model: model
    }
  }
}


impl<'a> Deref for Context<'a> {
  type Target = Model<'a>;
  fn deref(&self) -> &Model<'a> { self.model }
}
