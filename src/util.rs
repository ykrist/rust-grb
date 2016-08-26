// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use super::ffi;
use std::ffi::{CStr, CString};
use std::ptr::null;

pub unsafe fn from_c_str(s: *const ffi::c_char) -> String { CStr::from_ptr(s).to_string_lossy().into_owned() }


///
pub trait From<T> {
  fn from(T) -> Self;
}

impl From<i32> for ffi::c_int {
  fn from(val: i32) -> ffi::c_int { val }
}

impl From<i8> for ffi::c_char {
  fn from(val: i8) -> ffi::c_char { val }
}

impl From<f64> for ffi::c_double {
  fn from(val: f64) -> ffi::c_double { val }
}

impl From<String> for ffi::c_str {
  fn from(val: String) -> ffi::c_str { CString::new(val.as_str()).unwrap().as_ptr() }
}


/// make an empty instance.
pub trait Init {
  fn init() -> Self;
}

impl Init for ffi::c_char {
  fn init() -> ffi::c_char { 0 }
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

impl Init for ffi::c_str {
  fn init() -> ffi::c_str { null() }
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
  fn into(self) -> String { unsafe { from_c_str(self.as_ptr()) } }
}

impl Into<i8> for ffi::c_char {
  fn into(self) -> i8 { self }
}

impl Into<String> for ffi::c_str {
  fn into(self) -> String { unsafe { from_c_str(self).to_owned() } }
}


/// convert to Raw C Pointer.
pub trait AsRawPtr<T> {
  fn as_rawptr(&mut self) -> T;
}

impl AsRawPtr<*mut ffi::c_int> for i32 {
  fn as_rawptr(&mut self) -> *mut ffi::c_int { self }
}

impl AsRawPtr<*mut ffi::c_char> for i8 {
  fn as_rawptr(&mut self) -> *mut ffi::c_char { self }
}

impl AsRawPtr<*mut ffi::c_double> for f64 {
  fn as_rawptr(&mut self) -> *mut ffi::c_double { self }
}

impl AsRawPtr<*mut ffi::c_str> for ffi::c_str {
  fn as_rawptr(&mut self) -> *mut ffi::c_str { self }
}

impl AsRawPtr<*mut ffi::c_char> for Vec<ffi::c_char> {
  fn as_rawptr(&mut self) -> *mut ffi::c_char { self.as_mut_ptr() }
}


///
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


#[test]
fn conversion_must_success() {
  let s1 = "mip1.log";
  let s2 = unsafe { from_c_str(CString::new(s1).unwrap().as_ptr()) };
  assert!(s1 == s2);
}
