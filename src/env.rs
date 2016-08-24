// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use super::ffi;

use std::ptr::{null, null_mut};
use std::ffi::CString;
use error::{Error, Result};
use model::Model;
use util;


/// Defines the name of parameters
pub mod param {
  pub use ffi::{IntParam, DoubleParam, StringParam};

  // re-exports
  pub use self::IntParam::*;
  pub use self::DoubleParam::*;
  pub use self::StringParam::*;
}


/// Gurobi environment object
pub struct Env {
  env: *mut ffi::GRBenv
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

  /// Query the value of a parameter.
  pub fn get<P: Param>(&self, param: P) -> Result<P::Out> { Param::get(self, param) }

  /// Set the value of a parameter.
  pub fn set<P: Param>(&mut self, param: P, value: P::Out) -> Result<()> { Param::set(self, param, value) }

  pub fn error_from_api(&self, error: ffi::c_int) -> Error { Error::FromAPI(util::get_error_msg_env(self.env), error) }
}

impl Drop for Env {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreeenv(self.env) };
    self.env = null_mut();
  }
}


/// provides function to query/set the value of parameters.
pub trait Param: Sized + Into<CString> {
  type Out;
  type Buf: util::Init + util::Into<Self::Out> + util::AsRawPtr<Self::RawFrom>;
  type RawFrom;
  type RawTo: util::FromRaw<Self::Out>;

  #[allow(unused_imports)]
  fn get(env: &Env, param: Self) -> Result<Self::Out> {
    use util::{AsRawPtr, Into};

    let mut value: Self::Buf = util::Init::init();
    let error = unsafe { Self::get_param(env.env, param.into().as_ptr(), value.as_rawptr()) };
    if error != 0 {
      return Err(env.error_from_api(error));
    }

    Ok(util::Into::into(value))
  }

  fn set(env: &mut Env, param: Self, value: Self::Out) -> Result<()> {
    let error = unsafe { Self::set_param(env.env, param.into().as_ptr(), util::FromRaw::from(value)) };
    if error != 0 {
      return Err(env.error_from_api(error));
    }

    Ok(())
  }

  #[inline(always)]
  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: Self::RawFrom) -> ffi::c_int;

  #[inline(always)]
  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: Self::RawTo) -> ffi::c_int;
}


impl Param for param::IntParam {
  type Out = i32;
  type Buf = ffi::c_int;
  type RawFrom = *mut ffi::c_int;
  type RawTo = ffi::c_int;

  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_int) -> ffi::c_int {
    ffi::GRBgetintparam(env, paramname, value)
  }

  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: ffi::c_int) -> ffi::c_int {
    ffi::GRBsetintparam(env, paramname, value)
  }
}

impl Param for param::DoubleParam {
  type Out = f64;
  type Buf = ffi::c_double;
  type RawFrom = *mut ffi::c_double;
  type RawTo = ffi::c_double;

  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_double) -> ffi::c_int {
    ffi::GRBgetdblparam(env, paramname, value)
  }

  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: ffi::c_double) -> ffi::c_int {
    ffi::GRBsetdblparam(env, paramname, value)
  }
}


impl Param for param::StringParam {
  type Out = String;
  type Buf = Vec<ffi::c_char>;
  type RawFrom = *mut ffi::c_char;
  type RawTo = *const ffi::c_char;

  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_char) -> ffi::c_int {
    ffi::GRBgetstrparam(env, paramname, value)
  }

  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *const ffi::c_char) -> ffi::c_int {
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
  use env::Env;

  #[test]
  fn param_accesors_should_be_valid() {
    let mut env = Env::new("").unwrap();
    env.set(param::IISMethod, 1).unwrap();
    let iis_method = env.get(param::IISMethod).unwrap();
    assert_eq!(iis_method, 1);
  }
}
