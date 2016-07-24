extern crate gurobi_sys as ffi;

use std::ptr::null_mut;
use std::ffi::{CStr, CString};

/// represents error information which called the API.
#[derive(Debug)]
pub enum Error {
  /// This function has yet implemented
  NotImplemented,
  /// An exception returned from Gurobi C API
  FromAPI(String, ffi::c_int),
  /// see https://doc.rust-lang.org/std/ffi/struct.NulError.html
  NulError(std::ffi::NulError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Gurobi environment object
pub struct Env {
  env: *mut ffi::GRBenv,
}

impl Env {
  /// create an empty environment with log file
  pub fn new(logfilename: &str) -> Result<Env> {
    let logfilename = try!(CString::new(logfilename)
      .map_err(|e| Error::NulError(e)));
    let mut env: *mut ffi::GRBenv = null_mut();
    let error = unsafe { ffi::GRBloadenv(&mut env, logfilename.as_ptr()) };
    if error != 0 {
      return Err(Error::FromAPI(get_error_msg_env(env), error));
    }
    Ok(Env { env: env })
  }

  fn get_error_msg(&self) -> String {
    get_error_msg_env(self.env)
  }
}

impl Drop for Env {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreeenv(self.env) };
    self.env = null_mut();
  }
}

fn get_error_msg_env(env: *mut ffi::GRBenv) -> String {
  unsafe {
    CStr::from_ptr(ffi::GRBgeterrormsg(env)).to_string_lossy().into_owned()
  }
}

pub struct Model<'a> {
  model: *mut ffi::GRBmodel,
  env: &'a Env,
}

impl<'a> Model<'a> {
  pub fn new(env: &'a Env) -> Result<Model<'a>> {
    let mut model: *mut ffi::GRBmodel = null_mut();
    let error = unsafe {
      ffi::GRBnewmodel(env.env,
                       &mut model,
                       null_mut(),
                       0,
                       null_mut(),
                       null_mut(),
                       null_mut(),
                       null_mut(),
                       null_mut())
    };
    if error != 0 {
      return Err(Error::FromAPI(env.get_error_msg(), error));
    }
    Ok(Model {
      model: model,
      env: env,
    })
  }

  fn update(&mut self) -> Result<()> {
      Ok(())
  }

  pub fn optimize(&mut self) -> Result<()> {
      try!(self.update());

      let error = unsafe { ffi::GRBoptimize(self.model) };
      if error != 0 {
        return Err(Error::FromAPI(self.get_error_msg(), error));
      }

      Ok(())
  }

  fn get_error_msg(&self) -> String {
    self.env.get_error_msg()
  }
}

impl<'a> Drop for Model<'a> {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreemodel(self.model) };
    self.model = null_mut();
  }
}
