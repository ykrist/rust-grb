// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

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
//!
//! ```
//! extern crate gurobi;
//! use gurobi::*;
//!
//! fn main() {
//!   let env = Env::new("logfile.log").unwrap();
//!
//!   // create an empty model which associated with `env`:
//!   let mut model = env.new_model("model1").unwrap();
//!
//!   // add decision variables.
//!   let x = model.add_var("x", Binary).unwrap();
//!   let y = model.add_var("y", Continuous(-10.0, 10.0)).unwrap();
//!   let z = model.add_var("z", Continuous(-INFINITY, INFINITY)).unwrap();
//!
//!   // integrate all the variables into the model.
//!   model.update().unwrap();
//!
//!   // add a linear constraint
//!   model.add_constr("c0", &x - &y + 2.0 * &z, Equal, 0.0).unwrap();
//!   // ...
//!
//!   model.set_objective(&x, Maximize).unwrap();
//!
//!   // optimize the model.
//!   model.optimize().unwrap();
//!   assert_eq!(model.status().unwrap(), Status::Optimal);
//!
//!   assert_eq!(model.get(attr::ObjVal).unwrap() , 0.0);
//!
//!   let val = model.get_values(attr::X, &[x, y, z]).unwrap();
//!   assert_eq!(val, [0.0, -10.0, -5.0]);
//! }
//! ```

extern crate gurobi_sys as ffi;
extern crate itertools;

mod env;
mod model;
mod error;
mod util;

// re-exports
pub use env::{param, Env};
pub use model::{attr, Model, Var, Constr, QConstr, SOS, LinExpr, QuadExpr};
pub use model::{Proxy, Status, RelaxType};
pub use model::VarType::*;
pub use model::ConstrSense::*;
pub use model::ModelSense::*;
pub use model::SOSType::*;

pub use error::{Error, Result};

// constants
pub const INFINITY: f64 = 1e100;
pub const UNDEFINED: f64 = 1e101;

// vim: set foldmethod=syntax :
