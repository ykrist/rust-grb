// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use super::ffi;

use std::ptr::{null, null_mut};
use error::{Error, Result};
use model::Model;
use util;
use parameter;

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
  #[allow(unused_imports)]
  pub fn get<P: parameter::ParamBase>(&self, param: P) -> Result<P::Out> {
    use util::{AsRawPtr, Into};

    let mut value: P::Buf = util::Init::init();
    let error = unsafe { P::get_param(self.env, param.into().as_ptr(), value.as_rawptr()) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(util::Into::into(value))
  }

  /// Set the value of a parameter.
  pub fn set<P: parameter::ParamBase>(&mut self, param: P, value: P::Out) -> Result<()> {
    let error = unsafe { P::set_param(self.env, param.into().as_ptr(), util::FromRaw::from(value)) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }
}

pub trait ErrorFromAPI {
  fn error_from_api(&self, error: ffi::c_int) -> Error;
}

impl ErrorFromAPI for Env {
  fn error_from_api(&self, error: ffi::c_int) -> Error { Error::FromAPI(util::get_error_msg_env(self.env), error) }
}

impl Drop for Env {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreeenv(self.env) };
    self.env = null_mut();
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

#[test]
fn param_accesors_should_be_valid() {
  use super::param;
  let mut env = Env::new("").unwrap();
  env.set(param::IISMethod, 1).unwrap();
  let iis_method = env.get(param::IISMethod).unwrap();
  assert_eq!(iis_method, 1);
}
