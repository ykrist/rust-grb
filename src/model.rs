extern crate gurobi_sys as ffi;

use std::iter;
use std::ffi::CString;
use std::ptr::{null, null_mut};
use std::ops::{Add, Sub, Mul};

use env::Env;
use error::{Error, Result};
use util;
use types;


pub mod attr {
  pub use ffi::{IntAttr, DoubleAttr, CharAttr, StringAttr};

  pub use self::IntAttr::*;
  pub use self::DoubleAttr::*;
  pub use self::CharAttr::*;
  pub use self::StringAttr::*;
}


/// The type for new variable(s).
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


///
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


///
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


///
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

/// It represents a set of decision variables.
#[derive(Clone)]
pub struct Variable<S: Shape>(Vec<i32>, S);

#[derive(Clone)]
pub struct Constr<S: Shape>(Vec<i32>, S);

#[derive(Clone)]
pub struct QConstr<S: Shape>(Vec<i32>, S);


impl<S: Shape> Variable<S> {
  pub fn shape(&self) -> Option<S> { Some(self.1) }
}

/// It represents a set of linear expressions of decision variables.
pub struct LinExpr<S: Shape> {
  vars: Vec<Variable<S>>,
  coeff: Vec<f64>,
  offset: f64
}


impl<S: Shape> LinExpr<S> {
  pub fn new() -> Self {
    LinExpr {
      vars: Vec::new(),
      coeff: Vec::new(),
      offset: 0.0
    }
  }

  pub fn term(mut self, v: Variable<S>, c: f64) -> Self {
    self.vars.push(v);
    self.coeff.push(c);
    self
  }

  pub fn offset(mut self, offset: f64) -> Self {
    self.offset += offset;
    self
  }

  /// Get the shape of expression.
  pub fn shape(&self) -> Option<S> { self.vars.get(0).map(|v| v.1) }
}

impl<S: Shape> Into<QuadExpr<S>> for LinExpr<S> {
  fn into(self) -> QuadExpr<S> {
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



pub struct QuadExpr<S: Shape> {
  lind: Vec<Variable<S>>,
  lval: Vec<f64>,
  qrow: Vec<Variable<S>>,
  qcol: Vec<Variable<S>>,
  qval: Vec<f64>,
  offset: f64
}

impl<S: Shape> QuadExpr<S> {
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

  pub fn term(mut self, var: Variable<S>, coeff: f64) -> Self {
    self.lind.push(var);
    self.lval.push(coeff);
    self
  }

  pub fn qterm(mut self, row: Variable<S>, col: Variable<S>, coeff: f64) -> Self {
    self.qrow.push(row);
    self.qcol.push(col);
    self.qval.push(coeff);
    self
  }

  pub fn offset(mut self, offset: f64) -> Self {
    self.offset += offset;
    self
  }

  /// Get the shape of expression.
  pub fn shape(&self) -> Option<S> { self.lind.get(0).map(|v| v.1) }
}


impl<S: Shape> Mul<f64> for Variable<S> {
  type Output = LinExpr<S>;
  fn mul(self, rhs: f64) -> Self::Output { LinExpr::new().term(self, rhs) }
}

impl<S: Shape> Mul<Variable<S>> for f64 {
  type Output = LinExpr<S>;
  fn mul(self, rhs: Variable<S>) -> Self::Output { LinExpr::new().term(rhs, self) }
}


impl<S: Shape> Mul for Variable<S> {
  type Output = QuadExpr<S>;
  fn mul(self, rhs: Variable<S>) -> Self::Output { QuadExpr::new().qterm(self, rhs, 1.0) }
}

impl<S: Shape> Mul<f64> for QuadExpr<S> {
  type Output = QuadExpr<S>;
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


impl<S: Shape> Add<f64> for LinExpr<S> {
  type Output = LinExpr<S>;
  fn add(self, rhs: f64) -> Self::Output { self.offset(rhs) }
}

impl<S: Shape> Sub<f64> for LinExpr<S> {
  type Output = LinExpr<S>;
  fn sub(self, rhs: f64) -> Self::Output { self.offset(-rhs) }
}


impl<S: Shape> Add for LinExpr<S> {
  type Output = LinExpr<S>;
  fn add(mut self, rhs: LinExpr<S>) -> Self::Output {
    self.vars.extend(rhs.vars);
    self.coeff.extend(rhs.coeff);
    self.offset += rhs.offset;
    self
  }
}

impl<S: Shape> Sub for LinExpr<S> {
  type Output = LinExpr<S>;
  fn sub(mut self, rhs: LinExpr<S>) -> Self::Output {
    self.vars.extend(rhs.vars);
    self.coeff.extend(rhs.coeff.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}


impl<S: Shape> Add<LinExpr<S>> for QuadExpr<S> {
  type Output = QuadExpr<S>;
  fn add(mut self, rhs: LinExpr<S>) -> Self::Output {
    self.lind.extend(rhs.vars);
    self.lval.extend(rhs.coeff);
    self.offset += rhs.offset;
    self
  }
}

impl<S: Shape> Sub<LinExpr<S>> for QuadExpr<S> {
  type Output = QuadExpr<S>;
  fn sub(mut self, rhs: LinExpr<S>) -> Self::Output {
    self.lind.extend(rhs.vars);
    self.lval.extend(rhs.coeff.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}


impl<S: Shape> Add for QuadExpr<S> {
  type Output = QuadExpr<S>;
  fn add(mut self, rhs: QuadExpr<S>) -> QuadExpr<S> {
    self.lind.extend(rhs.lind);
    self.lval.extend(rhs.lval);
    self.qrow.extend(rhs.qrow);
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval);
    self.offset += rhs.offset;
    self
  }
}

impl<S: Shape> Sub for QuadExpr<S> {
  type Output = QuadExpr<S>;
  fn sub(mut self, rhs: QuadExpr<S>) -> QuadExpr<S> {
    self.lind.extend(rhs.lind);
    self.lval.extend(rhs.lval.into_iter().map(|c| -c));
    self.qrow.extend(rhs.qrow);
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}




/// Gurobi Model
pub struct Model<'a> {
  model: *mut ffi::GRBmodel,
  env: &'a Env,
  vars: Vec<i32>,
  constrs: Vec<i32>,
  qconstrs: Vec<i32>
}

impl<'a> Model<'a> {
  /// create an empty model which associated with certain environment.
  pub fn new(env: &'a Env, model: *mut ffi::GRBmodel) -> Model<'a> {
    Model {
      model: model,
      env: env,
      vars: Vec::new(),
      constrs: Vec::new(),
      qconstrs: Vec::new()
    }
  }

  /// apply all modification of the model to process.
  pub fn update(&mut self) -> Result<()> {
    let error = unsafe { ffi::GRBupdatemodel(self.model) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// create a copy of the model
  pub fn copy(&self) -> Result<Model> {
    let copied = unsafe { ffi::GRBcopymodel(self.model) };
    if copied.is_null() {
      return Err(Error::FromAPI("Failed to create a copy of the model".to_owned(), 20002));
    }
    Ok(Model {
      env: self.env,
      model: copied,
      vars: self.vars.clone(),
      constrs: self.constrs.clone(),
      qconstrs: self.qconstrs.clone()
    })
  }

  /// optimize the model.
  pub fn optimize(&mut self) -> Result<()> {
    try!(self.update());

    let error = unsafe { ffi::GRBoptimize(self.model) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  /// write information of the model to file.
  pub fn write(&self, filename: &str) -> Result<()> {
    let filename = try!(util::make_c_str(filename));
    let error = unsafe { ffi::GRBwrite(self.model, filename.as_ptr()) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// add a decision variable to the model.
  fn add_var(&mut self, name: &str, vtype: VarType, obj: f64) -> Result<i32> {
    // extract parameters
    let (vtype, lb, ub) = vtype.into();
    let name = try!(util::make_c_str(name));

    let error = unsafe {
      ffi::GRBaddvar(self.model,
                     0,
                     null(),
                     null(),
                     obj,
                     lb,
                     ub,
                     vtype,
                     name.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    let col_no = self.vars.len() as i32;
    self.vars.push(col_no);

    Ok(col_no)
  }

  /// add a set of decision variables to the model.
  pub fn add_vars<S: Shape>(&mut self, name: &str, vtype: VarType, shape: S) -> Result<Variable<S>> {
    let mut vars = Vec::with_capacity(shape.size());
    for vname in shape.names(name) {
      let v = try!(self.add_var(vname.as_str(), vtype, 0.0));
      vars.push(v);
    }
    Ok(Variable(vars, shape))
  }

  fn add_constr(&mut self, name: &str, vars: &[i32], coeff: &[f64], sense: ConstrSense, rhs: f64) -> Result<i32> {
    let constrname = try!(util::make_c_str(name));
    let error = unsafe {
      ffi::GRBaddconstr(self.model,
                        coeff.len() as ffi::c_int,
                        vars.as_ptr(),
                        coeff.as_ptr(),
                        sense.into(),
                        rhs,
                        constrname.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    let row_no = self.constrs.len() as i32;
    self.constrs.push(row_no);

    Ok(row_no)
  }


  /// add a linear constraint to the model.
  pub fn add_constrs<S: Shape>(&mut self, name: &str, expr: LinExpr<S>, sense: ConstrSense, rhs: f64)
                               -> Result<Constr<S>> {
    let shape = try!(expr.shape().ok_or(Error::InconsitentDims));

    let mut constrs = Vec::with_capacity(shape.size());
    for (i, cname) in shape.names(name).into_iter().enumerate() {
      let vars: Vec<_> = expr.vars.iter().map(|v| v.0[i]).collect();
      let c = try!(self.add_constr(cname.as_str(),
                                   vars.as_slice(),
                                   expr.coeff.as_slice(),
                                   sense,
                                   rhs - expr.offset));
      constrs.push(c);
    }
    Ok(Constr(constrs, shape))
  }

  fn add_qconstr(&mut self, constrname: &str, lind: &[i32], lval: &[f64], qrow: &[i32], qcol: &[i32], qval: &[f64],
                 sense: ConstrSense, rhs: f64)
                 -> Result<i32> {
    let constrname = try!(util::make_c_str(constrname));

    let error = unsafe {
      ffi::GRBaddqconstr(self.model,
                         lval.len() as ffi::c_int,
                         lind.as_ptr(),
                         lval.as_ptr(),
                         qval.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr(),
                         sense.into(),
                         rhs,
                         constrname.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    let qrow_no = self.qconstrs.len() as i32;
    self.qconstrs.push(qrow_no);

    Ok(qrow_no)
  }

  /// add a quadratic constraint to the model.
  pub fn add_qconstrs<S: Shape>(&mut self, name: &str, expr: QuadExpr<S>, sense: ConstrSense, rhs: f64)
                                -> Result<QConstr<S>> {
    let shape = try!(expr.shape().ok_or(Error::InconsitentDims));

    let mut constrs = Vec::with_capacity(shape.size());
    for (i, cname) in shape.names(name).into_iter().enumerate() {
      let lind: Vec<_> = expr.lind.iter().map(|v| v.0[i]).collect();
      let qrow: Vec<_> = expr.qrow.iter().map(|v| v.0[i]).collect();
      let qcol: Vec<_> = expr.qcol.iter().map(|v| v.0[i]).collect();
      let c = try!(self.add_qconstr(cname.as_str(),
                                    lind.as_slice(),
                                    expr.lval.as_slice(),
                                    qrow.as_slice(),
                                    qcol.as_slice(),
                                    expr.qval.as_slice(),
                                    sense,
                                    rhs - expr.offset));
      constrs.push(c);
    }
    Ok(QConstr(constrs, shape))
  }

  /// Set the objective function of the model.
  pub fn set_objective<Expr: Into<QuadExpr<()>>>(&mut self, expr: Expr, sense: ModelSense) -> Result<()> {
    let expr = expr.into();

    let lind: Vec<_> = expr.lind.into_iter().map(|v| v.0[0]).collect();
    let qrow: Vec<_> = expr.qrow.into_iter().map(|v| v.0[0]).collect();
    let qcol: Vec<_> = expr.qcol.into_iter().map(|v| v.0[0]).collect();

    try!(AttrArray::set_list(self, attr::Obj, lind.as_slice(), expr.lval.as_slice()));
    try!(self.add_qpterms(qrow.as_slice(), qcol.as_slice(), expr.qval.as_slice()));
    self.set(attr::ModelSense, sense.into())
  }

  /// add quadratic terms of objective function.
  fn add_qpterms(&mut self, qrow: &[i32], qcol: &[i32], qval: &[f64]) -> Result<()> {
    let error = unsafe {
      ffi::GRBaddqpterms(self.model,
                         qrow.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  /// add Special Order Set (SOS) constraint to the model.
  fn add_sos(&mut self, vars: &[i32], weights: &[f64], sostype: SOSType) -> Result<()> {
    if vars.len() != weights.len() {
      return Err(Error::InconsitentDims);
    }

    let beg = 0;
    let error = unsafe {
      ffi::GRBaddsos(self.model,
                     1,
                     vars.len() as ffi::c_int,
                     &sostype.into(),
                     &beg,
                     vars.as_ptr(),
                     weights.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  pub fn get<A: Attr>(&self, attr: A) -> Result<A::Output> { Attr::get(self, attr) }
  pub fn set<A: Attr>(&mut self, attr: A, value: A::Output) -> Result<()> { Attr::set(self, attr, value) }

  pub fn get_var<A: AttrArray + Copy, S: Shape>(&self, attr: A, var: &Variable<S>) -> Result<Vec<A::Output>> {
    let shape = try!(var.shape().ok_or(Error::InconsitentDims));

    let mut val = Vec::with_capacity(shape.size());
    for elem in var.0.iter() {
      let v = try!(A::get_element(self, attr, *elem));
      val.push(v);
    }
    Ok(val)
  }

  pub fn set_var<A: AttrArray + Copy, S: Shape>(&mut self, attr: A, var: &Variable<S>, val: &[A::Output])
                                                -> Result<()> {
    for (elem, v) in var.0.iter().zip(val.iter()) {
      try!(A::set_element(self, attr, *elem, v.clone()));
    }

    Ok(())
  }

  fn error_from_api(&self, errcode: ffi::c_int) -> Error { self.env.error_from_api(errcode) }
}


impl<'a> Drop for Model<'a> {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreemodel(self.model) };
    self.model = null_mut();
  }
}


///
pub trait Shape: Copy {
  fn size(&self) -> usize;
  fn names(&self, name: &str) -> Vec<String>;
}

impl Shape for () {
  fn size(&self) -> usize { 1 }
  fn names(&self, name: &str) -> Vec<String> { vec![name.to_string()] }
}

impl Shape for (usize) {
  fn size(&self) -> usize { *self }
  fn names(&self, name: &str) -> Vec<String> { (0..(*self)).map(|i| format!("{}[{}]", name, i)).collect() }
}

impl Shape for (usize, usize) {
  fn size(&self) -> usize { self.0 * self.1 }
  fn names(&self, name: &str) -> Vec<String> {
    (0..(self.0)).zip((0..(self.1))).map(|(i, j)| format!("{}[{}][{}]", name, i, j)).collect()
  }
}

impl Shape for (usize, usize, usize) {
  fn size(&self) -> usize { self.0 * self.1 * self.2 }
  fn names(&self, name: &str) -> Vec<String> {
    (0..(self.0))
      .zip((0..(self.1)))
      .zip((0..(self.2)))
      .map(|((i, j), k)| format!("{}[{}][{}][{}]", name, i, j, k))
      .collect()
  }
}


/// provides function to query/set the value of scalar attribute.
pub trait Attr: Into<CString> {
  type Output: Clone;
  type Init: Clone + types::Init + types::Into<Self::Output> + types::AsRawPtr<Self::RawGet>;
  type RawGet;
  type RawSet: types::From<Self::Output>;

  fn get(model: &Model, attr: Self) -> Result<Self::Output> {
    let mut value: Self::Init = types::Init::init();

    let error = unsafe {
      use types::AsRawPtr;
      Self::get_attr(model.model, attr.into().as_ptr(), value.as_rawptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(types::Into::into(value))
  }

  fn set(model: &mut Model, attr: Self, value: Self::Output) -> Result<()> {
    let error = unsafe { Self::set_attr(model.model, attr.into().as_ptr(), types::From::from(value)) };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(())
  }

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawGet) -> ffi::c_int;

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int;
}


/// provides function to query/set the value of vectorized attribute.
pub trait AttrArray: Into<CString> {
  type Output: Clone;
  type Init: Clone + types::Init + types::Into<Self::Output> + types::AsRawPtr<Self::RawGet>;
  type RawGet;
  type RawSet: types::From<Self::Output>;

  fn get_element(model: &Model, attr: Self, element: i32) -> Result<Self::Output> {
    let mut value: Self::Init = types::Init::init();

    let error = unsafe {
      use types::AsRawPtr;
      Self::get_attrelement(model.model,
                            attr.into().as_ptr(),
                            element,
                            value.as_rawptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(types::Into::into(value))
  }

  fn set_element(model: &mut Model, attr: Self, element: i32, value: Self::Output) -> Result<()> {
    let error = unsafe {
      Self::set_attrelement(model.model,
                            attr.into().as_ptr(),
                            element,
                            types::From::from(value))
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(())
  }

  fn get_array(model: &Model, attr: Self, first: usize, len: usize) -> Result<Vec<Self::Output>> {
    let mut values = Self::init_array(len);

    let error = unsafe {
      Self::get_attrarray(model.model,
                          attr.into().as_ptr(),
                          first as ffi::c_int,
                          len as ffi::c_int,
                          values.as_mut_ptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(values.into_iter().map(|s| types::Into::into(s)).collect())
  }

  fn set_array(model: &mut Model, attr: Self, first: usize, values: &[Self::Output]) -> Result<()> {
    let values = try!(Self::to_rawsets(values));

    let error = unsafe {
      Self::set_attrarray(model.model,
                          attr.into().as_ptr(),
                          first as ffi::c_int,
                          values.len() as ffi::c_int,
                          values.as_ptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(())
  }

  fn get_list(model: &Model, attr: Self, ind: &[i32]) -> Result<Vec<Self::Output>> {
    let mut values = Self::init_array(ind.len());

    let error = unsafe {
      Self::get_attrlist(model.model,
                         attr.into().as_ptr(),
                         ind.len() as ffi::c_int,
                         ind.as_ptr(),
                         values.as_mut_ptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(values.into_iter().map(|s| types::Into::into(s)).collect())
  }

  fn set_list(model: &mut Model, attr: Self, ind: &[i32], values: &[Self::Output]) -> Result<()> {
    if ind.len() != values.len() {
      return Err(Error::InconsitentDims);
    }

    let values = try!(Self::to_rawsets(values));

    let error = unsafe {
      Self::set_attrlist(model.model,
                         attr.into().as_ptr(),
                         ind.len() as ffi::c_int,
                         ind.as_ptr(),
                         values.as_ptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(())
  }

  fn init_array(len: usize) -> Vec<Self::Init> { iter::repeat(types::Init::init()).take(len).collect() }

  fn to_rawsets(values: &[Self::Output]) -> Result<Vec<Self::RawSet>> {
    Ok(values.iter().map(|v| types::From::from(v.clone())).collect())
  }

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: Self::RawGet)
                            -> ffi::c_int;

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: Self::RawSet)
                            -> ffi::c_int;

  unsafe fn get_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *mut Self::Init)
                          -> ffi::c_int;

  unsafe fn set_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *const Self::RawSet)
                          -> ffi::c_int;

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut Self::Init)
                         -> ffi::c_int;

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int;
}



impl Attr for attr::IntAttr {
  type Output = i32;
  type Init = i32;
  type RawGet = *mut ffi::c_int;
  type RawSet = ffi::c_int;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_int) -> ffi::c_int {
    ffi::GRBgetintattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetintattr(model, attrname, value)
  }
}

impl AttrArray for attr::IntAttr {
  type Output = i32;
  type Init = i32;
  type RawGet = *mut ffi::c_int;
  type RawSet = ffi::c_int;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_int)
                            -> ffi::c_int {
    ffi::GRBgetintattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: Self::RawSet)
                            -> ffi::c_int {
    ffi::GRBsetintattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *mut ffi::c_int)
                          -> ffi::c_int {
    ffi::GRBgetintattrarray(model, attrname, first, len, values)
  }

  unsafe fn set_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *const Self::RawSet)
                          -> ffi::c_int {
    ffi::GRBsetintattrarray(model, attrname, first, len, values)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut ffi::c_int)
                         -> ffi::c_int {
    ffi::GRBgetintattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int {
    ffi::GRBsetintattrlist(model, attrname, len, ind, values)
  }
}


impl Attr for attr::DoubleAttr {
  type Output = f64;
  type Init = f64;
  type RawGet = *mut ffi::c_double;
  type RawSet = ffi::c_double;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_double) -> ffi::c_int {
    ffi::GRBgetdblattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetdblattr(model, attrname, value)
  }
}

impl AttrArray for attr::DoubleAttr {
  type Output = f64;
  type Init = f64;
  type RawGet = *mut ffi::c_double;
  type RawSet = ffi::c_double;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_double)
                            -> ffi::c_int {
    ffi::GRBgetdblattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: Self::RawSet)
                            -> ffi::c_int {
    ffi::GRBsetdblattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *mut ffi::c_double)
                          -> ffi::c_int {
    ffi::GRBgetdblattrarray(model, attrname, first, len, values)
  }

  unsafe fn set_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *const Self::RawSet)
                          -> ffi::c_int {
    ffi::GRBsetdblattrarray(model, attrname, first, len, values)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut ffi::c_double)
                         -> ffi::c_int {
    ffi::GRBgetdblattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int {
    ffi::GRBsetdblattrlist(model, attrname, len, ind, values)
  }
}


impl AttrArray for attr::CharAttr {
  type Output = i8;
  type Init = i8;
  type RawGet = *mut ffi::c_char;
  type RawSet = ffi::c_char;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_char)
                            -> ffi::c_int {
    ffi::GRBgetcharattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: Self::RawSet)
                            -> ffi::c_int {
    ffi::GRBsetcharattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *mut ffi::c_char)
                          -> ffi::c_int {
    ffi::GRBgetcharattrarray(model, attrname, first, len, values)
  }

  unsafe fn set_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *const Self::RawSet)
                          -> ffi::c_int {
    ffi::GRBsetcharattrarray(model, attrname, first, len, values)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut ffi::c_char)
                         -> ffi::c_int {
    ffi::GRBgetcharattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int {
    ffi::GRBsetcharattrlist(model, attrname, len, ind, values)
  }
}


impl Attr for attr::StringAttr {
  type Output = String;
  type Init = ffi::c_str;
  type RawGet = *mut ffi::c_str;
  type RawSet = ffi::c_str;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_str) -> ffi::c_int {
    ffi::GRBgetstrattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetstrattr(model, attrname, value)
  }
}

impl AttrArray for attr::StringAttr {
  type Output = String;
  type Init = ffi::c_str;
  type RawGet = *mut ffi::c_str;
  type RawSet = ffi::c_str;

  fn to_rawsets(values: &[String]) -> Result<Vec<ffi::c_str>> {
    let values = values.into_iter().map(|s| util::make_c_str(s)).collect::<Vec<_>>();
    if values.iter().any(|ref s| s.is_err()) {
      return Err(Error::StringConversion);
    }
    Ok(values.into_iter().map(|s| s.unwrap().as_ptr()).collect())
  }

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_str)
                            -> ffi::c_int {
    ffi::GRBgetstrattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: Self::RawSet)
                            -> ffi::c_int {
    ffi::GRBsetstrattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *mut ffi::c_str)
                          -> ffi::c_int {
    ffi::GRBgetstrattrarray(model, attrname, first, len, values)
  }

  unsafe fn set_attrarray(model: *mut ffi::GRBmodel, attrname: ffi::c_str, first: ffi::c_int, len: ffi::c_int,
                          values: *const Self::RawSet)
                          -> ffi::c_int {
    ffi::GRBsetstrattrarray(model, attrname, first, len, values)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut ffi::c_str)
                         -> ffi::c_int {
    ffi::GRBgetstrattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const ffi::c_str)
                         -> ffi::c_int {
    ffi::GRBsetstrattrlist(model, attrname, len, ind, values)
  }
}
