// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

//! This crate provides Rust bindings for Gurobi Optimizer.
//!
//! ## Installing
//!
//! * Before using this crate, you should install Gurobi and obtain a [license](http://www.gurobi.com/downloads/licenses/license-center).
//!
//! * Make sure that the environment variable `GUROBI_HOME` is set to the installation path of Gurobi
//!   (like `C:\gurobi652\win64`, `/opt/gurobi652/linux64`).  If using the Conda package from the Gurobi
//!   channel, set `GUROBI_HOME=${CONDA_PREFIX}`.
//!
//! * On Windows, the toolchain should be MSVC ABI (it also requires Visual Studio or
//!   Visual C++ Build Tools).
//!   If you want to use GNU ABI with MinGW-w64/MSYS2 toolchain, you should create the import
//!   library for Gurobi runtime DLL (e.g. `gurobi65.dll`) and put it into `GUROBI_HOME/lib`.
//!   Procedure of creating import library is as follows:
//!
//!   ```shell-session
//!   $ pacman -S mingw-w64-x86_64-tools-git
//!   $ gendef - $(cygpath $GUROBI_HOME)/bin/gurobi65.dll > gurobi65.def
//!   $ dlltool --dllname gurobi65.dll --def gurobi65.def --output-lib $(cygpath $GUROBI}HOME)/lib/libgurobi65.dll.a
//!   ```
//!
//! ## Examples
//!
//! ```
//! use gurobi::*;
//!
//! let env = Env::new("logfile.log").unwrap();
//!
//! // create an empty model which associated with `env`:
//! let mut model = Model::new("model1", &env).unwrap();
//!
//! // add decision variables with no bounds
//! let x1 = add_var!(model, Continuous, name="x1", bounds=..).unwrap();
//! let x2 = add_var!(model, Integer, name="x2", bounds=..).unwrap();
//!
//! // add a linear constraints
//! let c0 = model.add_constr("c0", x1 + 2*x2, Greater, -14).unwrap();
//! let c1 = model.add_constr("c1", -4 * x1 - x2, Less, -33).unwrap();
//! let c2 = model.add_constr("c2", 2* x1, Less, 20 - x2).unwrap();
//!
//! // set the objective function.
//! model.set_objective(8*x1 + x2, Minimize).unwrap();
//!
//! // model is lazily updated by default
//! assert_eq!(model.get_obj_attr(attr::VarName, &x1).unwrap_err(), Error::ModelObjectPending);
//! assert_eq!(model.get_attr(attr::IsMIP).unwrap(), 0);
//! model.update().unwrap();
//! assert_eq!(model.get_attr(attr::IsMIP).unwrap(), 1, "Model is not a MIP.");
//!
//! // write model to the file.
//! model.write("logfile.lp").unwrap();
//!
//! // optimize the model
//! model.optimize().unwrap();
//! assert_eq!(model.status().unwrap(), Status::Optimal);
//!
//! // Querying a model attribute
//! assert_eq!(model.get_attr(attr::ObjVal).unwrap() , 59.0);
//!
//! // Querying a model object attributes
//! assert_eq!(&model.get_obj_attr(attr::ConstrName, &c0).unwrap(), "c0");
//! let x1_name = model.get_obj_attr(attr::VarName, &x1).unwrap();
//!
//! // Querying an attribute for multiple model objects
//! let val = model.get_obj_attr_batch(attr::X, &[x1, x2]).unwrap();
//! assert_eq!(val, [6.5, 7.0]);
//!
//! // Querying variables by name
//! assert_eq!(model.get_var_by_name(&x1_name).unwrap(), Some(x1));
//! ```

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

pub mod env;
mod error;
mod model;
mod util;
mod expr;
mod callback;
mod constants;
mod model_object;
pub mod param;
pub mod attr;

// re-exports
pub use env::Env;
pub use expr::{Expr, LinExpr, QuadExpr, AttachModel, GurobiSum};
pub use error::{Error, Result};
pub use model::Model;
pub use model::{VarType, ConstrSense, ModelSense, SOSType, Status, RelaxType};
pub use model_object::*;
pub use callback::{Callback, Where};
pub use model::VarType::*;
pub use model::ConstrSense::*;
pub use model::ModelSense::*;
pub use model::SOSType::*;
pub use model::RelaxType::*;

/// Large number used in C API
pub use constants::GRB_INFINITY as INFINITY;


/// Returns the version number of Gurobi
pub fn version() -> (i32, i32, i32) {
  let (mut major, mut minor, mut technical) = (0, 0, 0);
  unsafe { gurobi_sys::GRBversion(&mut major, &mut minor, &mut technical) };
  (major, minor, technical)
}
