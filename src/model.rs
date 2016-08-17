extern crate gurobi_sys as ffi;

use std::iter;
use std::ptr::{null, null_mut};
use std::ffi::CString;

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
#[derive(Debug)]
pub enum VarType {
  Binary,
  Continuous(f64, f64),
  Integer(i64, i64)
}

///
#[derive(Debug)]
pub enum ConstrSense {
  Equal,
  Greater,
  Less
}

///
#[derive(Debug)]
pub enum ModelSense {
  Minimize,
  Maximize
}

///
#[derive(Debug)]
pub enum SOSType {
  SOSType1,
  SOSType2
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
  pub fn add_var(&mut self, name: &str, vtype: VarType, obj: f64) -> Result<i32> {
    // extract parameters
    use self::VarType::*;
    let (vtype, lb, ub) = match vtype {
      Binary => ('B' as ffi::c_char, 0.0, 1.0),
      Continuous(lb, ub) => ('C' as ffi::c_char, lb, ub),
      Integer(lb, ub) => ('I' as ffi::c_char, lb as ffi::c_double, ub as ffi::c_double),
    };
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

  /// add a linear constraint to the model.
  pub fn add_constr(&mut self, name: &str, ind: &[i32], val: &[f64], sense: ConstrSense, rhs: f64) -> Result<i32> {
    if ind.len() != val.len() {
      return Err(Error::InconsitentDims);
    }

    let sense = match sense {
      ConstrSense::Equal => '=' as ffi::c_char,
      ConstrSense::Less => '<' as ffi::c_char,
      ConstrSense::Greater => '>' as ffi::c_char,
    };
    let constrname = try!(util::make_c_str(name));

    let error = unsafe {
      ffi::GRBaddconstr(self.model,
                        ind.len() as ffi::c_int,
                        ind.as_ptr(),
                        val.as_ptr(),
                        sense,
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

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self, constrname: &str, lind: &[i32], lval: &[f64], qrow: &[i32], qcol: &[i32],
                     qval: &[f64], sense: ConstrSense, rhs: f64)
                     -> Result<i32> {
    if lind.len() != lval.len() {
      return Err(Error::InconsitentDims);
    }
    if qrow.len() != qcol.len() || qcol.len() != qval.len() {
      return Err(Error::InconsitentDims);
    }

    let sense = match sense {
      ConstrSense::Equal => '=' as ffi::c_char,
      ConstrSense::Less => '<' as ffi::c_char,
      ConstrSense::Greater => '>' as ffi::c_char,
    };
    let constrname = try!(util::make_c_str(constrname));

    let error = unsafe {
      ffi::GRBaddqconstr(self.model,
                         lind.len() as ffi::c_int,
                         lind.as_ptr(),
                         lval.as_ptr(),
                         qrow.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr(),
                         sense,
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

  /// Set the objective function of the model.
  pub fn set_objective(&mut self, lind: &[i32], lval: &[f64], qrow: &[i32], qcol: &[i32], qval: &[f64],
                       sense: ModelSense)
                       -> Result<()> {
    let sense = match sense {
      ModelSense::Minimize => -1,
      ModelSense::Maximize => 1,
    };
    try!(self.set_list(attr::Obj, lind, lval));
    try!(self.add_qpterms(qrow, qcol, qval));
    self.set(attr::ModelSense, sense)
  }

  /// add quadratic terms of objective function.
  fn add_qpterms(&mut self, qrow: &[ffi::c_int], qcol: &[ffi::c_int], qval: &[ffi::c_double]) -> Result<()> {
    if qrow.len() != qcol.len() || qcol.len() != qval.len() {
      return Err(Error::InconsitentDims);
    }

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
  pub fn add_sos(&mut self, vars: &[i32], weights: &[f64], sostype: SOSType) -> Result<()> {
    if vars.len() != weights.len() {
      return Err(Error::InconsitentDims);
    }

    let sostype = match sostype {
      SOSType::SOSType1 => 1,
      SOSType::SOSType2 => 2,
    };

    let beg = 0;
    let error = unsafe {
      ffi::GRBaddsos(self.model,
                     1,
                     vars.len() as ffi::c_int,
                     &sostype,
                     &beg,
                     vars.as_ptr(),
                     weights.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  /// Query the value of a scalar attribute of the model.
  pub fn get<A: Attr>(&self, attr: A) -> Result<A::Output> { Attr::get(self, attr) }

  /// Set the value of a scalar attribute of the model.
  pub fn set<A: Attr>(&mut self, attr: A, value: A::Output) -> Result<()> { Attr::set(self, attr, value) }

  /// Query one of the value of a vector attribute from the model.
  pub fn get_element<A: AttrArray>(&self, attr: A, element: i32) -> Result<A::Output> {
    A::get_element(self, attr, element)
  }

  /// Set one of the value of a vector attribute to the model.
  pub fn set_element<A: AttrArray>(&mut self, attr: A, element: i32, value: A::Output) -> Result<()> {
    A::set_element(self, attr, element, value)
  }

  pub fn get_array<A: AttrArray>(&self, attr: A, first: usize, len: usize) -> Result<Vec<A::Output>> {
    A::get_array(self, attr, first, len)
  }

  pub fn set_array<A: AttrArray>(&mut self, attr: A, first: usize, values: &[A::Output]) -> Result<()> {
    A::set_array(self, attr, first, values)
  }

  pub fn get_list<A: AttrArray>(&self, attr: A, ind: &[i32]) -> Result<Vec<A::Output>> { A::get_list(self, attr, ind) }

  pub fn set_list<A: AttrArray>(&mut self, attr: A, ind: &[i32], values: &[A::Output]) -> Result<()> {
    A::set_list(self, attr, ind, values)
  }

  fn error_from_api(&self, errcode: ffi::c_int) -> Error { self.env.error_from_api(errcode) }
}

impl<'a> Drop for Model<'a> {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreemodel(self.model) };
    self.model = null_mut();
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
