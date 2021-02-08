use std::ffi::{CStr, CString};
use grb_sys as ffi;
use std::fmt;
/// Copy a raw C-string into a String
///
/// To quote the Gurobi docs:
/// Note that all interface routines that return string-valued attributes are returning pointers
/// into internal Gurobi data structures. The user should copy the contents of the pointer to a
/// different data structure before the next call to a Gurobi library routine. The user should
/// also be careful to never modify the data pointed to by the returned character pointer.
pub(crate) unsafe fn copy_c_str(s: ffi::c_str) -> String {
  CStr::from_ptr(s).to_string_lossy().into_owned() // to_string_lossy().into_owned() ALWAYS clones
}


#[test]
fn conversion_must_succeed() {
  use std::ffi::CString;
  let s1 = "mip1.log";
  let cs = CString::new(s1).unwrap();
  let s2 = unsafe { copy_c_str(cs.as_ptr()) };
  assert!(s1 == s2);
}


pub(crate) trait AsPtr {
  type Raw;
  /// Return the underlying Gurobi pointer for [`Model`] and [`Env`] objects
  ///
  /// # Safety
  /// One of the following conditions must hold
  /// - self is mutable
  /// - the resulting pointer is passed only to Gurobi library routines
  unsafe fn as_mut_ptr(&self) -> *mut Self::Raw;

  /// Return the underling Gurobi pointer
  fn as_ptr(&self) -> *const Self::Raw {
    (unsafe { self.as_mut_ptr() }) as *const Self::Raw
  }
}

/// Marker trait needed for unsafe in GurobiName blanket impl
pub trait GurobiNameMarker {}

pub trait GurobiName {
  fn name(&self) -> CString;
}

impl<T> GurobiName for T
  where
    T: GurobiNameMarker + fmt::Debug
{
  fn name(&self) -> CString {
    let s = format!("{:?}", &self);
    // We know the debug repr of these enum variants won't contain nul bytes or non-ascii chars
    unsafe { CString::from_vec_unchecked(s.into_bytes()) }
  }
}