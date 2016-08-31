// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

pub mod callback;
pub mod expr;

use ffi;
use itertools::{Itertools, Zip};

use std::cell::Cell;
use std::ffi::CString;
use std::iter;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::slice::Iter;

use attr;
use attribute::{Attr, AttrArray};
use self::callback::{Callback, New};
use self::expr::{LinExpr, QuadExpr};
use env::{Env, EnvAPI};
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
#[derive(Debug,Copy,Clone)]
pub enum ModelSense {
  Minimize = 1,
  Maximize = -1
}

impl Into<i32> for ModelSense {
  fn into(self) -> i32 { (unsafe { transmute::<_, i8>(self) }) as i32 }
}


/// Type of new SOS constraint
#[derive(Debug,Copy,Clone)]
pub enum SOSType {
  SOSType1 = 1,
  SOSType2 = 2
}

impl Into<i32> for SOSType {
  fn into(self) -> i32 { (unsafe { transmute::<_, i8>(self) }) as i32 }
}


/// Status of a model
#[derive(Debug,Copy,Clone,PartialEq)]
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

/// Type of cost function at feasibility relaxation
#[derive(Debug,Copy,Clone)]
pub enum RelaxType {
  /// The weighted magnitude of bounds and constraint violations
  /// (`penalty(s_i) = w_i s_i`)
  Linear = 0,

  /// The weighted square of magnitude of bounds and constraint violations
  /// (`penalty(s_i) = w_i s_i^2`)
  Quadratic = 1,

  /// The weighted count of bounds and constraint violations
  /// (`penalty(s_i) = w_i * [s_i > 0]`)
  Cardinality = 2
}

impl Into<i32> for RelaxType {
  fn into(self) -> i32 { (unsafe { transmute::<_, i8>(self) }) as i32 }
}


/// Provides methods to query/modify attributes associated with certain element.
#[derive(Clone)]
pub struct Proxy(Rc<Cell<i32>>);

impl Proxy {
  fn new(idx: i32) -> Proxy { Proxy(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }

  /// Query the value of attribute.
  pub fn get<A: AttrArray>(&self, model: &Model, attr: A) -> Result<A::Out> { model.get_element(attr, self.index()) }

  /// Set the value of attribute.
  pub fn set<A: AttrArray>(&self, model: &mut Model, attr: A, val: A::Out) -> Result<()> {
    model.set_element(attr, self.index(), val)
  }
}

impl PartialEq for Proxy {
  fn eq(&self, other: &Proxy) -> bool { self.0.as_ref() as *const Cell<i32> == other.0.as_ref() as *const Cell<i32> }
}


macro_rules! impl_traits_for_proxy {
  {$($t:ident)*} => { $(
    impl $t {
      fn new(idx: i32) -> $t { $t(Proxy::new(idx)) }
    }

    impl Deref for $t {
      type Target = Proxy;
      fn deref(&self) -> &Proxy { &self.0 }
    }

    impl DerefMut for $t {
      fn deref_mut(&mut self) -> &mut Proxy { &mut self.0 }
    }

    impl PartialEq for $t {
      fn eq(&self, other:&$t) -> bool { self.0.eq(&other.0) }
    }
  )* }
}


/// Proxy object of a variables
#[derive(Clone)]
pub struct Var(Proxy);

impl Var {
  pub fn get_type(&self, model: &Model) -> Result<(char, f64, f64)> {
    let lb = try!(self.get(&model, attr::LB));
    let ub = try!(self.get(&model, attr::UB));
    let vtype = try!(self.get(&model, attr::VType));
    let vtype = unsafe { transmute::<_, u8>(vtype) } as char;
    Ok((vtype, lb, ub))
  }
}

/// Proxy object of a linear constraint
#[derive(Clone)]
pub struct Constr(Proxy);

/// Proxy object of a quadratic constraint
#[derive(Clone)]
pub struct QConstr(Proxy);

/// Proxy object of a Special Order Set (SOS) constraint
#[derive(Clone)]
pub struct SOS(Proxy);

impl_traits_for_proxy! { Var Constr QConstr SOS }



struct CallbackData<'a> {
  model: &'a Model,
  callback: &'a mut FnMut(Callback) -> Result<()>
}

#[allow(unused_variables)]
extern "C" fn callback_wrapper(model: *mut ffi::GRBmodel, cbdata: *mut ffi::c_void, loc: ffi::c_int,
                               usrdata: *mut ffi::c_void)
                               -> ffi::c_int {

  let mut usrdata = unsafe { transmute::<_, &mut CallbackData>(usrdata) };
  let (callback, model) = (&mut usrdata.callback, &usrdata.model);

  match Callback::new(cbdata, loc.into(), model) {
    Err(err) => {
      println!("failed to create context: {:?}", err);
      -3
    }
    Ok(context) => {
      match catch_unwind(AssertUnwindSafe(|| if callback(context).is_ok() { 0 } else { -1 })) {
        Ok(ret) => ret,
        Err(e) => -3000,
      }
    }
  }
}

#[allow(unused_variables)]
extern "C" fn null_callback_wrapper(model: *mut ffi::GRBmodel, cbdata: *mut ffi::c_void, loc: ffi::c_int,
                                    usrdata: *mut ffi::c_void)
                                    -> ffi::c_int {
  0
}


/// Gurobi model object associated with certain environment.
pub struct Model {
  model: *mut ffi::GRBmodel,
  env: Env,
  vars: Vec<Var>,
  constrs: Vec<Constr>,
  qconstrs: Vec<QConstr>,
  sos: Vec<SOS>
}

pub trait FromRaw {
  /// create an empty model which associated with certain environment.
  fn from_raw(model: *mut ffi::GRBmodel) -> Result<Model>;
}

impl FromRaw for Model {
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
      model: model,
      env: env,
      vars: Vec::new(),
      constrs: Vec::new(),
      qconstrs: Vec::new(),
      sos: Vec::new()
    };
    try!(model.populate());
    Ok(model)
  }
}

impl Model {
  /// Create an empty Gurobi model from the environment.
  ///
  /// Note that all of the parameters in given environment will copy by Gurobi API
  /// and a new environment associated with the model will create.
  /// If you want to query/modify the value of parameters, use `get_env()`.
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
  /// // ...
  ///
  /// let model = Model::new("model1", &env).unwrap();
  /// ```
  pub fn new(modelname: &str, env: &Env) -> Result<Model> {
    let modelname = try!(CString::new(modelname));
    let mut model = null_mut();
    try!(env.check_apicall(unsafe {
      ffi::GRBnewmodel(env.get_ptr(),
                       &mut model,
                       modelname.as_ptr(),
                       0,
                       null(),
                       null(),
                       null(),
                       null(),
                       null())
    }));
    Self::from_raw(model)
  }

  /// Read a model from a file
  pub fn read_from(filename: &str, env: &Env) -> Result<Model> {
    let filename = try!(CString::new(filename));
    let mut model = null_mut();
    try!(env.check_apicall(unsafe { ffi::GRBreadmodel(env.get_ptr(), filename.as_ptr(), &mut model) }));
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

  /// Optimize the model with a callback function
  pub fn optimize_with_callback<F>(&mut self, mut callback: F) -> Result<()>
    where F: FnMut(Callback) -> Result<()> + 'static
  {
    let usrdata = CallbackData {
      model: self,
      callback: &mut callback
    };
    try!(self.check_apicall(unsafe { ffi::GRBsetcallbackfunc(self.model, callback_wrapper, transmute(&usrdata)) }));

    try!(self.check_apicall(unsafe { ffi::GRBoptimize(self.model) }));

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
    try!(self.update());

    let col_no = self.vars.len() as i32;
    self.vars.push(Var::new(col_no));

    Ok(self.vars.last().cloned().unwrap())
  }

  /// add decision variables to the model.
  pub fn add_vars(&mut self, names: &[&str], vtypes: &[VarType]) -> Result<Vec<Var>> {
    if names.len() != vtypes.len() {
      return Err(Error::InconsitentDims);
    }

    let mut _vtypes = Vec::with_capacity(vtypes.len());
    let mut _names = Vec::with_capacity(vtypes.len());
    let mut lbs = Vec::with_capacity(vtypes.len());
    let mut ubs = Vec::with_capacity(vtypes.len());
    for (&name, &vtype) in Zip::new((names, vtypes)) {
      let (vtype, lb, ub) = vtype.into();
      let name = try!(CString::new(name));
      _vtypes.push(vtype);
      _names.push(name.as_ptr());
      lbs.push(lb);
      ubs.push(ub);
    }
    let objs = vec![0.0; vtypes.len()];

    try!(self.check_apicall(unsafe {
      ffi::GRBaddvars(self.model,
                      _names.len() as ffi::c_int,
                      0,
                      null(),
                      null(),
                      null(),
                      objs.as_ptr(),
                      lbs.as_ptr(),
                      ubs.as_ptr(),
                      _vtypes.as_ptr(),
                      _names.as_ptr())
    }));
    try!(self.update());

    let xcols = self.vars.len();
    let cols = self.vars.len() + _names.len();
    for col_no in xcols..cols {
      self.vars.push(Var::new(col_no as i32));
    }

    Ok(self.vars[xcols..].iter().cloned().collect_vec())
  }


  /// add a linear constraint to the model.
  pub fn add_constr(&mut self, name: &str, expr: LinExpr, sense: ConstrSense, rhs: f64) -> Result<Constr> {
    let constrname = try!(CString::new(name));
    let (vars, coeff, offset) = expr.into();
    try!(self.check_apicall(unsafe {
      ffi::GRBaddconstr(self.model,
                        coeff.len() as ffi::c_int,
                        vars.as_ptr(),
                        coeff.as_ptr(),
                        sense.into(),
                        rhs - offset,
                        constrname.as_ptr())
    }));
    try!(self.update());

    let row_no = self.constrs.len() as i32;
    self.constrs.push(Constr::new(row_no));

    Ok(self.constrs.last().cloned().unwrap())
  }

  /// add linear constraints to the model.
  pub fn add_constrs(&mut self, name: &[&str], expr: &[LinExpr], sense: &[ConstrSense], rhs: &[f64])
                     -> Result<Vec<Constr>> {
    let mut constrnames = Vec::with_capacity(name.len());
    for &s in name.iter() {
      let name = try!(CString::new(s));
      constrnames.push(name.as_ptr());
    }

    let expr: Vec<(_, _, _)> = expr.into_iter().cloned().map(|e| e.into()).collect_vec();

    let sense = sense.iter().map(|&s| s.into()).collect_vec();
    let rhs = Zip::new((rhs, &expr)).map(|(rhs, expr)| rhs - expr.2).collect_vec();

    let mut beg = Vec::with_capacity(expr.len());

    let numnz = expr.iter().map(|expr| expr.0.len()).sum();
    let mut ind = Vec::with_capacity(numnz);
    let mut val = Vec::with_capacity(numnz);

    for expr in expr.iter() {
      let nz = ind.len();
      beg.push(nz as i32);
      ind.extend(&expr.0);
      val.extend(&expr.1);
    }

    try!(self.check_apicall(unsafe {
      ffi::GRBaddconstrs(self.model,
                         constrnames.len() as ffi::c_int,
                         beg.len() as ffi::c_int,
                         beg.as_ptr(),
                         ind.as_ptr(),
                         val.as_ptr(),
                         sense.as_ptr(),
                         rhs.as_ptr(),
                         constrnames.as_ptr())
    }));
    try!(self.update());

    let xrows = self.constrs.len();
    let rows = self.constrs.len() + constrnames.len();
    for row_no in xrows..rows {
      self.constrs.push(Constr::new(row_no as i32));
    }

    Ok(self.constrs[xrows..].iter().cloned().collect_vec())
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
    let constrname = try!(CString::new(name));
    let (vars, coeff, offset) = expr.into();
    try!(self.check_apicall(unsafe {
      ffi::GRBaddrangeconstr(self.model,
                             coeff.len() as ffi::c_int,
                             vars.as_ptr(),
                             coeff.as_ptr(),
                             lb - offset,
                             ub - offset,
                             constrname.as_ptr())
    }));
    try!(self.update());

    let col_no = self.vars.len() as i32;
    self.vars.push(Var::new(col_no));

    let row_no = self.constrs.len() as i32;
    self.constrs.push(Constr::new(row_no));

    Ok((self.vars.last().cloned().unwrap(), self.constrs.last().cloned().unwrap()))
  }

  /// Add range constraints to the model.
  pub fn add_ranges(&mut self, names: &[&str], expr: &[LinExpr], lb: &[f64], ub: &[f64])
                    -> Result<(Vec<Var>, Vec<Constr>)> {

    let mut constrnames = Vec::with_capacity(names.len());
    for &s in names.iter() {
      let name = try!(CString::new(s));
      constrnames.push(name.as_ptr());
    }

    let expr: Vec<(_, _, _)> = expr.into_iter().cloned().map(|e| e.into()).collect_vec();

    let lhs = Zip::new((lb, &expr)).map(|(lb, expr)| lb - expr.2).collect_vec();
    let rhs = Zip::new((ub, &expr)).map(|(ub, expr)| ub - expr.2).collect_vec();

    let mut beg = Vec::with_capacity(expr.len());

    let numnz = expr.iter().map(|expr| expr.0.len()).sum();
    let mut ind = Vec::with_capacity(numnz);
    let mut val = Vec::with_capacity(numnz);

    for expr in expr.iter() {
      let nz = ind.len();
      beg.push(nz as i32);
      ind.extend(&expr.0);
      val.extend(&expr.1);
    }

    try!(self.check_apicall(unsafe {
      ffi::GRBaddrangeconstrs(self.model,
                              constrnames.len() as ffi::c_int,
                              beg.len() as ffi::c_int,
                              beg.as_ptr(),
                              ind.as_ptr(),
                              val.as_ptr(),
                              lhs.as_ptr(),
                              rhs.as_ptr(),
                              constrnames.as_ptr())
    }));
    try!(self.update());

    let xcols = self.vars.len();
    let cols = self.vars.len() + names.len();
    for col_no in xcols..cols {
      self.vars.push(Var::new(col_no as i32));
    }

    let xrows = self.constrs.len();
    let rows = self.constrs.len() + constrnames.len();
    for row_no in xrows..rows {
      self.constrs.push(Constr::new(row_no as i32));
    }

    Ok((self.vars[xcols..].iter().cloned().collect_vec(), self.constrs[xrows..].iter().cloned().collect_vec()))
  }

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self, constrname: &str, expr: QuadExpr, sense: ConstrSense, rhs: f64) -> Result<QConstr> {
    let constrname = try!(CString::new(constrname));
    let (lind, lval, qrow, qcol, qval, offset) = expr.into();
    try!(self.check_apicall(unsafe {
      ffi::GRBaddqconstr(self.model,
                         lval.len() as ffi::c_int,
                         lind.as_ptr(),
                         lval.as_ptr(),
                         qval.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr(),
                         sense.into(),
                         rhs - offset,
                         constrname.as_ptr())
    }));
    try!(self.update());

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
    try!(self.update());

    let sos_no = self.sos.len() as i32;
    self.sos.push(SOS::new(sos_no));

    Ok(self.sos.last().cloned().unwrap())
  }

  /// Set the objective function of the model.
  pub fn set_objective<Expr: Into<QuadExpr>>(&mut self, expr: Expr, sense: ModelSense) -> Result<()> {
    let (lind, lval, qrow, qcol, qval, _) = Into::<QuadExpr>::into(expr).into();
    try!(self.del_qpterms());
    try!(self.add_qpterms(qrow.as_slice(), qcol.as_slice(), qval.as_slice()));

    try!(self.set_list(attr::Obj, lind.as_slice(), lval.as_slice()));

    self.set(attr::ModelSense, sense.into())
  }

  /// Query the value of attributes which associated with variable/constraints.
  pub fn get<A: Attr>(&self, attr: A) -> Result<A::Out> {
    let mut value: A::Buf = util::Init::init();

    try!(self.check_apicall(unsafe {
      use util::AsRawPtr;
      A::get_attr(self.model, attr.into().as_ptr(), value.as_rawptr())
    }));

    Ok(util::Into::into(value))
  }

  /// Set the value of attributes which associated with variable/constraints.
  pub fn set<A: Attr>(&mut self, attr: A, value: A::Out) -> Result<()> {
    try!(self.check_apicall(unsafe { A::set_attr(self.model, attr.into().as_ptr(), util::From::from(value)) }));
    self.update()
  }


  fn get_element<A: AttrArray>(&self, attr: A, element: i32) -> Result<A::Out> {
    let mut value: A::Buf = util::Init::init();

    try!(self.check_apicall(unsafe {
      use util::AsRawPtr;
      A::get_attrelement(self.model, attr.into().as_ptr(), element, value.as_rawptr())
    }));

    Ok(util::Into::into(value))
  }

  fn set_element<A: AttrArray>(&mut self, attr: A, element: i32, value: A::Out) -> Result<()> {
    try!(self.check_apicall(unsafe {
      A::set_attrelement(self.model,
                         attr.into().as_ptr(),
                         element,
                         util::From::from(value))
    }));
    self.update()
  }

  /// Query the value of attributes which associated with variable/constraints.
  pub fn get_values<A: AttrArray, P>(&self, attr: A, item: &[P]) -> Result<Vec<A::Out>>
    where P: Deref<Target = Proxy>
  {
    self.get_list(attr,
                  item.iter().map(|e| e.index()).collect_vec().as_slice())
  }

  fn get_list<A: AttrArray>(&self, attr: A, ind: &[i32]) -> Result<Vec<A::Out>> {
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
  pub fn set_values<A: AttrArray, P>(&mut self, attr: A, item: &[P], val: &[A::Out]) -> Result<()>
    where P: Deref<Target = Proxy>
  {
    try!(self.set_list(attr,
                       item.iter().map(|e| e.index()).collect_vec().as_slice(),
                       val));
    self.update()
  }

  fn set_list<A: AttrArray>(&mut self, attr: A, ind: &[i32], values: &[A::Out]) -> Result<()> {
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
    try!(self.update());

    let cols = try!(self.get(attr::NumVars)) as usize;
    let rows = try!(self.get(attr::NumConstrs)) as usize;
    let qrows = try!(self.get(attr::NumQConstrs)) as usize;

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
    try!(self.check_apicall(unsafe {
      ffi::GRBsetpwlobj(self.model,
                        var.index(),
                        x.len() as ffi::c_int,
                        x.as_ptr(),
                        y.as_ptr())
    }));
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

  /// Remove a variable from the model.
  pub fn remove_var(&mut self, mut item: Var) -> Result<()> {
    let index = item.index();
    if index >= self.vars.len() as i32 {
      return Err(Error::InconsitentDims);
    }

    if index != -1 {
      try!(self.check_apicall(unsafe { ffi::GRBdelvars(self.model, 1, &index) }));
      try!(self.update());

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
      try!(self.update());

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
      try!(self.update());

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
      try!(self.update());

      self.sos.remove(index as usize);
      item.set_index(-1);

      // reset all of the remaining items.
      for (idx, ref mut s) in self.sos.iter_mut().enumerate().skip(index as usize) {
        s.set_index(idx as i32);
      }
    }
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
    try!(self.check_apicall(unsafe { ffi::GRBchgcoeffs(self.model, 1, &var.index(), &constr.index(), &value) }));
    self.update()
  }

  /// Change a set of constant matrix coefficients of the model.
  pub fn set_coeffs(&mut self, vars: &[&Var], constrs: &[&Constr], values: &[f64]) -> Result<()> {
    if vars.len() != values.len() || constrs.len() != values.len() {
      return Err(Error::InconsitentDims);
    }

    let vars = vars.iter().map(|v| v.index()).collect_vec();
    let constrs = constrs.iter().map(|c| c.index()).collect_vec();

    try!(self.check_apicall(unsafe {
      ffi::GRBchgcoeffs(self.model,
                        vars.len() as ffi::c_int,
                        vars.as_ptr(),
                        constrs.as_ptr(),
                        values.as_ptr())
    }));
    self.update()
  }

  fn populate(&mut self) -> Result<()> {
    let cols = try!(self.get(attr::NumVars)) as usize;
    let rows = try!(self.get(attr::NumConstrs)) as usize;
    let numqconstrs = try!(self.get(attr::NumQConstrs)) as usize;
    let numsos = try!(self.get(attr::NumSOS)) as usize;

    self.vars = (0..cols).map(|idx| Var::new(idx as i32)).collect_vec();
    self.constrs = (0..rows).map(|idx| Constr::new(idx as i32)).collect_vec();
    self.qconstrs = (0..numqconstrs).map(|idx| QConstr::new(idx as i32)).collect_vec();
    self.sos = (0..numsos).map(|idx| SOS::new(idx as i32)).collect_vec();

    Ok(())
  }


  // add quadratic terms of objective function.
  fn add_qpterms(&mut self, qrow: &[i32], qcol: &[i32], qval: &[f64]) -> Result<()> {
    try!(self.check_apicall(unsafe {
      ffi::GRBaddqpterms(self.model,
                         qrow.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr())
    }));
    self.update()
  }

  // remove quadratic terms of objective function.
  fn del_qpterms(&mut self) -> Result<()> {
    try!(self.check_apicall(unsafe { ffi::GRBdelq(self.model) }));
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
