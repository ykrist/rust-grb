#![allow(clippy::missing_safety_doc)]
use grb_sys as ffi;
use std::ffi::CString;

use std::result::Result as StdResult;

#[doc(inline)]
pub use ffi::{IntAttr, DoubleAttr, CharAttr, StringAttr};
pub use ffi::IntAttr::*;
pub use ffi::DoubleAttr::*;
pub use ffi::CharAttr::*;
pub use ffi::StringAttr::*;

use crate::util::copy_c_str;

// We don't have any &Model here, so the best we can do is give Gurobi error codes.
// Fortunately, the errors encountered either are from converting String to CString, null pointers
// and trying to access Char attributes on the model (there are none defined), all Char attributes
// are either Constr or Var attributes.  The Gurobi error codes therefore match up nicely.
use crate::constants::{ERROR_INVALID_ARGUMENT, ERROR_DATA_NOT_AVAILABLE, ERROR_UNKNOWN_ATTR};

fn check_error_code(code: ffi::c_int) -> StdResult<(), ffi::c_int> {
    if code == 0 { Ok(()) } else { Err(code) }
}

pub trait Attr: Into<CString> {
    type Value;

    unsafe fn get(self, model: *mut ffi::GRBmodel) -> StdResult<Self::Value, ffi::c_int>;
    unsafe fn get_element(self, model: *mut ffi::GRBmodel, index: i32) -> StdResult<Self::Value, ffi::c_int>;
    unsafe fn get_elements(self, model: *mut ffi::GRBmodel, indices: &[i32]) -> StdResult<Vec<Self::Value>, ffi::c_int>;
    unsafe fn set(self, model: *mut ffi::GRBmodel, val: Self::Value) -> StdResult<(), ffi::c_int>;
    unsafe fn set_element(self, model: *mut ffi::GRBmodel, index: i32, value: Self::Value) -> StdResult<(), ffi::c_int>;
    unsafe fn set_elements(self, model: *mut ffi::GRBmodel, indices: &[i32], values: &[Self::Value]) -> StdResult<(), ffi::c_int>;
}

macro_rules! impl_gurobiattr_attr {
    ($init:expr, $getattr:path, $setattr:path) => {
            unsafe fn get(self, model: *mut ffi::GRBmodel) -> StdResult<Self::Value, ffi::c_int> {
                let aname : CString = self.into();
                let mut val = $init;
                check_error_code($getattr(model, aname.as_ptr(), &mut val))?;
                Ok(val)
            }

            unsafe fn set(self, model: *mut ffi::GRBmodel, value: Self::Value) -> StdResult<(), ffi::c_int> {
                let aname : CString = self.into();
                check_error_code($setattr(model, aname.as_ptr(), value))
            }
    }
}

macro_rules! impl_gurobiattr_elem_attr {
    ($init:expr, $getattrelement:path, $setattrelement:path) => {
            unsafe fn get_element(self, model: *mut ffi::GRBmodel, index: i32) -> StdResult<Self::Value, ffi::c_int> {
                let aname : CString = self.into();
                let mut val = $init;
                check_error_code($getattrelement(model, aname.as_ptr(), index, &mut val))?;
                Ok(val)
            }

            unsafe fn set_element(self, model: *mut ffi::GRBmodel, index: i32, value: Self::Value) -> StdResult<(), ffi::c_int> {
                let aname : CString = self.into();
                check_error_code($setattrelement(model, aname.as_ptr(), index, value))
            }
    }
}

macro_rules! impl_gurobiattr_elems_attr {
    ($init:expr, $getattrlist:path, $setattrlist:path) => {
            unsafe fn get_elements(self, model: *mut ffi::GRBmodel, indices: &[i32]) -> StdResult<Vec<Self::Value>, ffi::c_int> {
                let aname : CString = self.into();
                let mut vals = vec![$init; indices.len()];
                check_error_code($getattrlist(model, aname.as_ptr(), indices.len() as i32, indices.as_ptr(), vals.as_mut_ptr()))?;
                Ok(vals)
            }

            unsafe fn set_elements(self, model: *mut ffi::GRBmodel, indices: &[i32], values: &[Self::Value]) -> StdResult<(), ffi::c_int> {
                debug_assert_eq!(indices.len(), values.len()); // caller should check
                let aname : CString = self.into();
                check_error_code($setattrlist(model, aname.as_ptr(), indices.len() as i32, indices.as_ptr(), values.as_ptr()))
            }
    }
}


impl Attr for IntAttr {
    type Value = i32;
    impl_gurobiattr_attr!(i32::MIN, ffi::GRBgetintattr, ffi::GRBsetintattr);
    impl_gurobiattr_elem_attr!(i32::MIN, ffi::GRBgetintattrelement, ffi::GRBsetintattrelement);
    impl_gurobiattr_elems_attr!(i32::MIN, ffi::GRBgetintattrlist, ffi::GRBsetintattrlist);
}

impl Attr for DoubleAttr {
    type Value = f64;
    impl_gurobiattr_attr!(f64::NAN, ffi::GRBgetdblattr, ffi::GRBsetdblattr);
    impl_gurobiattr_elem_attr!(f64::NAN, ffi::GRBgetdblattrelement, ffi::GRBsetdblattrelement);
    impl_gurobiattr_elems_attr!(f64::NAN, ffi::GRBgetdblattrlist, ffi::GRBsetdblattrlist);
}

impl Attr for CharAttr {
    type Value = ffi::c_char;

    unsafe fn get(self, _: *mut ffi::GRBmodel) -> StdResult<Self::Value, ffi::c_int> {
        Err(ERROR_UNKNOWN_ATTR)
    }

    unsafe fn set(self, _: *mut ffi::GRBmodel, _: Self::Value) -> StdResult<(), ffi::c_int> {
        Err(ERROR_UNKNOWN_ATTR)
    }
    impl_gurobiattr_elem_attr!(0, ffi::GRBgetcharattrelement, ffi::GRBsetcharattrelement);
    impl_gurobiattr_elems_attr!(0, ffi::GRBgetcharattrlist, ffi::GRBsetcharattrlist);
}

impl Attr for StringAttr {
    type Value = String;

    unsafe fn get(self, model: *mut ffi::GRBmodel) -> StdResult<Self::Value, ffi::c_int> {
        let aname: CString = self.into();
        let mut s: ffi::c_str = std::ptr::null();
        check_error_code(ffi::GRBgetstrattr(model, aname.as_ptr(), &mut s))?;
        if s.is_null() {
            Err(ERROR_DATA_NOT_AVAILABLE)
        } else {
            Ok(copy_c_str(s))
        }
    }
    unsafe fn get_element(self, model: *mut ffi::GRBmodel, index: i32) -> StdResult<Self::Value, ffi::c_int> {
        let aname: CString = self.into();
        let mut s: ffi::c_str = std::ptr::null();
        check_error_code(ffi::GRBgetstrattrelement(model, aname.as_ptr(), index, &mut s))?;
        if s.is_null() {
            Err(ERROR_DATA_NOT_AVAILABLE)
        } else {
            Ok(copy_c_str(s))
        }
    }

    unsafe fn get_elements(self, model: *mut ffi::GRBmodel, indices: &[i32]) -> StdResult<Vec<Self::Value>, ffi::c_int> {
        let aname: CString = self.into();
        let mut cstrings: Vec<ffi::c_str> = vec![std::ptr::null(); indices.len()];
        check_error_code(ffi::GRBgetstrattrlist(model, aname.as_ptr(), indices.len() as ffi::c_int,
                                                indices.as_ptr(), cstrings.as_mut_ptr()))?;

        cstrings.into_iter().map(|s| {
            if s.is_null() {
                Err(ERROR_DATA_NOT_AVAILABLE)
            } else {
                Ok(copy_c_str(s))
            }
        }).collect()
    }
    unsafe fn set(self, model: *mut ffi::GRBmodel, val: Self::Value) -> StdResult<(), ffi::c_int> {
        let aname: CString = self.into();
        let s = CString::new(val).map_err(|_| ERROR_INVALID_ARGUMENT)?;
        check_error_code(
            ffi::GRBsetstrattr(model, aname.as_ptr(), s.as_ptr())
        )
    }
    unsafe fn set_element(self, model: *mut ffi::GRBmodel, index: i32, value: Self::Value) -> StdResult<(), ffi::c_int> {
        let aname: CString = self.into();
        let s = CString::new(value).map_err(|_| ERROR_INVALID_ARGUMENT)?;
        check_error_code(
            ffi::GRBsetstrattrelement(model, aname.as_ptr(), index, s.as_ptr())
        )
    }
    unsafe fn set_elements(self, model: *mut ffi::GRBmodel, indices: &[i32], values: &[Self::Value]) -> StdResult<(), ffi::c_int> {
        let aname: CString = self.into();
        let cstrings: StdResult<Vec<_>, _> = values.iter().cloned()
            .map(|val| CString::new(val).map_err(|_| ERROR_INVALID_ARGUMENT))
            .collect();
        let strarray : Vec<ffi::c_str> = cstrings?.iter().map(|s| s.as_ptr()).collect();
        check_error_code(
            ffi::GRBsetstrattrlist(model,
                                   aname.as_ptr(),
                                   indices.len() as ffi::c_int,
                                   indices.as_ptr(),
                                   strarray.as_ptr())
        )
    }
}
