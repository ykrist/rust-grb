extern crate gurobi_sys as ffi;

use util;


/// make an empty instance.
pub trait Init {
  fn init() -> Self;
}

impl Init for ffi::c_int {
  fn init() -> i32 {
    0
  }
}

impl Init for ffi::c_double {
  fn init() -> ffi::c_double {
    0.0
  }
}

impl Init for Vec<ffi::c_char> {
  fn init() -> Vec<ffi::c_char> {
    Vec::with_capacity(4096)
  }
}


/// convert into different type.
pub trait Into<T> {
  fn into(self) -> T;
}

impl Into<i32> for ffi::c_int {
  fn into(self) -> i32 {
    self
  }
}

impl Into<f64> for ffi::c_double {
  fn into(self) -> f64 {
    self
  }
}

impl Into<String> for Vec<ffi::c_char> {
  fn into(self) -> String {
    unsafe { util::from_c_str(self.as_ptr()) }
  }
}
