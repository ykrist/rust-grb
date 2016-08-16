extern crate gurobi_sys as ffi;

use util;
use std::ffi::CString;


/// make an empty instance.
pub trait Init {
  fn init() -> Self;
}

impl Init for ffi::c_int {
  fn init() -> i32 { 0 }
}

impl Init for ffi::c_double {
  fn init() -> ffi::c_double { 0.0 }
}

impl Init for Vec<ffi::c_char> {
  fn init() -> Vec<ffi::c_char> { Vec::with_capacity(4096) }
}


/// convert into different type.
pub trait Into<T> {
  fn into(self) -> T;
}

impl Into<i32> for ffi::c_int {
  fn into(self) -> i32 { self }
}

impl Into<f64> for ffi::c_double {
  fn into(self) -> f64 { self }
}

impl Into<String> for Vec<ffi::c_char> {
  fn into(self) -> String { unsafe { util::from_c_str(self.as_ptr()) } }
}


/// convert to Raw C Pointer.
pub trait AsRawPtr<T> {
  fn as_rawptr(&mut self) -> T;
}

impl AsRawPtr<*mut ffi::c_int> for i32 {
  fn as_rawptr(&mut self) -> *mut ffi::c_int { self }
}

impl AsRawPtr<*mut ffi::c_double> for f64 {
  fn as_rawptr(&mut self) -> *mut ffi::c_double { self }
}

impl AsRawPtr<*mut ffi::c_char> for Vec<ffi::c_char> {
  fn as_rawptr(&mut self) -> *mut ffi::c_char { self.as_mut_ptr() }
}



pub trait FromRaw<T> {
  fn from(T) -> Self;
}

impl FromRaw<i32> for ffi::c_int {
  fn from(val: i32) -> ffi::c_int { val }
}

impl FromRaw<f64> for ffi::c_double {
  fn from(val: f64) -> ffi::c_double { val }
}

impl FromRaw<String> for ffi::c_str {
  fn from(val: String) -> *const ffi::c_char { CString::new(val.as_str()).unwrap().as_ptr() }
}
