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
}

impl From<std::ffi::NulError> for Error {
  fn from(err: std::ffi::NulError) -> Error { Error::NulError(err) }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match *self {
      Error::FromAPI(ref message, code) => write!(f, "Error from API: {} ({})", message, code),
      Error::InconsitentDims => write!(f, "Inconsistent argument dimensions"),
      Error::NulError(ref err) => write!(f, "NulError: {}", err),
    }
  }
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    match *self {
      Error::FromAPI(..) => "error from C API",
      Error::NulError(ref err) => err.description(),
      Error::InconsitentDims => "Inconsistent argument dimensions",
    }
  }
}


/// A specialized
/// [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
/// type for operations in Gurobi Rust API
pub type Result<T> = std::result::Result<T, Error>;
