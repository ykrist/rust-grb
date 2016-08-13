#![allow(unused_variables)]

extern crate gurobi_sys as ffi;

pub use ffi::{IntAttr, DoubleAttr, CharAttr, StringAttr};
pub use ffi::IntAttr::*;
pub use ffi::DoubleAttr::*;
pub use ffi::CharAttr::*;
pub use ffi::StringAttr::*;

use std::iter;
use std::ffi::CString;
use error::{Error, Result};

/// provides function to query/set the value of attributes.
pub trait HasAttr<A, Output: Clone>: HasModelAPI
  where A: HasAttrAPI<Output>,
        CString: From<A>
{
  fn get(&self, attr: A) -> Result<Output> {
    let mut value = A::init();

    let error = unsafe {
      A::get_attr(self.get_model(),
                  CString::from(attr).as_ptr(),
                  A::as_rawget(&mut value))
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(A::to_out(value))
  }

  fn set(&mut self, attr: A, value: Output) -> Result<()> {
    let error = unsafe {
      A::set_attr(self.get_model(),
                  CString::from(attr).as_ptr(),
                  A::to_rawset(value))
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  fn get_element(&self, attr: A, element: i32) -> Result<Output> {
    let mut value = A::init();

    let error = unsafe {
      A::get_attrelement(self.get_model(),
                         CString::from(attr).as_ptr(),
                         element,
                         A::as_rawget(&mut value))
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(A::to_out(value))
  }

  fn set_element(&mut self,
                 attr: A,
                 element: i32,
                 value: Output)
                 -> Result<()> {
    let error = unsafe {
      A::set_attrelement(self.get_model(),
                         CString::from(attr).as_ptr(),
                         element,
                         A::to_rawset(value))
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  fn get_array(&self,
               attr: A,
               first: usize,
               len: usize)
               -> Result<Vec<Output>> {
    let mut values = A::init_array(len);

    let error = unsafe {
      A::get_attrarray(self.get_model(),
                       CString::from(attr).as_ptr(),
                       first as ffi::c_int,
                       len as ffi::c_int,
                       values.as_mut_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(values.into_iter()
      .map(|s| A::to_out(s))
      .collect())
  }

  fn set_array(&mut self,
               attr: A,
               first: usize,
               values: &[Output])
               -> Result<()> {
    let values = A::to_rawsets(values);

    let error = unsafe {
      A::set_attrarray(self.get_model(),
                       CString::from(attr).as_ptr(),
                       first as ffi::c_int,
                       values.len() as ffi::c_int,
                       values.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  fn get_list(&self, attr: A, ind: &[i32]) -> Result<Vec<Output>> {
    let mut values = A::init_array(ind.len());

    let error = unsafe {
      A::get_attrlist(self.get_model(),
                      CString::from(attr).as_ptr(),
                      ind.len() as ffi::c_int,
                      ind.as_ptr(),
                      values.as_mut_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(values.into_iter()
      .map(|s| A::to_out(s))
      .collect())
  }

  fn set_list(&mut self,
              attr: A,
              ind: &[i32],
              values: &[Output])
              -> Result<()> {
    if ind.len() != values.len() {
      return Err(Error::InconsitentDims);
    }

    let values = A::to_rawsets(values);

    let error = unsafe {
      A::set_attrlist(self.get_model(),
                      CString::from(attr).as_ptr(),
                      ind.len() as ffi::c_int,
                      ind.as_ptr(),
                      values.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }
}


pub trait HasModelAPI {
  unsafe fn get_model(&self) -> *mut ffi::GRBmodel;

  // make an instance of error object related to C API.
  fn error_from_api(&self, errcode: ffi::c_int) -> Error;
}

pub trait HasAttrAPI<Output: Clone> {
  type Init: Clone;
  type RawGet;
  type RawSet;

  unsafe fn get_attr(model: *mut ffi::GRBmodel,
                     attrname: ffi::c_str,
                     value: *mut Self::RawGet)
                     -> ffi::c_int;

  unsafe fn set_attr(model: *mut ffi::GRBmodel,
                     attrname: ffi::c_str,
                     value: Self::RawSet)
                     -> ffi::c_int;
  unsafe fn get_attrelement(model: *mut ffi::GRBmodel,
                            attrname: ffi::c_str,
                            element: ffi::c_int,
                            value: *mut Self::RawGet)
                            -> ffi::c_int;

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel,
                            attrname: ffi::c_str,
                            element: ffi::c_int,
                            value: Self::RawSet)
                            -> ffi::c_int;

  unsafe fn get_attrarray(model: *mut ffi::GRBmodel,
                          attrname: ffi::c_str,
                          first: ffi::c_int,
                          len: ffi::c_int,
                          values: *mut Self::Init)
                          -> ffi::c_int;

  unsafe fn set_attrarray(model: *mut ffi::GRBmodel,
                          attrname: ffi::c_str,
                          first: ffi::c_int,
                          len: ffi::c_int,
                          values: *const Self::RawSet)
                          -> ffi::c_int;

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel,
                         attrname: ffi::c_str,
                         len: ffi::c_int,
                         ind: *const ffi::c_int,
                         values: *mut Self::Init)
                         -> ffi::c_int;

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel,
                         attrname: ffi::c_str,
                         len: ffi::c_int,
                         ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int;



  fn init() -> Self::Init;
  // fn init() -> ffi::c_int { 0 }
  // fn init() -> ffi::c_str { null() }

  fn to_out(val: Self::Init) -> Output;
  // fn to_out(val: ffi::c_int) -> i32 { val as ffi::c_int }
  // fn to_out(val: ffi::c_str) -> String { unsafe { util::from_c_str(val).to_owned() } }

  fn as_rawget(val: &mut Self::Init) -> *mut Self::RawGet;
  fn to_rawset(val: Output) -> Self::RawSet;

  fn init_array(len: usize) -> Vec<Self::Init> {
    iter::repeat(Self::init()).take(len).collect()
  }

  fn to_rawsets(values: &[Output]) -> Vec<Self::RawSet> {
    values.iter().map(|v| Self::to_rawset(v.clone())).collect()
  }
  // fn to_rawsets(values: Vec<String>) -> Vec<ffi::c_str> {
  //     let values = values.into_iter().map(|s| make_c_str(s)).collect::<Vec<_>>();
  //     if values.iter().any(|ref s| s.is_err()) {
  //       return Err(Error::StringConversion);
  //     }
  //     values.into_iter().map(|s| s.unwrap().as_ptr()).collect()
  // }
}

impl HasAttrAPI<i32> for IntAttr {
  type Init = i32;
  type RawGet = ffi::c_int;
  type RawSet = ffi::c_int;

  unsafe fn get_attr(model: *mut ffi::GRBmodel,
                     attrname: ffi::c_str,
                     value: *mut Self::RawGet)
                     -> ffi::c_int {
    ffi::GRBgetintattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel,
                     attrname: ffi::c_str,
                     value: Self::RawSet)
                     -> ffi::c_int {
    ffi::GRBsetintattr(model, attrname, value)
  }

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel,
                            attrname: ffi::c_str,
                            element: ffi::c_int,
                            value: *mut Self::RawGet)
                            -> ffi::c_int {
    ffi::GRBgetintattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel,
                            attrname: ffi::c_str,
                            element: ffi::c_int,
                            value: Self::RawSet)
                            -> ffi::c_int {
    ffi::GRBsetintattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrarray(model: *mut ffi::GRBmodel,
                          attrname: ffi::c_str,
                          first: ffi::c_int,
                          len: ffi::c_int,
                          values: *mut Self::RawGet)
                          -> ffi::c_int {
    ffi::GRBgetintattrarray(model, attrname, first, len, values)
  }

  unsafe fn set_attrarray(model: *mut ffi::GRBmodel,
                          attrname: ffi::c_str,
                          first: ffi::c_int,
                          len: ffi::c_int,
                          values: *const Self::RawSet)
                          -> ffi::c_int {
    ffi::GRBsetintattrarray(model, attrname, first, len, values)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel,
                         attrname: ffi::c_str,
                         len: ffi::c_int,
                         ind: *const ffi::c_int,
                         values: *mut Self::RawGet)
                         -> ffi::c_int {
    ffi::GRBgetintattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel,
                         attrname: ffi::c_str,
                         len: ffi::c_int,
                         ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int {
    ffi::GRBsetintattrlist(model, attrname, len, ind, values)
  }


  fn init() -> ffi::c_int {
    0
  }

  fn to_out(val: ffi::c_int) -> i32 {
    val as ffi::c_int
  }

  fn as_rawget(val: &mut Self::Init) -> *mut Self::RawGet {
    val
  }
  fn to_rawset(val: i32) -> Self::RawSet {
    val
  }
}

impl HasAttrAPI<f64> for DoubleAttr {
  type Init = f64;
  type RawGet = ffi::c_double;
  type RawSet = ffi::c_double;

  unsafe fn get_attr(model: *mut ffi::GRBmodel,
                     attrname: ffi::c_str,
                     value: *mut Self::RawGet)
                     -> ffi::c_int {
    ffi::GRBgetdblattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel,
                     attrname: ffi::c_str,
                     value: Self::RawSet)
                     -> ffi::c_int {
    ffi::GRBsetdblattr(model, attrname, value)
  }

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel,
                            attrname: ffi::c_str,
                            element: ffi::c_int,
                            value: *mut Self::RawGet)
                            -> ffi::c_int {
    ffi::GRBgetdblattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel,
                            attrname: ffi::c_str,
                            element: ffi::c_int,
                            value: Self::RawSet)
                            -> ffi::c_int {
    ffi::GRBsetdblattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrarray(model: *mut ffi::GRBmodel,
                          attrname: ffi::c_str,
                          first: ffi::c_int,
                          len: ffi::c_int,
                          values: *mut Self::RawGet)
                          -> ffi::c_int {
    ffi::GRBgetdblattrarray(model, attrname, first, len, values)
  }

  unsafe fn set_attrarray(model: *mut ffi::GRBmodel,
                          attrname: ffi::c_str,
                          first: ffi::c_int,
                          len: ffi::c_int,
                          values: *const Self::RawSet)
                          -> ffi::c_int {
    ffi::GRBsetdblattrarray(model, attrname, first, len, values)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel,
                         attrname: ffi::c_str,
                         len: ffi::c_int,
                         ind: *const ffi::c_int,
                         values: *mut Self::RawGet)
                         -> ffi::c_int {
    ffi::GRBgetdblattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel,
                         attrname: ffi::c_str,
                         len: ffi::c_int,
                         ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int {
    ffi::GRBsetdblattrlist(model, attrname, len, ind, values)
  }


  fn init() -> ffi::c_double {
    0.0
  }

  fn to_out(val: ffi::c_double) -> f64 {
    val as ffi::c_double
  }

  fn as_rawget(val: &mut Self::Init) -> *mut Self::RawGet {
    val
  }
  fn to_rawset(val: f64) -> Self::RawSet {
    val
  }
}

// impl HasAttrAPI<i8> for CharAttr {}
// impl HasAttrAPI<f64> for DoubleAttr {}
// impl HasAttrAPI<String> for StringAttr {}
