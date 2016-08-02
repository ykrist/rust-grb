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
  }
  //   fn get(&self, attr: IntAttr) -> Result<i32> {
  //     let mut value: ffi::c_int = 0;
  //     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
  //     let error =
  //       unsafe { ffi::GRBgetintattr(self.model, attrname.as_ptr(), &mut value) };
  //     if error != 0 {
  //       return Err(self.error_from_api(error));
  //     }
  //     Ok(value as i32)
  //   }

  fn set(&mut self, attr: A, value: Output) -> Result<()> {
    Err(Error::NotImplemented)
  }
  //   fn set(&mut self, attr: IntAttr, value: i32) -> Result<()> {
  //     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
  //     let error =
  //       unsafe { ffi::GRBsetintattr(self.model, attrname.as_ptr(), value) };
  //     if error != 0 {
  //       return Err(self.error_from_api(error));
  //     }
  //     Ok(())
  //   }

  fn get_element(&self, attr: A, element: i32) -> Result<Output> {
    Err(Error::NotImplemented)
  }

  //   fn get_element(&self, attr: IntAttr, element: i32) -> Result<i32> {
  //     let mut value: ffi::c_int = 0;
  //     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
  //     let error = unsafe {
  //       ffi::GRBgetintattrelement(self.model,
  //                                 attrname.as_ptr(),
  //                                 element,
  //                                 &mut value)
  //     };
  //     if error != 0 {
  //       return Err(self.error_from_api(error));
  //     }
  //     Ok(value as i32)
  //   }

  fn set_element(&mut self,
                 attr: A,
                 element: i32,
                 value: Output)
                 -> Result<()> {
    Err(Error::NotImplemented)
  }
  //   fn set_element(&mut self,
  //                  attr: IntAttr,
  //                  element: i32,
  //                  value: i32)
  //                  -> Result<()> {
  //     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
  //     let error = unsafe {
  //       ffi::GRBsetintattrelement(self.model, attrname.as_ptr(), element, value)
  //     };
  //     if error != 0 {
  //       return Err(self.error_from_api(error));
  //     }
  //     Ok(())
  //   }

  fn get_array(&self,
               attr: A,
               first: usize,
               len: usize)
               -> Result<Vec<Output>> {
    Err(Error::NotImplemented)
  }
  //   fn get_array(&self,
  //                attr: IntAttr,
  //                first: usize,
  //                len: usize)
  //                -> Result<Vec<i32>> {
  //     let mut values = Vec::with_capacity(len);
  //     values.resize(len, 0);
  //     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
  //     let error = unsafe {
  //       ffi::GRBgetintattrarray(self.model,
  //                               attrname.as_ptr(),
  //                               first as ffi::c_int,
  //                               len as ffi::c_int,
  //                               values.as_mut_ptr())
  //     };
  //     if error != 0 {
  //       return Err(self.error_from_api(error));
  //     }
  //     Ok(values)
  //   }

  fn set_array(&mut self,
               attr: A,
               first: usize,
               values: &[Output])
               -> Result<()> {
    Err(Error::NotImplemented)
  }
  //   fn set_array(&mut self,
  //                attr: IntAttr,
  //                first: usize,
  //                values: &[i32])
  //                -> Result<()> {
  //     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
  //     let error = unsafe {
  //       ffi::GRBsetintattrarray(self.model,
  //                               attrname.as_ptr(),
  //                               first as ffi::c_int,
  //                               values.len() as ffi::c_int,
  //                               values.as_ptr())
  //     };
  //     if error != 0 {
  //       return Err(self.error_from_api(error));
  //     }
  //     Ok(())
  //   }

  fn get_list(&self, attr: A, ind: &[i32]) -> Result<Vec<Output>> {
    Err(Error::NotImplemented)
  }

  //   fn get_list(&self, attr: IntAttr, ind: &[i32]) -> Result<Vec<i32>> {
  //     let mut values = Vec::with_capacity(ind.len());
  //     values.resize(ind.len(), 0);
  //     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
  //     let error = unsafe {
  //       ffi::GRBgetintattrlist(self.model,
  //                              attrname.as_ptr(),
  //                              ind.len() as ffi::c_int,
  //                              ind.as_ptr(),
  //                              values.as_mut_ptr())
  //     };
  //     if error != 0 {
  //       return Err(self.error_from_api(error));
  //     }
  //     Ok(values)
  //   }

  fn set_list(&mut self, attr: A, ind: &[i32], value: &[Output]) -> Result<()> {
    Err(Error::NotImplemented)
  }
  //   fn set_list(&mut self,
  //               attr: IntAttr,
  //               ind: &[i32],
  //               values: &[i32])
  //               -> Result<()> {
  //     if ind.len() != values.len() {
  //       return Err(Error::InconsitentDims);
  //     }
  //     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
  //     let error = unsafe {
  //       ffi::GRBsetintattrlist(self.model,
  //                              attrname.as_ptr(),
  //                              ind.len() as ffi::c_int,
  //                              ind.as_ptr(),
  //                              values.as_ptr())
  //     };
  //     if error != 0 {
  //       return Err(self.error_from_api(error));
  //     }
  //     Ok(())
  //   }
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



// impl<'a> HasAttr<IntAttr, i32> for Model<'a> {
//   fn get(&self, attr: IntAttr) -> Result<i32> {
//     let mut value: ffi::c_int = 0;
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error =
//       unsafe { ffi::GRBgetintattr(self.model, attrname.as_ptr(), &mut value) };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(value as i32)
//   }
//
//   fn set(&mut self, attr: IntAttr, value: i32) -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error =
//       unsafe { ffi::GRBsetintattr(self.model, attrname.as_ptr(), value) };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_element(&self, attr: IntAttr, element: i32) -> Result<i32> {
//     let mut value: ffi::c_int = 0;
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetintattrelement(self.model,
//                                 attrname.as_ptr(),
//                                 element,
//                                 &mut value)
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(value as i32)
//   }
//
//   fn set_element(&mut self,
//                  attr: IntAttr,
//                  element: i32,
//                  value: i32)
//                  -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetintattrelement(self.model, attrname.as_ptr(), element, value)
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//
//   fn get_array(&self,
//                attr: IntAttr,
//                first: usize,
//                len: usize)
//                -> Result<Vec<i32>> {
//     let mut values = Vec::with_capacity(len);
//     values.resize(len, 0);
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetintattrarray(self.model,
//                               attrname.as_ptr(),
//                               first as ffi::c_int,
//                               len as ffi::c_int,
//                               values.as_mut_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(values)
//   }
//
//   fn set_array(&mut self,
//                attr: IntAttr,
//                first: usize,
//                values: &[i32])
//                -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetintattrarray(self.model,
//                               attrname.as_ptr(),
//                               first as ffi::c_int,
//                               values.len() as ffi::c_int,
//                               values.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_list(&self, attr: IntAttr, ind: &[i32]) -> Result<Vec<i32>> {
//     let mut values = Vec::with_capacity(ind.len());
//     values.resize(ind.len(), 0);
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetintattrlist(self.model,
//                              attrname.as_ptr(),
//                              ind.len() as ffi::c_int,
//                              ind.as_ptr(),
//                              values.as_mut_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(values)
//   }
//
//   fn set_list(&mut self,
//               attr: IntAttr,
//               ind: &[i32],
//               values: &[i32])
//               -> Result<()> {
//     if ind.len() != values.len() {
//       return Err(Error::InconsitentDims);
//     }
//
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetintattrlist(self.model,
//                              attrname.as_ptr(),
//                              ind.len() as ffi::c_int,
//                              ind.as_ptr(),
//                              values.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
// }
//
// impl<'a> HasAttr<CharAttr, i8> for Model<'a> {
//   // GRBmodel does not have any scalar attribute typed `char`.
//   fn get(&self, _: CharAttr) -> Result<i8> {
//     Err(Error::NotImplemented)
//   }
//   fn set(&mut self, _: CharAttr, _: i8) -> Result<()> {
//     Err(Error::NotImplemented)
//   }
//
//   fn get_element(&self, attr: CharAttr, element: i32) -> Result<i8> {
//     let mut value: ffi::c_char = 0;
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetcharattrelement(self.model,
//                                  attrname.as_ptr(),
//                                  element,
//                                  &mut value)
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(value)
//   }
//
//   fn set_element(&mut self,
//                  attr: CharAttr,
//                  element: i32,
//                  value: i8)
//                  -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetcharattrelement(self.model, attrname.as_ptr(), element, value)
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//
//   fn get_array(&self,
//                attr: CharAttr,
//                first: usize,
//                len: usize)
//                -> Result<Vec<i8>> {
//     let mut values = Vec::with_capacity(len);
//     values.resize(len, 0);
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetcharattrarray(self.model,
//                                attrname.as_ptr(),
//                                first as ffi::c_int,
//                                len as ffi::c_int,
//                                values.as_mut_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(values)
//   }
//
//   fn set_array(&mut self,
//                attr: CharAttr,
//                first: usize,
//                values: &[i8])
//                -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetcharattrarray(self.model,
//                                attrname.as_ptr(),
//                                first as ffi::c_int,
//                                values.len() as ffi::c_int,
//                                values.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_list(&self, attr: CharAttr, ind: &[i32]) -> Result<Vec<i8>> {
//     let mut values = Vec::with_capacity(ind.len());
//     values.resize(ind.len(), 0);
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetcharattrlist(self.model,
//                               attrname.as_ptr(),
//                               ind.len() as ffi::c_int,
//                               ind.as_ptr(),
//                               values.as_mut_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(values)
//   }
//
//   fn set_list(&mut self,
//               attr: CharAttr,
//               ind: &[i32],
//               values: &[i8])
//               -> Result<()> {
//     if ind.len() != values.len() {
//       return Err(Error::InconsitentDims);
//     }
//
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetcharattrlist(self.model,
//                               attrname.as_ptr(),
//                               ind.len() as ffi::c_int,
//                               ind.as_ptr(),
//                               values.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
// }
//
// impl<'a> HasAttr<DoubleAttr, f64> for Model<'a> {
//   fn get(&self, attr: DoubleAttr) -> Result<f64> {
//     let mut value: ffi::c_double = 0.0;
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error =
//       unsafe { ffi::GRBgetdblattr(self.model, attrname.as_ptr(), &mut value) };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(value as f64)
//   }
//
//   fn set(&mut self, attr: DoubleAttr, value: f64) -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error =
//       unsafe { ffi::GRBsetdblattr(self.model, attrname.as_ptr(), value) };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_element(&self, attr: DoubleAttr, element: i32) -> Result<f64> {
//     let mut value: ffi::c_double = 0.0;
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetdblattrelement(self.model,
//                                 attrname.as_ptr(),
//                                 element,
//                                 &mut value)
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(value as f64)
//   }
//
//   fn set_element(&mut self,
//                  attr: DoubleAttr,
//                  element: i32,
//                  value: f64)
//                  -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetdblattrelement(self.model, attrname.as_ptr(), element, value)
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_array(&self,
//                attr: DoubleAttr,
//                first: usize,
//                len: usize)
//                -> Result<Vec<f64>> {
//     let mut values = Vec::with_capacity(len);
//     values.resize(len, 0.0);
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetdblattrarray(self.model,
//                               attrname.as_ptr(),
//                               first as ffi::c_int,
//                               len as ffi::c_int,
//                               values.as_mut_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(values)
//   }
//
//   fn set_array(&mut self,
//                attr: DoubleAttr,
//                first: usize,
//                values: &[f64])
//                -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetdblattrarray(self.model,
//                               attrname.as_ptr(),
//                               first as ffi::c_int,
//                               values.len() as ffi::c_int,
//                               values.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_list(&self, attr: DoubleAttr, ind: &[i32]) -> Result<Vec<f64>> {
//     let mut values = Vec::with_capacity(ind.len());
//     values.resize(ind.len(), 0.0);
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetdblattrlist(self.model,
//                              attrname.as_ptr(),
//                              ind.len() as ffi::c_int,
//                              ind.as_ptr(),
//                              values.as_mut_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(values)
//   }
//
//   fn set_list(&mut self,
//               attr: DoubleAttr,
//               ind: &[i32],
//               values: &[f64])
//               -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetdblattrlist(self.model,
//                              attrname.as_ptr(),
//                              ind.len() as ffi::c_int,
//                              ind.as_ptr(),
//                              values.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
// }
//
// impl<'a> HasAttr<StringAttr, String> for Model<'a> {
//   fn get(&self, attr: StringAttr) -> Result<String> {
//     let mut value = null();
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error =
//       unsafe { ffi::GRBgetstrattr(self.model, attrname.as_ptr(), &mut value) };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(unsafe { from_c_str(value).to_owned() })
//   }
//
//   fn set(&mut self, attr: StringAttr, value: String) -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let value = try!(make_c_str(value.as_str()));
//     let error = unsafe {
//       ffi::GRBsetstrattr(self.model, attrname.as_ptr(), value.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_element(&self, attr: StringAttr, element: i32) -> Result<String> {
//     let mut value = null();
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetstrattrelement(self.model,
//                                 attrname.as_ptr(),
//                                 element,
//                                 &mut value)
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(unsafe { from_c_str(value).to_owned() })
//   }
//
//   fn set_element(&mut self,
//                  attr: StringAttr,
//                  element: i32,
//                  value: String)
//                  -> Result<()> {
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let value = try!(make_c_str(value.as_str()));
//     let error = unsafe {
//       ffi::GRBsetstrattrelement(self.model,
//                                 attrname.as_ptr(),
//                                 element,
//                                 value.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_array(&self,
//                attr: StringAttr,
//                first: usize,
//                len: usize)
//                -> Result<Vec<String>> {
//     let mut values = Vec::with_capacity(len);
//     values.resize(len, null());
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetstrattrarray(self.model,
//                               attrname.as_ptr(),
//                               first as ffi::c_int,
//                               len as ffi::c_int,
//                               values.as_mut_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(values.into_iter()
//       .map(|s| unsafe { from_c_str(s).to_owned() })
//       .collect())
//   }
//
//   fn set_array(&mut self,
//                attr: StringAttr,
//                first: usize,
//                values: &[String])
//                -> Result<()> {
//     let values = values.into_iter().map(|s| make_c_str(s)).collect::<Vec<_>>();
//     if values.iter().any(|ref s| s.is_err()) {
//       return Err(Error::StringConversion);
//     }
//     let values =
//       values.into_iter().map(|s| s.unwrap().as_ptr()).collect::<Vec<_>>();
//
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetstrattrarray(self.model,
//                               attrname.as_ptr(),
//                               first as ffi::c_int,
//                               values.len() as ffi::c_int,
//                               values.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
//
//   fn get_list(&self, attr: StringAttr, ind: &[i32]) -> Result<Vec<String>> {
//     let mut values = Vec::with_capacity(ind.len());
//     values.resize(ind.len(), null());
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBgetstrattrlist(self.model,
//                              attrname.as_ptr(),
//                              ind.len() as ffi::c_int,
//                              ind.as_ptr(),
//                              values.as_mut_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(values.into_iter().map(|s| unsafe { from_c_str(s) }).collect())
//   }
//
//   fn set_list(&mut self,
//               attr: StringAttr,
//               ind: &[i32],
//               values: &[String])
//               -> Result<()> {
//
//     let values = values.into_iter().map(|s| make_c_str(s)).collect::<Vec<_>>();
//     if values.iter().any(|ref s| s.is_err()) {
//       return Err(Error::StringConversion);
//     }
//     let values =
//       values.into_iter().map(|s| s.unwrap().as_ptr()).collect::<Vec<_>>();
//
//     let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
//     let error = unsafe {
//       ffi::GRBsetstrattrlist(self.model,
//                              attrname.as_ptr(),
//                              ind.len() as ffi::c_int,
//                              ind.as_ptr(),
//                              values.as_ptr())
//     };
//     if error != 0 {
//       return Err(self.error_from_api(error));
//     }
//     Ok(())
//   }
// }
