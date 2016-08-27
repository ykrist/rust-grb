// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use super::ffi;
use super::itertools::{Itertools, Zip};

use std::ffi::CString;
use std::iter;
use std::ptr::{null, null_mut};
use std::ops::{Add, Sub, Mul};
use std::mem::transmute;
use std::rc::Rc;
use std::cell::Cell;
use std::slice::Iter;

use attrib as attr;
use env::{Env, FromRaw, ErrorFromAPI};
use error::{Error, Result};
use util;



/// Type for new variable
#[derive(Debug,Clone,Copy)]
pub enum VarType {
  Binary,
  Continuous(f64, f64),
  Integer(i64, i64)
}

impl Into<(ffi::c_char, f64, f64)> for VarType {
  fn into(self) -> (ffi::c_char, f64, f64) {
    use self::VarType::*;
    match self {
      Binary => ('B' as ffi::c_char, 0.0, 1.0),
      Continuous(lb, ub) => ('C' as ffi::c_char, lb, ub),
      Integer(lb, ub) => ('I' as ffi::c_char, lb as ffi::c_double, ub as ffi::c_double),
    }
  }
}


/// Sense for new linear/quadratic constraint
#[derive(Debug,Copy,Clone)]
pub enum ConstrSense {
  Equal,
  Greater,
  Less
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
#[derive(Debug)]
pub enum ModelSense {
  Minimize,
  Maximize
}

impl Into<i32> for ModelSense {
  fn into(self) -> i32 {
    match self {
      ModelSense::Minimize => -1,
      ModelSense::Maximize => 1,
    }
  }
}


/// Type of new SOS constraint
#[derive(Debug)]
pub enum SOSType {
  SOSType1,
  SOSType2
}

impl Into<i32> for SOSType {
  fn into(self) -> i32 {
    match self {
      SOSType::SOSType1 => 1,
      SOSType::SOSType2 => 2,
    }
  }
}


/// Status of a model
#[derive(Debug,PartialEq)]
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
  InProgress
}

impl From<i32> for Status {
  fn from(val: i32) -> Status {
    match val {
      1...14 => unsafe { transmute(val as i8) },
      _ => panic!("cannot convert to Status: {}", val)
    }
  }
}


pub trait ProxyBase {
  fn new(i32) -> Self;
  fn index(&self) -> i32;
  fn set_index(&mut self, value: i32);
}

/// Provides methods to query/modify attributes associated with certain element.
pub trait Proxy: ProxyBase {
  /// Query the value of attribute.
  fn get<A: attr::AttrArrayBase>(&self, model: &Model, attr: A) -> Result<A::Out> {
    model.get_element(attr, self.index())
  }

  /// Set the value of attribute.
  fn set<A: attr::AttrArrayBase>(&mut self, model: &mut Model, attr: A, val: A::Out) -> Result<()> {
    model.set_element(attr, self.index(), val)
  }
}


/// Proxy object of a variables
#[derive(Clone)]
pub struct Var(Rc<Cell<i32>>);

/// Proxy object of a linear constraint
#[derive(Clone)]
pub struct Constr(Rc<Cell<i32>>);

/// Proxy object of a quadratic constraint
#[derive(Clone)]
pub struct QConstr(Rc<Cell<i32>>);

/// Proxy object of a Special Order Set (SOS) constraint
#[derive(Clone)]
pub struct SOS(Rc<Cell<i32>>);

impl ProxyBase for Var {
  fn new(idx: i32) -> Var { Var(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }
}

impl Proxy for Var {}

impl ProxyBase for Constr {
  fn new(idx: i32) -> Constr { Constr(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }
}

impl Proxy for Constr {}

impl ProxyBase for QConstr {
  fn new(idx: i32) -> QConstr { QConstr(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }
}

impl Proxy for QConstr {}

impl ProxyBase for SOS {
  fn new(idx: i32) -> SOS { SOS(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }
}

impl Proxy for SOS {}



/// Linear expression of variables
///
/// A linear expression consists of a constant term plus a list of coefficients and variables.
#[derive(Clone)]
pub struct LinExpr {
  vars: Vec<Var>,
  coeff: Vec<f64>,
  offset: f64
}

impl LinExpr {
  /// Create an empty linear expression.
  pub fn new() -> Self {
    LinExpr {
      vars: Vec::new(),
      coeff: Vec::new(),
      offset: 0.0
    }
  }

  /// Add a linear term into the expression.
  pub fn add_term(mut self, coeff: f64, var: Var) -> Self {
    self.coeff.push(coeff);
    self.vars.push(var);
    self
  }

  /// Add a constant into the expression.
  pub fn add_constant(mut self, constant: f64) -> Self {
    self.offset += constant;
    self
  }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let vars = try!(model.get_values(attr::exports::X, self.vars.as_slice()));

    Ok(Zip::new((vars, self.coeff.iter())).fold(0.0, |acc, (ind, val)| acc + ind * val) + self.offset)
  }
}

impl<'a> Into<QuadExpr> for &'a Var {
  fn into(self) -> QuadExpr { QuadExpr::new().add_term(1.0, self.clone()) }
}

impl Into<QuadExpr> for LinExpr {
  fn into(self) -> QuadExpr {
    QuadExpr {
      lind: self.vars,
      lval: self.coeff,
      offset: self.offset,
      qrow: Vec::new(),
      qcol: Vec::new(),
      qval: Vec::new()
    }
  }
}


/// Quadratic expression of variables
///
/// A quadratic expression consists of a linear expression and a set of
/// variable-variable-coefficient triples to express the quadratic term.
#[derive(Clone)]
pub struct QuadExpr {
  lind: Vec<Var>,
  lval: Vec<f64>,
  qrow: Vec<Var>,
  qcol: Vec<Var>,
  qval: Vec<f64>,
  offset: f64
}

impl QuadExpr {
  pub fn new() -> Self {
    QuadExpr {
      lind: Vec::new(),
      lval: Vec::new(),
      qrow: Vec::new(),
      qcol: Vec::new(),
      qval: Vec::new(),
      offset: 0.0
    }
  }

  /// Add a linear term into the expression.
  pub fn add_term(mut self, coeff: f64, var: Var) -> Self {
    self.lind.push(var);
    self.lval.push(coeff);
    self
  }

  /// Add a quadratic term into the expression.
  pub fn add_qterm(mut self, coeff: f64, row: Var, col: Var) -> Self {
    self.qval.push(coeff);
    self.qrow.push(row);
    self.qcol.push(col);
    self
  }

  /// Add a constant into the expression.
  pub fn add_constant(mut self, constant: f64) -> Self {
    self.offset += constant;
    self
  }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let lind = try!(model.get_values(attr::exports::X, self.lind.as_slice()));
    let qrow = try!(model.get_values(attr::exports::X, self.qrow.as_slice()));
    let qcol = try!(model.get_values(attr::exports::X, self.qcol.as_slice()));

    Ok(Zip::new((lind, self.lval.iter())).fold(0.0, |acc, (ind, val)| acc + ind * val) +
       Zip::new((qrow, qcol, self.qval.iter())).fold(0.0, |acc, (row, col, val)| acc + row * col * val) +
       self.offset)
  }
}


impl Mul<f64> for Var {
  type Output = LinExpr;
  fn mul(self, rhs: f64) -> Self::Output { LinExpr::new().add_term(rhs, self) }
}

impl<'a> Mul<&'a Var> for f64 {
  type Output = LinExpr;
  fn mul(self, rhs: &'a Var) -> Self::Output { LinExpr::new().add_term(self, rhs.clone()) }
}


impl<'a> Mul for &'a Var {
  type Output = QuadExpr;
  fn mul(self, rhs: &'a Var) -> Self::Output { QuadExpr::new().add_qterm(1.0, self.clone(), rhs.clone()) }
}

impl Mul<f64> for QuadExpr {
  type Output = QuadExpr;
  fn mul(mut self, rhs: f64) -> Self::Output {
    for i in 0..(self.lval.len()) {
      self.lval[i] *= rhs;
    }
    for j in 0..(self.qval.len()) {
      self.qval[j] *= rhs;
    }
    self.offset *= rhs;
    self
  }
}


impl Add<f64> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: f64) -> Self::Output { self.add_constant(rhs) }
}

impl Sub<f64> for LinExpr {
  type Output = LinExpr;
  fn sub(self, rhs: f64) -> Self::Output { self.add_constant(-rhs) }
}


impl Add for LinExpr {
  type Output = LinExpr;
  fn add(mut self, rhs: LinExpr) -> Self::Output {
    self.vars.extend(rhs.vars);
    self.coeff.extend(rhs.coeff);
    self.offset += rhs.offset;
    self
  }
}

impl Sub for LinExpr {
  type Output = LinExpr;
  fn sub(mut self, rhs: LinExpr) -> Self::Output {
    self.vars.extend(rhs.vars);
    self.coeff.extend(rhs.coeff.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}


impl Add<LinExpr> for QuadExpr {
  type Output = QuadExpr;
  fn add(mut self, rhs: LinExpr) -> Self::Output {
    self.lind.extend(rhs.vars);
    self.lval.extend(rhs.coeff);
    self.offset += rhs.offset;
    self
  }
}

impl Sub<LinExpr> for QuadExpr {
  type Output = QuadExpr;
  fn sub(mut self, rhs: LinExpr) -> Self::Output {
    self.lind.extend(rhs.vars);
    self.lval.extend(rhs.coeff.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}

impl Add for QuadExpr {
  type Output = QuadExpr;
  fn add(mut self, rhs: QuadExpr) -> QuadExpr {
    self.lind.extend(rhs.lind);
    self.lval.extend(rhs.lval);
    self.qrow.extend(rhs.qrow);
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval);
    self.offset += rhs.offset;
    self
  }
}

impl Sub for QuadExpr {
  type Output = QuadExpr;
  fn sub(mut self, rhs: QuadExpr) -> QuadExpr {
    self.lind.extend(rhs.lind);
    self.lval.extend(rhs.lval.into_iter().map(|c| -c));
    self.qrow.extend(rhs.qrow);
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}

impl<'a> Add for &'a Var {
  type Output = LinExpr;
  fn add(self, rhs: &Var) -> LinExpr { LinExpr::new().add_term(1.0, self.clone()).add_term(1.0, rhs.clone()) }
}

impl<'a> Sub for &'a Var {
  type Output = LinExpr;
  fn sub(self, rhs: &Var) -> LinExpr { LinExpr::new().add_term(1.0, self.clone()).add_term(-1.0, rhs.clone()) }
}

impl<'a> Add<LinExpr> for &'a Var {
  type Output = LinExpr;
  fn add(self, rhs: LinExpr) -> LinExpr { rhs.add_term(1.0, self.clone()) }
}

impl<'a> Add<&'a Var> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: &'a Var) -> LinExpr { self.add_term(1.0, rhs.clone()) }
}



/// Type of cost function at feasibility relaxation
#[derive(Debug)]
pub enum RelaxType {
  /// The weighted magnitude of bounds and constraint violations
  /// (`penalty(s_i) = w_i s_i`)
  Linear,

  /// The weighted square of magnitude of bounds and constraint violations
  /// (`penalty(s_i) = w_i s_i^2`)
  Quadratic,

  /// The weighted count of bounds and constraint violations
  /// (`penalty(s_i) = w_i * [s_i > 0]`)
  Cardinality
}

impl Into<i32> for RelaxType {
  fn into(self) -> i32 {
    match self {
      RelaxType::Linear => 0,
      RelaxType::Quadratic => 1,
      RelaxType::Cardinality => 2,
    }
  }
}

pub mod callback {
  use std::mem::transmute;
  use super::super::ffi;
  use std::ptr::null;
  use util;


  #[derive(Debug, Clone)]
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


  pub enum WhatInt {
    PreColDel = 1000,
    PreRowDel = 1001,
    PreSenChg = 1002,
    PreBndChg = 1003,
    PreCoeChg = 1004,
    SpxIsPert = 2004,
    MIPSolCnt = 3003,
    MIPCutCnt = 3004,
    MIPSolSolCnt = 4001,
    MIPNodeStatus = 5001,
    MIPNodeSolCnt = 5006,
    BarrierItrCnt = 7001
  }

  pub enum WhatDouble {
    Runtime = 6001,
    SpxItrCnt = 2000,
    SpxObjVal = 2001,
    SpxPrimInf = 2002,
    SpxDualInf = 2003,
    MIPObjBst = 3000,
    MIPObjBnd = 3001,
    MIPNodCnt = 3002,
    MIPNodLeft = 3005,
    MIPItrCnt = 3006,
    MIPSolObj = 4002,
    MIPSolObjBst = 4003,
    MIPSolObjBnd = 4004,
    MIPSolNodCnt = 4005,
    MIPNodeObjBst = 5003,
    MIPNodeObjBnd = 5004,
    MIPNodeNodCnt = 5005,
    BarrierPrimObj = 7002,
    BarrierDualObj = 7003,
    BarrierPrimInf = 7004,
    BarrierDualInf = 7005,
    BarrierCompl = 7006
  }

  pub enum WhatDoubleArray {
    MIPSolSol = 4001,
    MIPNodeRel = 5002
  }

  pub enum WhatString {
    MsgString = 6002
  }

  pub trait What: Into<i32> {
    type Output;
    type Buf;

    fn init() -> Self::Buf;
    fn to_out(buf: Self::Buf) -> Self::Output;
  }


  impl Into<i32> for WhatInt {
    fn into(self) -> i32 { self as i32 }
  }

  impl Into<i32> for WhatDouble {
    fn into(self) -> i32 { self as i32 }
  }

  impl Into<i32> for WhatDoubleArray {
    fn into(self) -> i32 { self as i32 }
  }

  impl Into<i32> for WhatString {
    fn into(self) -> i32 { self as i32 }
  }

  impl What for WhatInt {
    type Output = i32;
    type Buf = ffi::c_int;
    fn init() -> ffi::c_int { 0 }
    fn to_out(buf: ffi::c_int) -> i32 { buf }
  }

  impl What for WhatDouble {
    type Output = f64;
    type Buf = ffi::c_double;
    fn init() -> ffi::c_double { 0.0 }
    fn to_out(buf: ffi::c_double) -> f64 { buf }
  }

  impl What for WhatString {
    type Output = String;
    type Buf = ffi::c_str;
    fn init() -> ffi::c_str { null() }
    fn to_out(buf: ffi::c_str) -> String { unsafe { util::from_c_str(buf) } }
  }


  // re-exports
  pub use self::Where::*;
  pub use self::WhatInt::*;
  pub use self::WhatDouble::*;
  pub use self::WhatDoubleArray::*;
  pub use self::WhatString::*;
}


#[allow(dead_code)]
pub struct Context<'a> {
  cbdata: *mut ffi::c_void,
  loc: callback::Where,
  model: &'a Model<'a>,
  ncols: usize
}

impl<'a> Context<'a> {
  pub fn get_loc(&self) -> &callback::Where { &self.loc }

  pub fn get_model(&self) -> &Model { self.model }

  pub fn get<C: callback::What>(&self, what: C) -> Result<C::Output> {
    let mut buf = C::init();
    let error = unsafe {
      ffi::GRBcbget(self.cbdata,
                    self.loc.clone().into(),
                    what.into(),
                    transmute(&mut buf))
    };
    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(C::to_out(buf))
  }

  pub fn get_array(&self, what: callback::WhatDoubleArray) -> Result<Vec<f64>> {
    let mut buf = Vec::with_capacity(self.ncols);
    buf.resize(self.ncols, 0.0);
    let error = unsafe {
      ffi::GRBcbget(self.cbdata,
                    self.loc.clone().into(),
                    what.into(),
                    transmute(buf.as_mut_ptr()))
    };
    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(buf)
  }

  /// Add a new cutting plane to the MIP model.
  pub fn add_cut(&self, lhs: LinExpr, sense: ConstrSense, rhs: f64) -> Result<()> {
    let error = unsafe {
      ffi::GRBcbcut(self.cbdata,
                    lhs.coeff.len() as ffi::c_int,
                    lhs.vars.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                    lhs.coeff.as_ptr(),
                    sense.into(),
                    rhs - lhs.offset)
    };
    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(())
  }


  /// Add a new lazy constraint to the MIP model.
  pub fn add_lazy(&self, lhs: LinExpr, sense: ConstrSense, rhs: f64) -> Result<()> {
    let error = unsafe {
      ffi::GRBcblazy(self.cbdata,
                     lhs.coeff.len() as ffi::c_int,
                     lhs.vars.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                     lhs.coeff.as_ptr(),
                     sense.into(),
                     rhs - lhs.offset)
    };
    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(())
  }

  /// Provide a new feasible solution for a MIP model.
  pub fn set_solution(&self, solution: &[f64]) -> Result<()> {
    if solution.len() < self.ncols {
      return Err(Error::InconsitentDims);
    }
    let error = unsafe { ffi::GRBcbsolution(self.cbdata, solution.as_ptr()) };

    if error != 0 {
      return Err(Error::FromAPI("Callback error".to_owned(), 40000));
    }
    Ok(())
  }

  /// terminate the optimization.
  pub fn terminate(&self) { self.get_model().terminate(); }
}


pub type Callback = fn(Context) -> Result<()>;

extern "C" fn callback_wrapper(model: *mut ffi::GRBmodel, cbdata: *mut ffi::c_void, loc: ffi::c_int,
                               usrdata: *mut ffi::c_void)
                               -> ffi::c_int {
  // null callback
  if usrdata.is_null() {
    return 0;
  }

  let themodel: &Model = unsafe { transmute(usrdata) };
  if themodel.model != model {
    println!("invalid callback context.");
    return -1;
  }

  if let Some(callback) = themodel.callback {
    let context = Context {
      cbdata: cbdata,
      loc: loc.into(),
      model: &themodel,
      ncols: themodel.vars.len()
    };

    match callback(context) {
      Ok(_) => 0,
      Err(_) => -1,
    }
  } else {
    0
  }
}


/// Gurobi model object associated with certain environment.
pub struct Model<'a> {
  model: *mut ffi::GRBmodel,
  env: &'a Env,
  vars: Vec<Var>,
  constrs: Vec<Constr>,
  qconstrs: Vec<QConstr>,
  sos: Vec<SOS>,
  callback: Option<Callback>
}

impl<'a> Model<'a> {
  /// create an empty model which associated with certain environment.
  pub fn new(env: &'a Env, model: *mut ffi::GRBmodel) -> Result<Model> {
    let mut model = Model {
      model: model,
      env: env,
      vars: Vec::new(),
      constrs: Vec::new(),
      qconstrs: Vec::new(),
      sos: Vec::new(),
      callback: None
    };
    try!(model.populate());
    Ok(model)
  }

  /// create a copy of the model
  pub fn copy(&self) -> Result<Model> {
    let copied = unsafe { ffi::GRBcopymodel(self.model) };
    if copied.is_null() {
      return Err(Error::FromAPI("Failed to create a copy of the model".to_owned(), 20002));
    }

    Model::new(self.env, copied).map(|mut model| {
      model.callback = self.callback;
      model
    })
  }

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
    Model::new(self.env, fixed)
  }

  /// Create an relaxation of the model (undocumented).
  pub fn relax(&self) -> Result<Model> {
    let relaxed = unsafe { ffi::GRBrelaxmodel(self.model) };
    if relaxed.is_null() {
      return Err(Error::FromAPI("failed to create relaxed model".to_owned(), 20002));
    }
    Model::new(self.env, relaxed)
  }

  /// Perform presolve on the model.
  pub fn presolve(&self) -> Result<Model> {
    let presolved = unsafe { ffi::GRBpresolvemodel(self.model) };
    if presolved.is_null() {
      return Err(Error::FromAPI("failed to create presolved model".to_owned(), 20002));
    }
    Model::new(self.env, presolved)
  }

  /// Create a feasibility model (undocumented).
  pub fn feasibility(&self) -> Result<Model> {
    let feasibility = unsafe { ffi::GRBfeasibility(self.model) };
    if feasibility.is_null() {
      return Err(Error::FromAPI("failed to create feasibility model".to_owned(), 20002));
    }
    Model::new(self.env, feasibility)
  }

  /// Apply all modification of the model to process
  pub fn update(&mut self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBupdatemodel(self.model) }) }

  /// Optimize the model synchronously
  pub fn optimize(&mut self) -> Result<()> {
    try!(self.update());
    self.check_apicall(unsafe { ffi::GRBoptimize(self.model) })
  }

  /// Optimize the model asynchronously
  pub fn optimize_async(&mut self) -> Result<()> {
    try!(self.update());
    self.check_apicall(unsafe { ffi::GRBoptimizeasync(self.model) })
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
  #[deprecated]
  pub fn tune(&self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBtunemodel(self.model) }) }

  /// Prepare to retrieve the results of `tune()`.
  /// See also references [on official
  /// manual](https://www.gurobi.com/documentation/6.5/refman/parameter_tuning_tool.html#sec:Tuning).
  #[deprecated]
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
  pub fn discard_concurrent_envs(&self) { unsafe { ffi::GRBdiscardconcurrentenvs(self.model) } }

  /// Insert a message into log file.
  ///
  /// When **message** cannot convert to raw C string, a panic is occurred.
  pub fn message(&self, message: &str) { self.env.message(message); }

  /// Import optimization data of the model from a file.
  pub fn read(&mut self, filename: &str) -> Result<()> {
    let filename = try!(CString::new(filename));
    self.check_apicall(unsafe { ffi::GRBread(self.model, filename.as_ptr()) })
  }

  /// Export optimization data of the model to a file.
  pub fn write(&self, filename: &str) -> Result<()> {
    let filename = try!(CString::new(filename));
    self.check_apicall(unsafe { ffi::GRBwrite(self.model, filename.as_ptr()) })
  }


  /// add a decision variable to the model.
  pub fn add_var(&mut self, name: &str, vtype: VarType) -> Result<Var> {
    // extract parameters
    let (vtype, lb, ub) = vtype.into();
    let name = try!(CString::new(name));

    try!(self.check_apicall(unsafe {
      ffi::GRBaddvar(self.model,
                     0,
                     null(),
                     null(),
                     0.0,
                     lb,
                     ub,
                     vtype,
                     name.as_ptr())
    }));

    let col_no = self.vars.len() as i32;
    self.vars.push(Var::new(col_no));

    Ok(self.vars.last().cloned().unwrap())
  }

  /// add a linear constraint to the model.
  pub fn add_constr(&mut self, name: &str, expr: LinExpr, sense: ConstrSense, rhs: f64) -> Result<Constr> {
    let constrname = try!(CString::new(name));
    try!(self.check_apicall(unsafe {
      ffi::GRBaddconstr(self.model,
                        expr.coeff.len() as ffi::c_int,
                        expr.vars.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                        expr.coeff.as_ptr(),
                        sense.into(),
                        rhs - expr.offset,
                        constrname.as_ptr())
    }));

    let row_no = self.constrs.len() as i32;
    self.constrs.push(Constr::new(row_no));

    Ok(self.constrs.last().cloned().unwrap())
  }

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self, constrname: &str, expr: QuadExpr, sense: ConstrSense, rhs: f64) -> Result<QConstr> {
    let constrname = try!(CString::new(constrname));
    try!(self.check_apicall(unsafe {
      ffi::GRBaddqconstr(self.model,
                         expr.lval.len() as ffi::c_int,
                         expr.lind.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                         expr.lval.as_ptr(),
                         expr.qval.len() as ffi::c_int,
                         expr.qrow.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                         expr.qcol.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                         expr.qval.as_ptr(),
                         sense.into(),
                         rhs,
                         constrname.as_ptr())
    }));

    let qrow_no = self.qconstrs.len() as i32;
    self.qconstrs.push(QConstr::new(qrow_no));

    Ok(self.qconstrs.last().cloned().unwrap())
  }

  /// add Special Order Set (SOS) constraint to the model.
  pub fn add_sos(&mut self, vars: &[Var], weights: &[f64], sostype: SOSType) -> Result<SOS> {
    if vars.len() != weights.len() {
      return Err(Error::InconsitentDims);
    }

    let vars = vars.iter().map(|v| v.index()).collect_vec();
    let beg = 0;

    try!(self.check_apicall(unsafe {
      ffi::GRBaddsos(self.model,
                     1,
                     vars.len() as ffi::c_int,
                     &sostype.into(),
                     &beg,
                     vars.as_ptr(),
                     weights.as_ptr())
    }));

    let sos_no = self.sos.len() as i32;
    self.sos.push(SOS::new(sos_no));

    Ok(self.sos.last().cloned().unwrap())
  }

  /// Set the objective function of the model.
  pub fn set_objective<Expr: Into<QuadExpr>>(&mut self, expr: Expr, sense: ModelSense) -> Result<()> {
    let expr = expr.into();
    let lind = expr.lind.into_iter().map(|v| v.index()).collect_vec();
    let qrow = expr.qrow.into_iter().map(|v| v.index()).collect_vec();
    let qcol = expr.qcol.into_iter().map(|v| v.index()).collect_vec();

    try!(self.del_qpterms());
    try!(self.update());

    try!(self.set_list(attr::exports::Obj, lind.as_slice(), expr.lval.as_slice()));
    try!(self.add_qpterms(qrow.as_slice(), qcol.as_slice(), expr.qval.as_slice()));

    self.set(attr::exports::ModelSense, sense.into())
  }

  /// Query the value of attributes which associated with variable/constraints.
  pub fn get<A: attr::AttrBase>(&self, attr: A) -> Result<A::Out> {
    let mut value: A::Buf = util::Init::init();

    try!(self.check_apicall(unsafe {
      use util::AsRawPtr;
      A::get_attr(self.model, attr.into().as_ptr(), value.as_rawptr())
    }));

    Ok(util::Into::into(value))
  }

  /// Set the value of attributes which associated with variable/constraints.
  pub fn set<A: attr::AttrBase>(&mut self, attr: A, value: A::Out) -> Result<()> {
    self.check_apicall(unsafe { A::set_attr(self.model, attr.into().as_ptr(), util::From::from(value)) })
  }


  fn get_element<A: attr::AttrArrayBase>(&self, attr: A, element: i32) -> Result<A::Out> {
    let mut value: A::Buf = util::Init::init();

    try!(self.check_apicall(unsafe {
      use util::AsRawPtr;
      A::get_attrelement(self.model, attr.into().as_ptr(), element, value.as_rawptr())
    }));

    Ok(util::Into::into(value))
  }

  fn set_element<A: attr::AttrArrayBase>(&mut self, attr: A, element: i32, value: A::Out) -> Result<()> {
    self.check_apicall(unsafe {
      A::set_attrelement(self.model,
                         attr.into().as_ptr(),
                         element,
                         util::From::from(value))
    })
  }

  /// Query the value of attributes which associated with variable/constraints.
  pub fn get_values<A: attr::AttrArrayBase, P: Proxy>(&self, attr: A, item: &[P]) -> Result<Vec<A::Out>> {
    self.get_list(attr,
                  item.iter().map(|e| e.index()).collect_vec().as_slice())
  }

  fn get_list<A: attr::AttrArrayBase>(&self, attr: A, ind: &[i32]) -> Result<Vec<A::Out>> {
    let mut values: Vec<_> = iter::repeat(util::Init::init()).take(ind.len()).collect();

    try!(self.check_apicall(unsafe {
      A::get_attrlist(self.model,
                      attr.into().as_ptr(),
                      ind.len() as ffi::c_int,
                      ind.as_ptr(),
                      values.as_mut_ptr())
    }));

    Ok(values.into_iter().map(|s| util::Into::into(s)).collect())
  }


  /// Set the value of attributes which associated with variable/constraints.
  pub fn set_values<A: attr::AttrArrayBase, P: Proxy>(&mut self, attr: A, item: &[P], val: &[A::Out]) -> Result<()> {
    self.set_list(attr,
                  item.iter().map(|e| e.index()).collect_vec().as_slice(),
                  val)
  }

  fn set_list<A: attr::AttrArrayBase>(&mut self, attr: A, ind: &[i32], values: &[A::Out]) -> Result<()> {
    if ind.len() != values.len() {
      return Err(Error::InconsitentDims);
    }

    let values = try!(A::to_rawsets(values));

    self.check_apicall(unsafe {
      A::set_attrlist(self.model,
                      attr.into().as_ptr(),
                      ind.len() as ffi::c_int,
                      ind.as_ptr(),
                      values.as_ptr())
    })
  }

  /// Modify the model to create a feasibility relaxation.
  ///
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
  /// * Slack variables for relaxation and linear/quadratic constraints related to theirs.
  pub fn feas_relax(&mut self, relaxtype: RelaxType, minrelax: bool, vars: &[Var], lbpen: &[f64], ubpen: &[f64],
                    constrs: &[Constr], rhspen: &[f64])
                    -> Result<(f64, Iter<Var>, Iter<Constr>, Iter<QConstr>)> {
    if vars.len() != lbpen.len() || vars.len() != ubpen.len() {
      return Err(Error::InconsitentDims);
    }

    if constrs.len() != rhspen.len() {
      return Err(Error::InconsitentDims);
    }

    let mut pen_lb = vec![super::INFINITY; self.vars.len()];
    let mut pen_ub = vec![super::INFINITY; self.vars.len()];
    for (ref v, &lb, &ub) in Zip::new((vars, lbpen, ubpen)) {
      let idx = v.index();
      if idx >= self.vars.len() as i32 {
        return Err(Error::InconsitentDims);
      }
      pen_lb[idx as usize] = lb;
      pen_ub[idx as usize] = ub;
    }

    let mut pen_rhs = vec![super::INFINITY; self.constrs.len()];
    for (ref c, &rhs) in Zip::new((constrs, rhspen)) {
      let idx = c.index();
      if idx >= self.constrs.len() as i32 {
        return Err(Error::InconsitentDims);
      }

      pen_rhs[idx as usize] = rhs;
    }

    let minrelax = if minrelax { 1 } else { 0 };

    let mut feasobj = 0f64;
    try!(self.check_apicall(unsafe {
      ffi::GRBfeasrelax(self.model,
                        relaxtype.into(),
                        minrelax,
                        pen_lb.as_ptr(),
                        pen_ub.as_ptr(),
                        pen_rhs.as_ptr(),
                        &mut feasobj)
    }));

    let cols = try!(self.get(attr::exports::NumVars)) as usize;
    let rows = try!(self.get(attr::exports::NumConstrs)) as usize;
    let qrows = try!(self.get(attr::exports::NumQConstrs)) as usize;

    let xcols = self.vars.len();
    let xrows = self.constrs.len();
    let xqrows = self.qconstrs.len();

    self.vars.extend((xcols..cols).map(|idx| Var::new(idx as i32)));
    self.constrs.extend((xrows..rows).map(|idx| Constr::new(idx as i32)));
    self.qconstrs.extend((xqrows..qrows).map(|idx| QConstr::new(idx as i32)));

    Ok((feasobj, self.vars[xcols..].iter(), self.constrs[xrows..].iter(), self.qconstrs[xqrows..].iter()))
  }

  /// Set a piecewise-linear objective function of a certain variable in the model.
  pub fn set_pwl_obj(&mut self, var: &Var, x: &[f64], y: &[f64]) -> Result<()> {
    if x.len() != y.len() {
      return Err(Error::InconsitentDims);
    }
    self.check_apicall(unsafe {
      ffi::GRBsetpwlobj(self.model,
                        var.index(),
                        x.len() as ffi::c_int,
                        x.as_ptr(),
                        y.as_ptr())
    })
  }

  /// Retrieve the status of the model.
  pub fn status(&self) -> Result<Status> { self.get(attr::exports::Status).map(|val| val.into()) }

  /// Retrieve an iterator of the variables in the model.
  pub fn get_vars(&self) -> Iter<Var> { self.vars.iter() }

  /// Retrieve an iterator of the linear constraints in the model.
  pub fn get_constrs(&self) -> Iter<Constr> { self.constrs.iter() }

  /// Retrieve an iterator of the quadratic constraints in the model.
  pub fn get_qconstrs(&self) -> Iter<QConstr> { self.qconstrs.iter() }

  /// Retrieve an iterator of the special order set (SOS) constraints in the model.
  pub fn get_sos(&self) -> Iter<SOS> { self.sos.iter() }

  /// Remove a variable from the model.
  pub fn remove_var(&mut self, mut item: Var) -> Result<()> {
    let index = item.index();
    if index >= self.vars.len() as i32 {
      return Err(Error::InconsitentDims);
    }

    if index != -1 {
      try!(self.check_apicall(unsafe { ffi::GRBdelvars(self.model, 1, &index) }));

      self.vars.remove(index as usize);
      item.set_index(-1);

      // reset all of the remaining items.
      for (idx, ref mut v) in self.vars.iter_mut().enumerate().skip(index as usize) {
        v.set_index(idx as i32);
      }
    }
    Ok(())
  }

  /// Remove a linear constraint from the model.
  pub fn remove_constr(&mut self, mut item: Constr) -> Result<()> {
    let index = item.index();
    if index >= self.constrs.len() as i32 {
      return Err(Error::InconsitentDims);
    }

    if index != -1 {
      try!(self.check_apicall(unsafe { ffi::GRBdelconstrs(self.model, 1, &index) }));

      self.constrs.remove(index as usize);
      item.set_index(-1);

      // reset all of the remaining items.
      for (idx, ref mut c) in self.constrs.iter_mut().enumerate().skip(index as usize) {
        c.set_index(idx as i32);
      }
    }
    Ok(())
  }

  /// Remove a quadratic constraint from the model.
  pub fn remove_qconstr(&mut self, mut item: QConstr) -> Result<()> {
    let index = item.index();
    if index >= self.qconstrs.len() as i32 {
      return Err(Error::InconsitentDims);
    }

    if index != -1 {
      try!(self.check_apicall(unsafe { ffi::GRBdelqconstrs(self.model, 1, &index) }));

      self.qconstrs.remove(index as usize);
      item.set_index(-1);

      // reset all of the remaining items.
      for (idx, ref mut qc) in self.qconstrs.iter_mut().enumerate().skip(index as usize) {
        qc.set_index(idx as i32);
      }
    }
    Ok(())
  }

  /// Remove a special order set (SOS) cnstraint from the model.
  pub fn remove_sos(&mut self, mut item: SOS) -> Result<()> {
    let index = item.index();
    if index >= self.sos.len() as i32 {
      return Err(Error::InconsitentDims);
    }

    if index != -1 {
      try!(self.check_apicall(unsafe { ffi::GRBdelsos(self.model, 1, &index) }));

      self.sos.remove(index as usize);
      item.set_index(-1);

      // reset all of the remaining items.
      for (idx, ref mut s) in self.sos.iter_mut().enumerate().skip(index as usize) {
        s.set_index(idx as i32);
      }
    }
    Ok(())
  }

  pub fn set_callback(&mut self, callback: Callback) -> Result<()> {
    try!(self.check_apicall(unsafe { ffi::GRBsetcallbackfunc(self.model, callback_wrapper, transmute(&self)) }));
    self.callback = Some(callback);
    Ok(())
  }

  pub fn reset_callback(&mut self) -> Result<()> {
    try!(self.check_apicall(unsafe { ffi::GRBsetcallbackfunc(self.model, callback_wrapper, null_mut()) }));
    self.callback = None;
    Ok(())
  }

  /// Retrieve a single constant matrix coefficient of the model.
  pub fn get_coeff(&self, var: &Var, constr: &Constr) -> Result<f64> {
    let mut value = 0.0;
    try!(self.check_apicall(unsafe { ffi::GRBgetcoeff(self.model, var.index(), constr.index(), &mut value) }));
    Ok(value)
  }

  /// Change a single constant matrix coefficient of the model.
  pub fn set_coeff(&mut self, var: &Var, constr: &Constr, value: f64) -> Result<()> {
    self.check_apicall(unsafe { ffi::GRBchgcoeffs(self.model, 1, &var.index(), &constr.index(), &value) })
  }

  /// Change a set of constant matrix coefficients of the model.
  pub fn set_coeffs(&mut self, vars: &[&Var], constrs: &[&Constr], values: &[f64]) -> Result<()> {
    if vars.len() != values.len() || constrs.len() != values.len() {
      return Err(Error::InconsitentDims);
    }

    let vars = vars.iter().map(|v| v.index()).collect_vec();
    let constrs = constrs.iter().map(|c| c.index()).collect_vec();

    self.check_apicall(unsafe {
      ffi::GRBchgcoeffs(self.model,
                        vars.len() as ffi::c_int,
                        vars.as_ptr(),
                        constrs.as_ptr(),
                        values.as_ptr())
    })
  }

  fn populate(&mut self) -> Result<()> {
    self.callback = None;

    let cols = try!(self.get(attr::exports::NumVars)) as usize;
    let rows = try!(self.get(attr::exports::NumConstrs)) as usize;
    let numqconstrs = try!(self.get(attr::exports::NumQConstrs)) as usize;
    let numsos = try!(self.get(attr::exports::NumSOS)) as usize;

    self.vars = (0..cols).map(|idx| Var::new(idx as i32)).collect_vec();
    self.constrs = (0..rows).map(|idx| Constr::new(idx as i32)).collect_vec();
    self.qconstrs = (0..numqconstrs).map(|idx| QConstr::new(idx as i32)).collect_vec();
    self.sos = (0..numsos).map(|idx| SOS::new(idx as i32)).collect_vec();

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
    })
  }

  // remove quadratic terms of objective function.
  fn del_qpterms(&mut self) -> Result<()> { self.check_apicall(unsafe { ffi::GRBdelq(self.model) }) }

  fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
    if error != 0 {
      return Err(self.env.error_from_api(error));
    }
    Ok(())
  }
}


impl<'a> Drop for Model<'a> {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreemodel(self.model) };
    self.model = null_mut();
  }
}

#[test]
fn removing_variable_should_be_successed() {
  use super::*;
  let mut env = Env::new("").unwrap();
  env.set(param::OutputFlag, 0).unwrap();
  let mut model = env.new_model("hoge").unwrap();

  let x = model.add_var("x", Binary).unwrap();
  let y = model.add_var("y", Binary).unwrap();
  let z = model.add_var("z", Binary).unwrap();
  model.update().unwrap();

  model.remove_var(y.clone()).unwrap();
  model.update().unwrap();

  assert_eq!(model.get(attr::NumVars).unwrap(), 2);

  assert_eq!(x.index(), 0);
  assert_eq!(y.index(), -1);
  assert_eq!(z.index(), 1);
}
