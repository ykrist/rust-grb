extern crate gurobi_sys as ffi;

use std::ptr::{null, null_mut};
use std::ffi::CString;
use error::{Error, Result};
use model::Model;
use param::{HasEnvAPI, HasParamAPI, HasParam};
use util;

pub mod param {

  extern crate gurobi_sys as ffi;

  // re-exports
  pub use ffi::{IntParam, DoubleParam, StringParam};
  pub use ffi::IntParam::*;
  pub use ffi::DoubleParam::*;
  pub use ffi::StringParam::*;

  use std::ffi::CString;
  use error::{Error, Result};
  use util;

  /// Provides general C API related to GRBenv.
  pub trait HasEnvAPI {
    unsafe fn get_env(&self) -> *mut ffi::GRBenv;
    fn get_error_msg(&self) -> String;
  }

  pub trait Into<T> {
    fn init() -> Self;
    fn into(self) -> T;
  }

  impl Into<i32> for ffi::c_int {
    fn init() -> i32 {
      0
    }

    fn into(self) -> i32 {
      self
    }
  }

  impl Into<f64> for ffi::c_double {
    fn init() -> ffi::c_double {
      0.0
    }

    fn into(self) -> f64 {
      self
    }
  }

  impl Into<String> for Vec<ffi::c_char> {
    fn init() -> Vec<ffi::c_char> {
      Vec::with_capacity(4096)
    }

    fn into(self) -> String {
      unsafe { util::from_c_str(self.as_ptr()) }
    }
  }


  /// Provides C APIs and some utility functions related to parameter access.
  pub trait HasParamAPI<Output> {
    type RawFrom;
    type RawTo;
    type Init: Into<Output>;

    unsafe fn get_param(env: *mut ffi::GRBenv,
                        paramname: ffi::c_str,
                        value: Self::RawFrom)
                        -> ffi::c_int;

    unsafe fn set_param(env: *mut ffi::GRBenv,
                        paramname: ffi::c_str,
                        value: Self::RawTo)
                        -> ffi::c_int;

    fn as_rawfrom(val: &mut Self::Init) -> Self::RawFrom;
    fn as_rawto(output: Output) -> Self::RawTo;
  }

  /// provides function to query/set the value of parameters.
  pub trait HasParam<P, Output>: HasEnvAPI
    where CString: From<P>,
          P: HasParamAPI<Output>
  {
    /// Query the value of a parameter.
    fn get(&self, param: P) -> Result<Output> {
      let mut value = Into::<Output>::init();
      let error = unsafe {
        P::get_param(self.get_env(),
                     CString::from(param).as_ptr(),
                     P::as_rawfrom(&mut value))
      };
      if error != 0 {
        return Err(Error::FromAPI(self.get_error_msg(), error));
      }
      Ok(Into::<_>::into(value))
    }

    /// Set the value of a parameter.
    fn set(&mut self, param: P, value: Output) -> Result<()> {
      let error = unsafe {
        P::set_param(self.get_env(),
                     CString::from(param).as_ptr(),
                     P::as_rawto(value))
      };
      if error != 0 {
        return Err(Error::FromAPI(self.get_error_msg(), error));
      }
      Ok(())
    }
  }


  impl HasParamAPI<i32> for IntParam {
    type Init = i32;
    type RawFrom = *mut ffi::c_int;
    type RawTo = ffi::c_int;

    #[inline(always)]
    unsafe fn get_param(env: *mut ffi::GRBenv,
                        paramname: ffi::c_str,
                        value: *mut ffi::c_int)
                        -> ffi::c_int {
      ffi::GRBgetintparam(env, paramname, value)
    }

    #[inline(always)]
    unsafe fn set_param(env: *mut ffi::GRBenv,
                        paramname: ffi::c_str,
                        value: ffi::c_int)
                        -> ffi::c_int {
      ffi::GRBsetintparam(env, paramname, value)
    }


    #[inline(always)]
    fn as_rawfrom(val: &mut i32) -> *mut ffi::c_int {
      val
    }

    #[inline(always)]
    fn as_rawto(output: i32) -> ffi::c_int {
      output
    }
  }

  impl HasParamAPI<f64> for DoubleParam {
    type Init = f64;
    type RawFrom = *mut ffi::c_double;
    type RawTo = ffi::c_double;

    #[inline(always)]
    unsafe fn get_param(env: *mut ffi::GRBenv,
                        paramname: ffi::c_str,
                        value: *mut ffi::c_double)
                        -> ffi::c_int {
      ffi::GRBgetdblparam(env, paramname, value)
    }

    #[inline(always)]
    unsafe fn set_param(env: *mut ffi::GRBenv,
                        paramname: ffi::c_str,
                        value: ffi::c_double)
                        -> ffi::c_int {
      ffi::GRBsetdblparam(env, paramname, value)
    }

    #[inline(always)]
    fn as_rawfrom(val: &mut f64) -> *mut ffi::c_double {
      val
    }

    #[inline(always)]
    fn as_rawto(output: f64) -> ffi::c_double {
      output
    }
  }


  impl HasParamAPI<String> for StringParam {
    type Init = Vec<ffi::c_char>;
    type RawFrom = *mut ffi::c_char;
    type RawTo = *const ffi::c_char;

    #[inline(always)]
    unsafe fn get_param(env: *mut ffi::GRBenv,
                        paramname: ffi::c_str,
                        value: *mut ffi::c_char)
                        -> ffi::c_int {
      ffi::GRBgetstrparam(env, paramname, value)
    }

    #[inline(always)]
    unsafe fn set_param(env: *mut ffi::GRBenv,
                        paramname: ffi::c_str,
                        value: *const ffi::c_char)
                        -> ffi::c_int {
      ffi::GRBsetstrparam(env, paramname, value)
    }

    #[inline(always)]
    fn as_rawfrom(val: &mut Vec<ffi::c_char>) -> *mut ffi::c_char {
      val.as_mut_ptr()
    }

    #[inline(always)]
    fn as_rawto(output: String) -> *const ffi::c_char {
      CString::new(output.as_str()).unwrap().as_ptr()
    }
  }

}


/// Gurobi environment object
pub struct Env {
  env: *mut ffi::GRBenv,
}

impl Env {
  /// create an environment with log file
  pub fn new(logfilename: &str) -> Result<Env> {
    let mut env = null_mut::<ffi::GRBenv>();
    let logfilename = try!(util::make_c_str(logfilename));
    let error = unsafe { ffi::GRBloadenv(&mut env, logfilename.as_ptr()) };
    if error != 0 {
      return Err(Error::FromAPI(util::get_error_msg_env(env), error));
    }
    Ok(Env { env: env })
  }

  /// create an empty model object associted with the environment.
  pub fn new_model(&self, modelname: &str) -> Result<Model> {
    let modelname = try!(util::make_c_str(modelname));
    let mut model = null_mut::<ffi::GRBmodel>();
    let error = unsafe {
      ffi::GRBnewmodel(self.env,
                       &mut model,
                       modelname.as_ptr(),
                       0,
                       null(),
                       null(),
                       null(),
                       null(),
                       null())
    };
    if error != 0 {
      return Err(Error::FromAPI(self.get_error_msg(), error));
    }

    Ok(Model::new(self, model))
  }
}

impl Drop for Env {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreeenv(self.env) };
    self.env = null_mut();
  }
}

impl HasEnvAPI for Env {
  unsafe fn get_env(&self) -> *mut ffi::GRBenv {
    self.env
  }

  fn get_error_msg(&self) -> String {
    util::get_error_msg_env(self.env)
  }
}

impl<P, Output> HasParam<P, Output> for Env
  where CString: From<P>,
        P: HasParamAPI<Output>
{
}


// #[test]
// fn env_with_logfile() {
//   use std::path::Path;
//   use std::fs::remove_file;
//
//   let path = Path::new("test_env.log");
//
//   if path.exists() {
//     remove_file(path).unwrap();
//   }
//
//   {
//     let env = Env::new(path.to_str().unwrap()).unwrap();
//   }
//
//   assert!(path.exists());
//   remove_file(path).unwrap();
// }

#[cfg(test)]
mod test {
  use param;
  use env::Env;
  use param::HasParam;

  #[test]
  fn param_accesors_should_be_valid() {
    let mut env = Env::new("").unwrap();
    env.set(param::IntParam::IISMethod, 1).unwrap();
    let iis_method = env.get(param::IntParam::IISMethod).unwrap();
    assert_eq!(iis_method, 1);
  }
}
