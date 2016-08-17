//! A crate which provides low-level Rust API of Gurobi Optimizer.
//!
//! This crate provides wrappers of the Gurobi solver which supports some
//! types of mathematical programming problems (e.g. Linear programming; LP,
//! Mixed Integer Linear Programming; MILP, and so on).
//!
//! ## Installation
//! Before using this crate, you should install Gurobi and obtain a license.
//! The instruction can be found
//! [here](http://www.gurobi.com/downloads/licenses/license-center).
//!
//! ## Examples
//! Work in progress...

extern crate gurobi_sys as ffi;

pub mod error;
pub mod env;
pub mod model;
pub mod expr;
mod util;
mod types;

// re-exports
pub use error::{Error, Result};

pub use env::{param, Env};

pub use model::{attr, Model};
pub use model::VarType::*;
pub use model::ConstrSense::*;
pub use model::ModelSense::*;
pub use model::SOSType::*;

pub use expr::{LinExpr, QuadExpr, LinTerm, QuadTerm};

// vim: set foldmethod=syntax :
