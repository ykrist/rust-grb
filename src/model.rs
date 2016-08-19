use super::ffi;
use super::itertools::Zip;

use std::iter;
use std::ffi::CString;
use std::ptr::{null, null_mut};
use std::ops::{Add, Sub, Mul};

use env::Env;
use error::{Error, Result};
use util;
use types;
use core::{Tensor, TensorVal, Shape};


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
  type Buf: types::Init + types::Into<Self::Out> + types::AsRawPtr<Self::RawGet>;
  type RawGet;
  type RawSet: types::From<Self::Out>;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawGet) -> ffi::c_int;

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int;


  fn get(model: &Model, attr: Self) -> Result<Self::Out> {
    let mut value: Self::Buf = types::Init::init();

    let error = unsafe {
      use types::AsRawPtr;
      Self::get_attr(model.model, attr.into().as_ptr(), value.as_rawptr())
    };
    if error != 0 {
      return Err(model.error_from_api(error));
    }

    Ok(types::Into::into(value))
  }

  fn set(model: &mut Model, attr: Self, value: Self::Out) -> Result<()> {
    let error = unsafe { Self::set_attr(model.model, attr.into().as_ptr(), types::From::from(value)) };
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
  type Buf: Clone + types::Init + types::Into<Self::Out> + types::AsRawPtr<Self::RawGet>;
  type RawGet;
  type RawSet: types::From<Self::Out>;

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut Self::Buf)
                         -> ffi::c_int;

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int;

  // fn get_element(model: &Model, attr: Self, element: i32) -> Result<Self::Out> {
  //   let mut value: Self::Buf = types::Init::init();
  //
  //   let error = unsafe {
  //     use types::AsRawPtr;
  //     Self::get_attrelement(model.model,
  //                           attr.into().as_ptr(),
  //                           element,
  //                           value.as_rawptr())
  //   };
  //   if error != 0 {
  //     return Err(model.error_from_api(error));
  //   }
  //
  //   Ok(types::Into::into(value))
  // }
  //
  // fn set_element(model: &mut Model, attr: Self, element: i32, value: Self::Out) -> Result<()> {
  //   let error = unsafe {
  //     Self::set_attrelement(model.model,
  //                           attr.into().as_ptr(),
  //                           element,
  //                           types::From::from(value))
  //   };
  //   if error != 0 {
  //     return Err(model.error_from_api(error));
  //   }
  //
  //   Ok(())
  // }
  //
  // fn get_array(model: &Model, attr: Self, first: usize, len: usize) -> Result<Vec<Self::Out>> {
  //   let mut values: Vec<_> = iter::repeat(types::Init::init()).take(len).collect();
  //   let error = unsafe {
  //     Self::get_attrarray(model.model,
  //                         attr.into().as_ptr(),
  //                         first as ffi::c_int,
  //                         len as ffi::c_int,
  //                         values.as_mut_ptr())
  //   };
  //   if error != 0 {
  //     return Err(model.error_from_api(error));
  //   }
  //
  //   Ok(values.into_iter().map(|s| types::Into::into(s)).collect())
  // }
  //
  // fn set_array(model: &mut Model, attr: Self, first: usize, values: &[Self::Out]) -> Result<()> {
  //   let values = try!(Self::to_rawsets(values));
  //
  //   let error = unsafe {
  //     Self::set_attrarray(model.model,
  //                         attr.into().as_ptr(),
  //                         first as ffi::c_int,
  //                         values.len() as ffi::c_int,
  //                         values.as_ptr())
  //   };
  //   if error != 0 {
  //     return Err(model.error_from_api(error));
  //   }
  //
  //   Ok(())
  // }

  fn get_list(model: &Model, attr: Self, ind: &[i32]) -> Result<Vec<Self::Out>> {
    let mut values: Vec<_> = iter::repeat(types::Init::init()).take(ind.len()).collect();

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
    Ok(values.iter().map(|v| types::From::from(v.clone())).collect())
  }
}

impl AttrArray for attr::IntAttr {
  // {{{
  type Out = i32;
  type Buf = i32;
  type RawGet = *mut ffi::c_int;
  type RawSet = ffi::c_int;

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

/// represents a set of decision variables.
#[derive(Clone)]
pub struct Var<S: Shape>(Vec<i32>, S);

impl<S: Shape> Var<S> {
  pub fn get<A: AttrArray + Copy>(&self, model: &Model, attr: A) -> Result<TensorVal<A::Out, S>> {
    model.get_value(attr, self)
  }

  pub fn set<A: AttrArray + Copy>(&mut self, model: &mut Model, attr: A, val: &Tensor<A::Out, S>) -> Result<()> {
    model.set_value(attr, self, val)
  }
}

impl<S: Shape> Tensor<i32, S> for Var<S> {
  fn shape(&self) -> Option<S> { Some(self.1) }
  fn body(&self) -> &Vec<i32> { &self.0 }
}


/// The proxy object of a set of linear constraints.
#[derive(Clone)]
pub struct Constr<S: Shape>(Vec<i32>, S);

impl<S: Shape> Constr<S> {
  pub fn get<A: AttrArray + Copy>(&self, model: &Model, attr: A) -> Result<TensorVal<A::Out, S>> {
    model.get_value(attr, self)
  }

  pub fn set<A: AttrArray + Copy>(&mut self, model: &mut Model, attr: A, val: &Tensor<A::Out, S>) -> Result<()> {
    model.set_value(attr, self, val)
  }
}

impl<S: Shape> Tensor<i32, S> for Constr<S> {
  fn shape(&self) -> Option<S> { Some(self.1) }
  fn body(&self) -> &Vec<i32> { &self.0 }
}


/// The proxy object of a set of quadratic constraints.
#[derive(Clone)]
pub struct QConstr<S: Shape>(Vec<i32>, S);

impl<S: Shape> QConstr<S> {
  pub fn get<A: AttrArray + Copy>(&self, model: &Model, attr: A) -> Result<TensorVal<A::Out, S>> {
    model.get_value(attr, self)
  }

  pub fn set<A: AttrArray + Copy>(&mut self, model: &mut Model, attr: A, val: &Tensor<A::Out, S>) -> Result<()> {
    model.set_value(attr, self, val)
  }
}

impl<S: Shape> Tensor<i32, S> for QConstr<S> {
  fn shape(&self) -> Option<S> { Some(self.1) }
  fn body(&self) -> &Vec<i32> { &self.0 }
}



/// represents a set of linear expressions of decision variables.
#[derive(Clone)]
pub struct LinExpr<S: Shape> {
  vars: Vec<Var<S>>,
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

  pub fn term(mut self, v: Var<S>, c: f64) -> Self {
    self.vars.push(v);
    self.coeff.push(c);
    self
  }

  pub fn offset(mut self, offset: f64) -> Self {
    self.offset += offset;
    self
  }

  /// Get actual value of the expression.
  pub fn value(&self, model: &Model) -> Result<TensorVal<f64, S>> { model.calc_value(self) }

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


/// represents a set of quadratic expressions of decision variables.
#[derive(Clone)]
pub struct QuadExpr<S: Shape> {
  lind: Vec<Var<S>>,
  lval: Vec<f64>,
  qrow: Vec<Var<S>>,
  qcol: Vec<Var<S>>,
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

  pub fn term(mut self, var: Var<S>, coeff: f64) -> Self {
    self.lind.push(var);
    self.lval.push(coeff);
    self
  }

  pub fn qterm(mut self, row: Var<S>, col: Var<S>, coeff: f64) -> Self {
    self.qrow.push(row);
    self.qcol.push(col);
    self.qval.push(coeff);
    self
  }

  pub fn offset(mut self, offset: f64) -> Self {
    self.offset += offset;
    self
  }

  /// Get actual value of the expression.
  pub fn value(&self, model: &Model) -> Result<TensorVal<f64, S>> { model.calc_value(self) }

  /// Get the shape of expression.
  pub fn shape(&self) -> Option<S> { self.lind.get(0).map(|v| v.1) }
}


impl<S: Shape> Mul<f64> for Var<S> {
  type Output = LinExpr<S>;
  fn mul(self, rhs: f64) -> Self::Output { LinExpr::new().term(self, rhs) }
}

impl<S: Shape> Mul<Var<S>> for f64 {
  type Output = LinExpr<S>;
  fn mul(self, rhs: Var<S>) -> Self::Output { LinExpr::new().term(rhs, self) }
}


impl<S: Shape> Mul for Var<S> {
  type Output = QuadExpr<S>;
  fn mul(self, rhs: Var<S>) -> Self::Output { QuadExpr::new().qterm(self, rhs, 1.0) }
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
  pub fn add_vars<S: Shape>(&mut self, name: &str, vtype: VarType, shape: S) -> Result<Var<S>> {
    let mut vars = Vec::with_capacity(shape.size());
    for vname in shape.names(name) {
      let v = try!(self.add_var(vname.as_str(), vtype, 0.0));
      vars.push(v);
    }
    Ok(Var(vars, shape))
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
  #[allow(dead_code)]
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

  /// Query the value of attribute which associated with the model.
  pub fn get<A: Attr>(&self, attr: A) -> Result<A::Out> { A::get(self, attr) }

  /// Set the value of attribute which associated with the model.
  pub fn set<A: Attr>(&mut self, attr: A, value: A::Out) -> Result<()> { A::set(self, attr, value) }

  fn get_value<A, S, E>(&self, attr: A, e: &E) -> Result<TensorVal<A::Out, S>>
    where A: AttrArray + Copy, S: Shape, E: Tensor<i32, S>
  {
    let shape = try!(e.shape().ok_or(Error::InconsitentDims));
    A::get_list(self, attr, e.body().as_slice()).map(|val| TensorVal::new(val, shape))
  }

  fn set_value<A, S, E>(&mut self, attr: A, e: &E, val: &Tensor<A::Out, S>) -> Result<()>
    where A: AttrArray + Copy, S: Shape, E: Tensor<i32, S>
  {
    A::set_list(self, attr, e.body().as_slice(), val.body().as_slice())
  }

  /// Query the value of attributes which associated with variable/constraints.
  pub fn get_values<A, S, E>(&self, attr: A, e: &[&E]) -> Result<Vec<TensorVal<A::Out, S>>>
    where A: AttrArray + Copy, S: Shape, E: Tensor<i32, S>
  {
    let mut buf = Vec::with_capacity(e.len());
    for &e in e.iter() {
      let value = try!(self.get_value(attr, e));
      buf.push(value);
    }
    Ok(buf)
  }

  /// Set the value of attributes which associated with variable/constraints.
  pub fn set_values<A, S, E>(&mut self, attr: A, e: &[&E], val: &[&Tensor<A::Out, S>]) -> Result<()>
    where A: AttrArray + Copy, S: Shape, E: Tensor<i32, S>
  {
    for (&e, &val) in Zip::new((e.iter(), val.iter())) {
      try!(self.set_value(attr, e, val));
    }

    Ok(())
  }

  /// calculates the actual value of linear/quadratic expression.
  fn calc_value<S, E>(&self, expr: &E) -> Result<TensorVal<f64, S>>
    where S: Shape, E: Clone + Into<QuadExpr<S>>
  {
    let expr: QuadExpr<S> = (*expr).clone().into();

    let mut lbuf = Vec::with_capacity(expr.lind.len());
    for v in expr.lind.iter() {
      let val = try!(self.get_value(attr::X, v));
      lbuf.push(val.body().clone());
    }

    let mut qrow = Vec::with_capacity(expr.qval.len());
    for r in expr.qrow.iter() {
      let vrow = try!(self.get_value(attr::X, r));
      qrow.push(vrow.body().clone());
    }

    let mut qcol = Vec::with_capacity(expr.qval.len());
    for c in expr.qcol.iter() {
      let vcol = try!(self.get_value(attr::X, c));
      qcol.push(vcol.body().clone());
    }

    let shape = try!(expr.shape().ok_or(Error::InconsitentDims));
    let mut val = Vec::with_capacity(shape.size());
    for i in 0..(shape.size()) {
      let lval = Zip::new((lbuf.iter(), expr.lval.iter())).fold(0.0, |acc, (v, c)| acc + v[i] * c);
      let qval = Zip::new((qrow.iter(), qcol.iter(), expr.qval.iter()))
        .fold(0.0, |acc, (r, c, cf)| acc + r[i] * c[i] * cf);
      val.push(lval + qval + expr.offset);
    }
    Ok(TensorVal::new(val, shape))
  }

  fn error_from_api(&self, errcode: ffi::c_int) -> Error { self.env.error_from_api(errcode) }
}


impl<'a> Drop for Model<'a> {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreemodel(self.model) };
    self.model = null_mut();
  }
}
