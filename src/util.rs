use std::ffi::CStr;
use gurobi_sys as ffi;

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
