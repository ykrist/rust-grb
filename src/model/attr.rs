// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

/// Defines the name of attributes
pub mod exports {
  pub use ffi::{IntAttr, DoubleAttr, CharAttr, StringAttr};
  pub use self::IntAttr::*;
  pub use self::DoubleAttr::*;
  pub use self::CharAttr::*;
  pub use self::StringAttr::*;
}
use self::exports::*;

use ffi;
use std::ffi::CString;
use util;
use error::Result;

/// provides function to query/set the value of scalar attribute.
pub trait AttrBase: Into<CString> {
  type Out;
  type Buf: util::Init + util::Into<Self::Out> + util::AsRawPtr<Self::RawGet>;
  type RawGet;
  type RawSet: util::From<Self::Out>;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawGet) -> ffi::c_int;

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int;
}

impl AttrBase for IntAttr {
  type Out = i32;
  type Buf = i32;
  type RawGet = *mut ffi::c_int;
  type RawSet = ffi::c_int;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_int) -> ffi::c_int {
    ffi::GRBgetintattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetintattr(model, attrname, value)
  }
}

impl AttrBase for DoubleAttr {
  type Out = f64;
  type Buf = f64;
  type RawGet = *mut ffi::c_double;
  type RawSet = ffi::c_double;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_double) -> ffi::c_int {
    ffi::GRBgetdblattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetdblattr(model, attrname, value)
  }
}

impl AttrBase for StringAttr {
  type Out = String;
  type Buf = ffi::c_str;
  type RawGet = *mut ffi::c_str;
  type RawSet = ffi::c_str;

  unsafe fn get_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: *mut ffi::c_str) -> ffi::c_int {
    ffi::GRBgetstrattr(model, attrname, value)
  }

  unsafe fn set_attr(model: *mut ffi::GRBmodel, attrname: ffi::c_str, value: Self::RawSet) -> ffi::c_int {
    ffi::GRBsetstrattr(model, attrname, value)
  }
}


pub trait AttrArrayBase: Into<CString> {
  type Out: Clone;
  type Buf: Clone + util::Init + util::Into<Self::Out> + util::AsRawPtr<Self::RawGet>;
  type RawGet;
  type RawSet: util::From<Self::Out>;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            values: Self::RawGet)
                            -> ffi::c_int;
  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            values: Self::RawSet)
                            -> ffi::c_int;

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut Self::Buf)
                         -> ffi::c_int;

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int;

  fn to_rawsets(values: &[Self::Out]) -> Result<Vec<Self::RawSet>> {
    Ok(values.iter().map(|v| util::From::from(v.clone())).collect())
  }
}


impl AttrArrayBase for IntAttr {
  type Out = i32;
  type Buf = i32;
  type RawGet = *mut ffi::c_int;
  type RawSet = ffi::c_int;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_int)
                            -> ffi::c_int {
    ffi::GRBgetintattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: ffi::c_int)
                            -> ffi::c_int {
    ffi::GRBsetintattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut ffi::c_int)
                         -> ffi::c_int {
    ffi::GRBgetintattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int {
    ffi::GRBsetintattrlist(model, attrname, len, ind, values)
  }
}

impl AttrArrayBase for DoubleAttr {
  type Out = f64;
  type Buf = f64;
  type RawGet = *mut ffi::c_double;
  type RawSet = ffi::c_double;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_double)
                            -> ffi::c_int {
    ffi::GRBgetdblattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: ffi::c_double)
                            -> ffi::c_int {
    ffi::GRBsetdblattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut ffi::c_double)
                         -> ffi::c_int {
    ffi::GRBgetdblattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int {
    ffi::GRBsetdblattrlist(model, attrname, len, ind, values)
  }
}

impl AttrArrayBase for CharAttr {
  type Out = i8;
  type Buf = i8;
  type RawGet = *mut ffi::c_char;
  type RawSet = ffi::c_char;

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_char)
                            -> ffi::c_int {
    ffi::GRBgetcharattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: ffi::c_char)
                            -> ffi::c_int {
    ffi::GRBsetcharattrelement(model, attrname, element, value)
  }

  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut ffi::c_char)
                         -> ffi::c_int {
    ffi::GRBgetcharattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const Self::RawSet)
                         -> ffi::c_int {
    ffi::GRBsetcharattrlist(model, attrname, len, ind, values)
  }
}

impl AttrArrayBase for StringAttr {
  type Out = String;
  type Buf = ffi::c_str;
  type RawGet = *mut ffi::c_str;
  type RawSet = ffi::c_str;

  fn to_rawsets(values: &[String]) -> Result<Vec<ffi::c_str>> {
    let mut buf = Vec::with_capacity(values.len());
    for value in values.into_iter() {
      let value = try!(CString::new(value.as_str()));
      buf.push(value.as_ptr())
    }
    Ok(buf)
  }

  unsafe fn get_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int,
                            value: *mut ffi::c_str)
                            -> ffi::c_int {
    ffi::GRBgetstrattrelement(model, attrname, element, value)
  }

  unsafe fn set_attrelement(model: *mut ffi::GRBmodel, attrname: ffi::c_str, element: ffi::c_int, value: ffi::c_str)
                            -> ffi::c_int {
    ffi::GRBsetstrattrelement(model, attrname, element, value)
  }


  unsafe fn get_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *mut ffi::c_str)
                         -> ffi::c_int {
    ffi::GRBgetstrattrlist(model, attrname, len, ind, values)
  }

  unsafe fn set_attrlist(model: *mut ffi::GRBmodel, attrname: ffi::c_str, len: ffi::c_int, ind: *const ffi::c_int,
                         values: *const ffi::c_str)
                         -> ffi::c_int {
    ffi::GRBsetstrattrlist(model, attrname, len, ind, values)
  }
}
