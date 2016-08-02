#![allow(unused_variables)]

extern crate gurobi_sys as ffi;

pub use ffi::{IntAttr, DoubleAttr, CharAttr, StringAttr};
pub use ffi::IntAttr::*;
pub use ffi::DoubleAttr::*;
pub use ffi::CharAttr::*;
pub use ffi::StringAttr::*;

use error::{Error, Result};

/// provides function to query/set the value of attributes.
pub trait HasAttr<A, Output>
  where A: HasAttrAPI<Output>
{
  fn get(&self, attr: A) -> Result<Output> {
    Err(Error::NotImplemented)
    // let mut value: ffi::c_int = 0;
    // let mut value = null();

    // let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    // let error =
    //   unsafe { ffi::GRBgetintattr(self.model, attrname.as_ptr(), &mut value) };
    // if error != 0 {
    //   return Err(self.error_from_api(error));
    // }

    // Ok(value as i32)
    // Ok(unsafe { from_c_str(value).to_owned() })
  }

  fn set(&mut self, attr: A, value: Output) -> Result<()> {
    Err(Error::NotImplemented)
    // let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    // let error =
    //   unsafe { ffi::GRBsetintattr(self.model, attrname.as_ptr(), value) };
    // if error != 0 {
    //   return Err(self.error_from_api(error));
    // }
    // Ok(())
  }

  fn get_element(&self, attr: A, element: i32) -> Result<Output> {
    Err(Error::NotImplemented)
    // let mut value: ffi::c_int = 0;
    // let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    // let error = unsafe {
    //   ffi::GRBgetintattrelement(self.model,
    //                             attrname.as_ptr(),
    //                             element,
    //                             &mut value)
    // };
    // if error != 0 {
    //   return Err(self.error_from_api(error));
    // }
    // Ok(value as i32)
  }

  fn set_element(&mut self,
                 attr: A,
                 element: i32,
                 value: Output)
                 -> Result<()> {
    Err(Error::NotImplemented)
    // let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    // let error = unsafe {
    //   ffi::GRBsetintattrelement(self.model, attrname.as_ptr(), element, value)
    // };
    // if error != 0 {
    //   return Err(self.error_from_api(error));
    // }
    // Ok(())
  }

  fn get_array(&self,
               attr: A,
               first: usize,
               len: usize)
               -> Result<Vec<Output>> {
    Err(Error::NotImplemented)
    // let mut values = Vec::with_capacity(len);
    // values.resize(len, 0);
    // / values.resize(len, null());
    // let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    // let error = unsafe {
    //   ffi::GRBgetintattrarray(self.model,
    //                           attrname.as_ptr(),
    //                           first as ffi::c_int,
    //                           len as ffi::c_int,
    //                           values.as_mut_ptr())
    // };
    // if error != 0 {
    //   return Err(self.error_from_api(error));
    // }
    // Ok(values)
    // / Ok(values.into_iter()
    // /   .map(|s| unsafe { from_c_str(s).to_owned() })
    // /   .collect())
  }

  fn set_array(&mut self,
               attr: A,
               first: usize,
               values: &[Output])
               -> Result<()> {
    Err(Error::NotImplemented)
    // let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    // let error = unsafe {
    //   ffi::GRBsetintattrarray(self.model,
    //                           attrname.as_ptr(),
    //                           first as ffi::c_int,
    //                           values.len() as ffi::c_int,
    //                           values.as_ptr())
    // };
    // if error != 0 {
    //   return Err(self.error_from_api(error));
    // }
    // Ok(())
  }
  //     let values = values.into_iter().map(|s| make_c_str(s)).collect::<Vec<_>>();
  //     if values.iter().any(|ref s| s.is_err()) {
  //       return Err(Error::StringConversion);
  //     }
  //     let values =
  //       values.into_iter().map(|s| s.unwrap().as_ptr()).collect::<Vec<_>>();

  fn get_list(&self, attr: A, ind: &[i32]) -> Result<Vec<Output>> {
    Err(Error::NotImplemented)
    // let mut values = Vec::with_capacity(ind.len());
    // values.resize(ind.len(), 0);
    // let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    // let error = unsafe {
    //   ffi::GRBgetintattrlist(self.model,
    //                          attrname.as_ptr(),
    //                          ind.len() as ffi::c_int,
    //                          ind.as_ptr(),
    //                          values.as_mut_ptr())
    // };
    // if error != 0 {
    //   return Err(self.error_from_api(error));
    // }
    // Ok(values)
  }

  fn set_list(&mut self, attr: A, ind: &[i32], value: &[Output]) -> Result<()> {
    Err(Error::NotImplemented)
    // if ind.len() != values.len() {
    //   return Err(Error::InconsitentDims);
    // }
    // let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    // let error = unsafe {
    //   ffi::GRBsetintattrlist(self.model,
    //                          attrname.as_ptr(),
    //                          ind.len() as ffi::c_int,
    //                          ind.as_ptr(),
    //                          values.as_ptr())
    // };
    // if error != 0 {
    //   return Err(self.error_from_api(error));
    // }
    // Ok(())
  }
}


pub trait HasModelAPI {
  unsafe fn get_model(&self) -> *mut ffi::GRBmodel;

  // make an instance of error object related to C API.
  fn error_from_api(&self, errcode: ffi::c_int) -> Error;
}

pub trait HasAttrAPI<Output> {}

impl HasAttrAPI<i32> for IntAttr {}
impl HasAttrAPI<i8> for CharAttr {}
impl HasAttrAPI<f64> for DoubleAttr {}
impl HasAttrAPI<String> for StringAttr {}
