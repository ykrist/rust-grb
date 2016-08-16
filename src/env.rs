extern crate gurobi_sys as ffi;

use std::ptr::{null, null_mut};
use std::ffi::CString;
use error::{Error, Result};
use model::Model;
use util;

use types::{Init, Into, AsRawPtr, FromRaw};


pub mod param {
  // re-exports
  pub use ffi::{IntParam, DoubleParam, StringParam};
  pub use ffi::IntParam::*;
  pub use ffi::DoubleParam::*;
  pub use ffi::StringParam::*;
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
      return Err(self.error_from_api(error));
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


/// Provides general C API related to GRBenv.
pub trait EnvAPI {
  unsafe fn get_env(&self) -> *mut ffi::GRBenv;
  fn error_from_api(&self, ffi::c_int) -> Error;
}

impl EnvAPI for Env {
  unsafe fn get_env(&self) -> *mut ffi::GRBenv {
    self.env
  }

  fn error_from_api(&self, error: ffi::c_int) -> Error {
    Error::FromAPI(util::get_error_msg_env(self.env), error)
  }
}


/// provides function to query/set the value of parameters.
pub trait Param<P, Output>: EnvAPI
  where CString: From<P>,
        P: ParamAPI<Output>
{
  /// Query the value of a parameter.
  fn get(&self, param: P) -> Result<Output> {
    let mut value = Init::init();
    let error = unsafe {
      P::get_param(self.get_env(),
                   CString::from(param).as_ptr(),
                   P::as_rawfrom(&mut value))
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(Into::<_>::into(value))
  }

  /// Set the value of a parameter.
  fn set(&mut self, param: P, value: Output) -> Result<()> {
    let error = unsafe {
      P::set_param(self.get_env(),
                   CString::from(param).as_ptr(),
                   P::to_rawto(value))
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }
}

impl<P, Output> Param<P, Output> for Env
  where CString: From<P>,
        P: ParamAPI<Output>
{
}


/// Provides C APIs and some utility functions related to parameter access.
pub trait ParamAPI<Output> {
  type RawFrom;
  type RawTo: FromRaw<Output>;
  type Buf: Init + Into<Output> + AsRawPtr<Self::RawFrom>;

  unsafe fn get_param(env: *mut ffi::GRBenv,
                      paramname: ffi::c_str,
                      value: Self::RawFrom)
                      -> ffi::c_int;

  unsafe fn set_param(env: *mut ffi::GRBenv,
                      paramname: ffi::c_str,
                      value: Self::RawTo)
                      -> ffi::c_int;

  fn as_rawfrom(val: &mut Self::Buf) -> Self::RawFrom {
    val.as_rawptr()
  }

  fn to_rawto(output: Output) -> Self::RawTo {
    FromRaw::<Output>::from(output)
  }
}


impl ParamAPI<i32> for param::IntParam {
  type Buf = i32;
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
}

impl ParamAPI<f64> for param::DoubleParam {
  type Buf = f64;
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
}


impl ParamAPI<String> for param::StringParam {
  type Buf = Vec<ffi::c_char>;
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
  use env::param;
  use env::{Env, Param};

  #[test]
  fn param_accesors_should_be_valid() {
    let mut env = Env::new("").unwrap();
    env.set(param::IntParam::IISMethod, 1).unwrap();
    let iis_method = env.get(param::IntParam::IISMethod).unwrap();
    assert_eq!(iis_method, 1);
  }
}
