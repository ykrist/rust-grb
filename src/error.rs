extern crate gurobi_sys as ffi;
use std;

/// The error type for operations in Gurobi Rust API.
#[derive(Debug)]
pub enum Error {
  /// An exception returned from Gurobi C API
  FromAPI(String, ffi::c_int),

  /// See https://doc.rust-lang.org/std/ffi/struct.NulError.html
  NulError(std::ffi::NulError),

  /// Inconsistent argument dimensions
  InconsitentDims,

  /// String conversion error
  StringConversion
}

/// A specialized
/// [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
/// type for operations in Gurobi Rust API.
pub type Result<T> = std::result::Result<T, Error>;
