// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use super::ffi;
use super::itertools::{Itertools, Zip};

use std::iter;
use std::ffi::CString;
use std::ptr::{null, null_mut};
use std::ops::{Add, Sub, Mul};
use std::mem::transmute;
use std::rc::Rc;
use std::cell::Cell;
use std::slice::Iter;

use env::Env;
use error::{Error, Result};
use util;


pub mod attr {
  pub use ffi::{IntAttr, DoubleAttr, CharAttr, StringAttr};

  pub use self::IntAttr::*;
  pub use self::DoubleAttr::*;
  pub use self::CharAttr::*;
  pub use self::StringAttr::*;
}

/// provides function to query/set the value of scalar attribute.
pub trait Attr: Into<CString> {
  type Out;
  type Buf: util::Init + util::Into<Self::Out> + util::AsRawPtr<Self::RawGet>;
  type RawGet;
  type RawSet: util::From<Self::Out>;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawGet) -> ffi::c_int;

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int;


  fn get(model: &Model, attr: Self) -> Result<Self::Out> {
    let mut value: Self::Buf = util::Init::init();

    let error = unsafe {
      use util::AsRawPtr;
      Self::get_attr(model.model, attr.into().as_ptr(), value.as_rawptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(util::Into::into(value))
  }

  fn set(model: &mut Model, attr: Self, value: Self::Out) -> Result<()> {
    let error = unsafe { Self::set_attr(model.model, attr.into().as_ptr(), util::From::from(value)) };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(())
  }
}

impl Attr for attr::IntAttr {
  // {{{
  type Out = i32;
  type Buf = i32;
  type RawGet = *mut ffi::c_int;
  type RawSet = ffi::c_int;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_int) -> ffi::c_int {
    ffi::GRBgetintattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetintattr(model, attrname, value)
  }
} // }}}

impl Attr for attr::DoubleAttr {
  // {{{
  type Out = f64;
  type Buf = f64;
  type RawGet = *mut ffi::c_double;
  type RawSet = ffi::c_double;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_double) -> ffi::c_int {
    ffi::GRBgetdblattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetdblattr(model, attrname, value)
  }
} // }}}

impl Attr for attr::StringAttr {
  // {{{
  type Out = String;
  type Buf = ffi::c_str;
  type RawGet = *mut ffi::c_str;
  type RawSet = ffi::c_str;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_str) -> ffi::c_int {
    ffi::GRBgetstrattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetstrattr(model, attrname, value)
  }
} // }}}


/// provides function to query/set the value of vectorized attribute.
pub trait AttrArray: Into<CString> {
  type Out: Clone;
  type Buf: Clone + util::Init + util::Into<Self::Out> + util::AsRawPtr<Self::RawGet>;
  type RawGet;
  type RawSet: util::From<Self::Out>;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            values: Self::RawGet)
                            -> ffi::c_int;
  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            values: Self::RawSet)
                            -> ffi::c_int;

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut Self::Buf)
                         -> ffi::c_int;

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int;

  fn get_element(model: &Model, attr: Self, element: i32) -> Result<Self::Out> {
    let mut value: Self::Buf = util::Init::init();

    let error = unsafe {
      use util::AsRawPtr;
      Self::get_attrelement(model.model,
                            attr.into().as_ptr(),
                            element,
                            value.as_rawptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(util::Into::into(value))
  }

  fn set_element(model: &mut Model, attr: Self, element: i32, value: Self::Out) -> Result<()> {
    let error = unsafe {
      Self::set_attrelement(model.model,
                            attr.into().as_ptr(),
                            element,
                            util::From::from(value))
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(())
  }

  fn get_list(model: &Model, attr: Self, ind: &[i32]) -> Result<Vec<Self::Out>> {
    let mut values: Vec<_> = iter::repeat(util::Init::init()).take(ind.len()).collect();

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

    Ok(values.into_iter().map(|s| util::Into::into(s)).collect())
  }

  fn set_list(model: &mut Model, attr: Self, ind: &[i32], values: &[Self::Out]) -> Result<()> {
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

  fn to_rawsets(values: &[Self::Out]) -> Result<Vec<Self::RawSet>> {
    Ok(values.iter().map(|v| util::From::from(v.clone())).collect())
  }
}

impl AttrArray for attr::IntAttr {
  // {{{
  type Out = i32;
  type Buf = i32;
  type RawGet = *mut ffi::c_int;
  type RawSet = ffi::c_int;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_int)
                            -> ffi::c_int {
    ffi::GRBgetintattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: ffi::c_int)
                            -> ffi::c_int {
    ffi::GRBsetintattrelement(model, attrname, element, value)
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
} // }}}

impl AttrArray for attr::DoubleAttr {
  // {{{
  type Out = f64;
  type Buf = f64;
  type RawGet = *mut ffi::c_double;
  type RawSet = ffi::c_double;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_double)
                            -> ffi::c_int {
    ffi::GRBgetdblattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: ffi::c_double)
                            -> ffi::c_int {
    ffi::GRBsetdblattrelement(model, attrname, element, value)
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
} // }}}

impl AttrArray for attr::CharAttr {
  // {{{
  type Out = i8;
  type Buf = i8;
  type RawGet = *mut ffi::c_char;
  type RawSet = ffi::c_char;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_char)
                            -> ffi::c_int {
    ffi::GRBgetcharattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: ffi::c_char)
                            -> ffi::c_int {
    ffi::GRBsetcharattrelement(model, attrname, element, value)
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
} // }}}

impl AttrArray for attr::StringAttr {
  // {{{
  type Out = String;
  type Buf = ffi::c_str;
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

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: ffi::c_str)
                            -> ffi::c_int {
    ffi::GRBsetstrattrelement(model, attrname, element, value)
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
} // }}}


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


/// A set of values which represents the status of a model.
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

pub trait Proxy: ProxyBase {
  fn get<A: AttrArray>(&self, model: &Model, attr: A) -> Result<A::Out> { model.get_value(attr, self.index()) }

  fn set<A: AttrArray>(&mut self, model: &mut Model, attr: A, val: A::Out) -> Result<()> {
    model.set_value(attr, self.index(), val)
  }
}

impl<T: ProxyBase> Proxy for T {}


/// represents a set of decision variables.
#[derive(Clone)]
pub struct Var(Rc<Cell<i32>>);

/// The proxy object of a set of linear constraints.
#[derive(Clone)]
pub struct Constr(Rc<Cell<i32>>);

/// The proxy object of a set of quadratic constraints.
#[derive(Clone)]
pub struct QConstr(Rc<Cell<i32>>);

/// The proxy object of a Special Order Set (SOS) constraint.
#[derive(Clone)]
pub struct SOS(Rc<Cell<i32>>);

impl ProxyBase for Var {
  fn new(idx: i32) -> Var { Var(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }
}

impl ProxyBase for Constr {
  fn new(idx: i32) -> Constr { Constr(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }
}

impl ProxyBase for QConstr {
  fn new(idx: i32) -> QConstr { QConstr(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }
}

impl ProxyBase for SOS {
  fn new(idx: i32) -> SOS { SOS(Rc::new(Cell::new(idx))) }
  fn index(&self) -> i32 { self.0.get() }
  fn set_index(&mut self, value: i32) { self.0.set(value) }
}




/// A linear expression of Gurobi model.
///
/// It consists of a constant term plus a list of coefficients and variables.
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
  pub fn get_value(&self, model: &Model) -> Result<f64> { model.calc_value(self) }
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


/// A quadratic expression of Gurobi model.
///
/// It consists of a linear expression, and a set of
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
  pub fn get_value(&self, model: &Model) -> Result<f64> { model.calc_value(self) }
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



/// The type of cost function at feasibility relaxation.
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



/// A Gurobi model object associated with certain environment.
pub struct Model<'a> {
  model: *mut ffi::GRBmodel,
  env: &'a Env,
  vars: Vec<Var>,
  constrs: Vec<Constr>,
  qconstrs: Vec<QConstr>,
  sos: Vec<SOS>
}

impl<'a> Model<'a> {
  /// create an empty model which associated with certain environment.
  pub fn new(env: &'a Env, model: *mut ffi::GRBmodel) -> Model<'a> {
    Model {
      model: model,
      env: env,
      vars: Vec::new(),
      constrs: Vec::new(),
      qconstrs: Vec::new(),
      sos: Vec::new()
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
      qconstrs: self.qconstrs.clone(),
      sos: self.sos.clone()
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
  pub fn add_var(&mut self, name: &str, vtype: VarType) -> Result<Var> {
    // extract parameters
    let (vtype, lb, ub) = vtype.into();
    let name = try!(util::make_c_str(name));

    let error = unsafe {
      ffi::GRBaddvar(self.model,
                     0,
                     null(),
                     null(),
                     0.0,
                     lb,
                     ub,
                     vtype,
                     name.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    let col_no = self.vars.len() as i32;
    self.vars.push(Var::new(col_no));

    Ok(self.vars.last().cloned().unwrap())
  }

  /// add a linear constraint to the model.
  pub fn add_constr(&mut self, name: &str, expr: LinExpr, sense: ConstrSense, rhs: f64) -> Result<Constr> {
    let constrname = try!(util::make_c_str(name));
    let error = unsafe {
      ffi::GRBaddconstr(self.model,
                        expr.coeff.len() as ffi::c_int,
                        expr.vars.into_iter().map(|e| e.index()).collect_vec().as_ptr(),
                        expr.coeff.as_ptr(),
                        sense.into(),
                        rhs - expr.offset,
                        constrname.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    let row_no = self.constrs.len() as i32;
    self.constrs.push(Constr::new(row_no));

    Ok(self.constrs.last().cloned().unwrap())
  }

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self, constrname: &str, expr: QuadExpr, sense: ConstrSense, rhs: f64) -> Result<QConstr> {
    let constrname = try!(util::make_c_str(constrname));

    let error = unsafe {
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
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

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

    try!(AttrArray::set_list(self, attr::Obj, lind.as_slice(), expr.lval.as_slice()));
    try!(self.add_qpterms(qrow.as_slice(), qcol.as_slice(), expr.qval.as_slice()));
    self.set(attr::ModelSense, sense.into())
  }

  /// Query the value of attributes which associated with variable/constraints.
  pub fn get<A: Attr>(&self, attr: A) -> Result<A::Out> { A::get(self, attr) }

  /// Set the value of attributes which associated with variable/constraints.
  pub fn set<A: Attr>(&mut self, attr: A, val: A::Out) -> Result<()> { A::set(self, attr, val) }

  fn get_value<A: AttrArray>(&self, attr: A, e: i32) -> Result<A::Out> { A::get_element(self, attr, e) }

  fn set_value<A: AttrArray>(&mut self, attr: A, e: i32, val: A::Out) -> Result<()> {
    A::set_element(self, attr, e, val)
  }

  /// Query the value of attributes which associated with variable/constraints.
  pub fn get_values<A: AttrArray, P: Proxy>(&self, attr: A, item: &[P]) -> Result<Vec<A::Out>> {
    A::get_list(self,
                attr,
                item.iter().map(|e| e.index()).collect_vec().as_slice())
  }

  /// Set the value of attributes which associated with variable/constraints.
  pub fn set_values<A: AttrArray, P: Proxy>(&mut self, attr: A, item: &[P], val: &[A::Out]) -> Result<()> {
    A::set_list(self,
                attr,
                item.iter().map(|e| e.index()).collect_vec().as_slice(),
                val)
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
    let error = unsafe {
      ffi::GRBfeasrelax(self.model,
                        relaxtype.into(),
                        minrelax,
                        pen_lb.as_ptr(),
                        pen_ub.as_ptr(),
                        pen_rhs.as_ptr(),
                        &mut feasobj)
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

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

  /// Compute an Irreducible Inconsistent Subsystem (IIS) of the model.
  pub fn compute_iis(&mut self) -> Result<()> {
    let error = unsafe { ffi::GRBcomputeIIS(self.model) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
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

  /// Remove a variable from the model.
  pub fn remove_var(&mut self, mut item: Var) -> Result<()> {
    let index = item.index();
    if index >= self.vars.len() as i32 {
      return Err(Error::InconsitentDims);
    }

    if index != -1 {
      let error = unsafe { ffi::GRBdelvars(self.model, 1, &index) };
      if error != 0 {
        return Err(self.error_from_api(error));
      }

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
      let error = unsafe { ffi::GRBdelconstrs(self.model, 1, &index) };
      if error != 0 {
        return Err(self.error_from_api(error));
      }

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
      let error = unsafe { ffi::GRBdelqconstrs(self.model, 1, &index) };
      if error != 0 {
        return Err(self.error_from_api(error));
      }

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
      let error = unsafe { ffi::GRBdelsos(self.model, 1, &index) };
      if error != 0 {
        return Err(self.error_from_api(error));
      }

      self.sos.remove(index as usize);
      item.set_index(-1);

      // reset all of the remaining items.
      for (idx, ref mut s) in self.sos.iter_mut().enumerate().skip(index as usize) {
        s.set_index(idx as i32);
      }
    }
    Ok(())
  }


  /// calculates the actual value of linear/quadratic expression.
  fn calc_value<E: Into<QuadExpr> + Clone>(&self, expr: &E) -> Result<f64> {
    let expr: QuadExpr = (*expr).clone().into();

    let lind = try!(self.get_values(attr::X, expr.lind.as_slice()));
    let qrow = try!(self.get_values(attr::X, expr.qrow.as_slice()));
    let qcol = try!(self.get_values(attr::X, expr.qcol.as_slice()));

    Ok(Zip::new((lind, expr.lval)).fold(0.0, |acc, (ind, val)| acc + ind * val) +
       Zip::new((qrow, qcol, expr.qval)).fold(0.0, |acc, (row, col, val)| acc + row * col * val) + expr.offset)
  }

  fn error_from_api(&self, errcode: ffi::c_int) -> Error { self.env.error_from_api(errcode) }
}


impl<'a> Drop for Model<'a> {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreemodel(self.model) };
    self.model = null_mut();
  }
}

#[test]
fn test_proxy() {
use super::*;

  let mut env = Env::new("").unwrap();
  env.set(param::OutputFlag, 0).unwrap();

  let mut model = env.new_model("hoge").unwrap();
  let x = model.add_var("x", Binary).unwrap();
  let y = model.add_var("y", Binary).unwrap();
  let z = model.add_var("z", Binary).unwrap();
  model.update().unwrap();

  model.remove_var(y.clone()).unwrap();

  assert_eq!(x.index(), 0);
  assert_eq!(y.index(), -1);
  assert_eq!(z.index(), 1);
}
