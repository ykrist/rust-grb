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

/// Gurobi environment object (see the Gurobi [manual](https://www.gurobi.com/documentation/9.1/refman/environments.html))
pub struct Env {
  env: *mut ffi::GRBenv,
  require_drop: bool
}

/// A Gurobi environment which hasn't been started yet. Some Gurobi parameters,
/// such as [`Record`](https://www.gurobi.com/documentation/9.1/refman/record.html)
/// need to be set before the environment has been started.
///
/// # Examples
/// ```
/// use gurobi::*;
/// let mut env = Env::empty()?;
/// env.set(param::OutputFlag, 0)?
///   .set(param::UpdateMode, 1)?
///   .set(param::LogFile, "".to_string())?;
/// let env = env.start();
/// # Ok::<(), gurobi::Error>(())
/// ```
pub struct EmptyEnv {
  env : Env
}

impl EmptyEnv {
  /// Query a parameter value
  pub fn get<P: Param>(&self, param : P) -> Result<P::Value> {
    self.env.get(param)
  }

  /// Set a parameter value
  pub fn set<P: Param>(&mut self, param : P, value: P::Value) -> Result<&mut Self> {
    self.env.set(param, value)?;
    Ok(self)
  }

  /// Start the environment, return the [`Env`] on success.
  pub fn start(self) -> Result<Env> {
    self.env.check_apicall(unsafe { ffi::GRBstartenv(self.env.get_ptr()) })?;
    Ok(self.env)
  }
}

impl Env {
  pub(crate) fn from_raw(env: *mut ffi::GRBenv) -> Env {
    Env {
      env,
      require_drop: false // TODO this seems sketchy, should use Rc instead
    }
  }

  /// Create a new empty and un-started environment.
  pub fn empty() -> Result<EmptyEnv> {
    let mut env = null_mut();
    let err_code = unsafe { ffi::GRBemptyenv(&mut env) };
    if err_code != 0 {
      return Err(Error::FromAPI(get_error_msg(env), err_code));
    }
    let env = Env { env, require_drop: true };
    Ok(EmptyEnv{env})
  }

  /// Create an environment with log file
  ///
  /// Setting `logfilename` to an empty string will not create a logfile.
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
