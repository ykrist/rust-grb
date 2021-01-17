// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

/// Defines the name of parameters
use gurobi_sys as ffi;
use std::ffi::CString;
use crate::util;

pub use ffi::{IntParam, DoubleParam, StringParam};
pub use ffi::IntParam::*;
pub use ffi::DoubleParam::*;
pub use ffi::StringParam::*;

// TODO simplify associated types
pub trait Param: Sized + Into<CString> {
  type Out;
  type Buf: util::Init + util::Into<Self::Out> + util::AsRawPtr<Self::RawFrom>;
  type RawFrom;
  type RawTo: util::FromRaw<Self::Out>;

  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: Self::RawFrom) -> ffi::c_int;

  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: Self::RawTo) -> ffi::c_int;
}



impl Param for IntParam {
  type Out = i32;
  type Buf = ffi::c_int;
  type RawFrom = *mut ffi::c_int;
  type RawTo = ffi::c_int;

  #[inline]
  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_int) -> ffi::c_int {
    ffi::GRBgetintparam(env, paramname, value)
  }

  #[inline]
  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: ffi::c_int) -> ffi::c_int {
    ffi::GRBsetintparam(env, paramname, value)
  }
}

impl Param for DoubleParam {
  type Out = f64;
  type Buf = ffi::c_double;
  type RawFrom = *mut ffi::c_double;
  type RawTo = ffi::c_double;

  #[inline]
  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_double) -> ffi::c_int {
    ffi::GRBgetdblparam(env, paramname, value)
  }

  #[inline]
  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: ffi::c_double) -> ffi::c_int {
    ffi::GRBsetdblparam(env, paramname, value)
  }
}

impl Param for StringParam {
  type Out = String;
  type Buf = Vec<ffi::c_char>;
  type RawFrom = *mut ffi::c_char;
  type RawTo = *const ffi::c_char;

  #[inline]
  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_char) -> ffi::c_int {
    ffi::GRBgetstrparam(env, paramname, value)
  }

  #[inline]
  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *const ffi::c_char) -> ffi::c_int {
    ffi::GRBsetstrparam(env, paramname, value)
  }
}
