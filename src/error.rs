// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use ffi;
use std;

/// The error type for operations in Gurobi Rust API
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

impl From<std::ffi::NulError> for Error {
  fn from(err: std::ffi::NulError) -> Error { Error::NulError(err) }
}


/// A specialized
/// [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
/// type for operations in Gurobi Rust API
pub type Result<T> = std::result::Result<T, Error>;
