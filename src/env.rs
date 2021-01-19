// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use gurobi_sys as ffi;

use std::ffi::CString;
use std::ptr::null_mut;

use crate::error::{Error, Result};
use crate::model::Model;
use crate::param::Param;
use crate::util;

/// Gurobi environment object
pub struct Env {
  env: *mut ffi::GRBenv,
  require_drop: bool
}

impl Env {
  pub(crate) fn from_raw(env: *mut ffi::GRBenv) -> Env {
    Env {
      env,
      require_drop: false // TODO this seems sketchy, should use Rc instead
    }
  }

  /// Create an environment with log file
  pub fn new(logfilename: &str) -> Result<Env> {
    let mut env = null_mut();
    let logfilename = CString::new(logfilename)?;
    let error = unsafe { ffi::GRBloadenv(&mut env, logfilename.as_ptr()) };
    if error != 0 {
      return Err(Error::FromAPI(get_error_msg(env), error));
    }
    Ok(Env {
      env,
      require_drop: true
    })
  }

  /// Create a client environment on a computer server with log file
  pub fn new_client(logfilename: &str, computeserver: &str, port: i32, password: &str, priority: i32, timeout: f64)
                    -> Result<Env> {
    let mut env = null_mut();
    let logfilename = CString::new(logfilename)?;
    let computeserver = CString::new(computeserver)?;
    let password = CString::new(password)?;
    let error = unsafe {
      ffi::GRBloadclientenv(&mut env,
                            logfilename.as_ptr(),
                            computeserver.as_ptr(),
                            port,
                            password.as_ptr(),
                            priority,
                            timeout)
    };
    if error != 0 {
      return Err(Error::FromAPI(get_error_msg(env), error));
    }
    Ok(Env {
      env,
      require_drop: true
    })
  }

  /// Create an empty Gurobi model from the environment
  #[deprecated]
  pub fn new_model(&self, modelname: &str) -> Result<Model> { Model::new(modelname, self) }

  /// Read a model from a file
  #[deprecated]
  pub fn read_model(&self, filename: &str) -> Result<Model> { Model::read_from(filename, self) }

  /// Query the value of a parameter
  pub fn get<P: Param>(&self, param: P) -> Result<P::Value> {
    unsafe { param.get_param(self.env) }.map_err(|code| self.error_from_api(code))
  }

  /// Set the value of a parameter
  pub fn set<P: Param>(&mut self, param: P, value: P::Value) -> Result<()> {
    unsafe { param.set_param(self.env, value) }.map_err(|code| self.error_from_api(code))
  }

  /// Import a set of parameter values from a file
  pub fn read_params(&mut self, filename: &str) -> Result<()> {
    let filename = CString::new(filename)?;
    self.check_apicall(unsafe { ffi::GRBreadparams(self.env, filename.as_ptr()) })
  }

  /// Write the set of parameter values to a file
  pub fn write_params(&self, filename: &str) -> Result<()> {
    let filename = CString::new(filename)?;
    self.check_apicall(unsafe { ffi::GRBwriteparams(self.env, filename.as_ptr()) })
  }

  /// Insert a message into log file.
  ///
  /// When **message** cannot convert to raw C string, a panic is occurred.
  pub fn message(&self, message: &str) {
    let msg = CString::new(message).unwrap();
    unsafe { ffi::GRBmsg(self.env, msg.as_ptr()) };
  }

  pub(crate) fn error_from_api(&self, error: ffi::c_int) -> Error {
    Error::FromAPI(get_error_msg(self.env), error)
  }

  pub(crate) fn get_ptr(&self) -> *mut ffi::GRBenv { self.env }

  pub(crate) fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }
}


impl Drop for Env {
  fn drop(&mut self) {
    if self.require_drop {
      debug_assert!(!self.env.is_null());
      unsafe { ffi::GRBfreeenv(self.env) };
      self.env = null_mut();
    }
  }
}

fn get_error_msg(env: *mut ffi::GRBenv) -> String { unsafe { util::copy_c_str(ffi::GRBgeterrormsg(env)) } }


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
fn param_get_set() {
  use super::*;
  let mut env = Env::new("").unwrap();
  env.set(param::IISMethod, 1).unwrap();
  assert_eq!(env.get(param::IISMethod).unwrap(), 1);
  env.set(param::IISMethod, 0).unwrap();
  assert_eq!(env.get(param::IISMethod).unwrap(), 0);
  assert!(env.set(param::IISMethod, 9999).is_err());
}
