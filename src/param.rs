// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

/// Defines the name of parameters
pub mod exports {
  pub use ffi::{IntParam, DoubleParam, StringParam};

  // re-exports
  pub use self::IntParam::*;
  pub use self::DoubleParam::*;
  pub use self::StringParam::*;
}
use self::exports::*;

use ffi;
use std::ffi::CString;
use util;


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

  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_int) -> ffi::c_int {
    ffi::GRBgetintparam(env, paramname, value)
  }

  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: ffi::c_int) -> ffi::c_int {
    ffi::GRBsetintparam(env, paramname, value)
  }
}

impl Param for DoubleParam {
  type Out = f64;
  type Buf = ffi::c_double;
  type RawFrom = *mut ffi::c_double;
  type RawTo = ffi::c_double;

  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_double) -> ffi::c_int {
    ffi::GRBgetdblparam(env, paramname, value)
  }

  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: ffi::c_double) -> ffi::c_int {
    ffi::GRBsetdblparam(env, paramname, value)
  }
}

impl Param for StringParam {
  type Out = String;
  type Buf = Vec<ffi::c_char>;
  type RawFrom = *mut ffi::c_char;
  type RawTo = *const ffi::c_char;

  unsafe fn get_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *mut ffi::c_char) -> ffi::c_int {
    ffi::GRBgetstrparam(env, paramname, value)
  }

  unsafe fn set_param(env: *mut ffi::GRBenv, paramname: ffi::c_str, value: *const ffi::c_char) -> ffi::c_int {
    ffi::GRBsetstrparam(env, paramname, value)
  }
}
