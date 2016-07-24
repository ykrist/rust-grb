#![allow(improper_ctypes)]

extern crate libc;

pub use libc::c_int;
pub use libc::c_schar;

#[repr(C)]
pub struct GRBenv;

extern "C" {
  pub fn GRBloadenv(envP: *mut *mut GRBenv,
                    logfilename: *const c_schar)
                    -> c_int;
  pub fn GRBgeterrormsg(env: *mut GRBenv) -> *const c_schar;
  pub fn GRBfreeenv(env: *mut GRBenv);
}
