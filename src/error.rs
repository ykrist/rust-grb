// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use gurobi_sys as ffi;

/// The error type for operations in Gurobi Rust API
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
  /// An error returned from Gurobi C API.  Contains the message and the error code.
  FromAPI(String, i32),
  /// Conversion to a C-style null-termined string failed.  Contains the underlying [`std::ffi::NulError`].
  NulError(std::ffi::NulError),
  /// Inconsistent argument dimensions
  InconsistentDims,
  /// Query/modifying a removed variable or constraint
  ModelObjectRemoved,
  /// Model object hasn't been updated yet.  A call to [`Model::update`](crate::Model::update) is needed.
  ModelObjectPending,
  /// Model object comes from a different model
  ModelObjectMismatch,
  /// A call to [`Model::update`](crate::Model::update) is required before this operation
  ModelUpdateNeeded,
  /// Modelling errors caused by the user, usually by providing quadratic expressions to methods that expect
  /// linear terms such as [`Model::add_constr`](crate::Model::add_constr).
  AlgebraicError(String),
}

impl From<std::ffi::NulError> for Error {
  fn from(err: std::ffi::NulError) -> Error { Error::NulError(err) }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Error::FromAPI(message, code) => write!(f, "Error from API: {} ({})", message, code),
      Error::InconsistentDims => write!(f, "Inconsistent argument dimensions"),
      Error::NulError(err) =>  f.write_fmt(format_args!("NulError: {}", err)),
      Error::ModelObjectRemoved => f.write_str("Variable or constraint has been removed from the model"),
      Error::ModelObjectPending => f.write_str("Variable or constraint is awaiting model update"),
      Error::ModelObjectMismatch => f.write_str("Variable or constraint is part of a different model"),
      Error::ModelUpdateNeeded => f.write_str("Variables or constraints have been added/removed.  Call model.update() first."),
      Error::AlgebraicError(s) => f.write_fmt(format_args!("Algebraic error: {}", s)),
    }
  }
}

impl std::error::Error for Error {}

/// A specialized
/// [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html)
/// type for operations in Gurobi Rust API
pub type Result<T> = std::result::Result<T, Error>;
