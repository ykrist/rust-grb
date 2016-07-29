extern crate gurobi_sys as ffi;

use std::ffi::{CStr, CString};
use error::{Error, Result};

pub fn get_error_msg_env(env: *mut ffi::GRBenv) -> String {
  unsafe { from_c_str(ffi::GRBgeterrormsg(env)) }
}

pub fn make_c_str(s: &str) -> Result<CString> {
  CString::new(s).map_err(|e| Error::NulError(e))
}

pub unsafe fn from_c_str(s: *const ffi::c_char) -> String {
  CStr::from_ptr(s).to_string_lossy().into_owned()
}


#[test]
fn conversion_must_success() {
  let s1 = "mip1.log";
  let s2 = unsafe { from_c_str(make_c_str(s1).unwrap().as_ptr()) };
  assert!(s1 == s2);
}
