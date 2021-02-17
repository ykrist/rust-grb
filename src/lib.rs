//! This crate provides Rust bindings for Gurobi Optimizer.  It currently requires Gurobi 9.0 or higher.
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
//! use grb::prelude::*;
//!
//! let mut model = Model::new("model1")?;
//!
//! // add decision variables with no bounds
//! let x1 = add_ctsvar!(model, name: "x1", bounds: ..)?;
//! let x2 = add_intvar!(model, name: "x2", bounds: ..)?;
//!
//! // add a linear constraints
//! let c0 = model.add_constr("c0", c!(x1 + 2*x2 >= -14))?;
//! let c1 = model.add_constr("c1", c!(-4 * x1 - x2 <= -33))?;
//! let c2 = model.add_constr("c2", c!(2* x1 <= 20 - x2))?;
//!
//! // model is lazily updated by default
//! assert_eq!(model.get_obj_attr(attr::VarName, &x1).unwrap_err(), grb::Error::ModelObjectPending);
//! assert_eq!(model.get_attr(attr::IsMIP)?, 0);
//!
//! // set the objective function, which updates the model objects (variables and constraints).
//! // One could also call `model.update()`
//! model.set_objective(8*x1 + x2, Minimize)?;
//! assert_eq!(model.get_obj_attr(attr::VarName, &x1)?, "x1");
//! assert_eq!(model.get_attr(attr::IsMIP)?, 1);
//!
//! // write model to the file.
//! model.write("logfile.lp")?;
//!
//! // optimize the model
//! model.optimize()?;
//! assert_eq!(model.status()?, Status::Optimal);
//!
//! // Querying a model attribute
//! assert_eq!(model.get_attr(attr::ObjVal)? , 59.0);
//!
//! // Querying a model object attributes
//! assert_eq!(model.get_obj_attr(attr::Slack, &c0)?, -34.5);
//! let x1_name = model.get_obj_attr(attr::VarName, &x1)?;
//!
//! // Querying an attribute for multiple model objects
//! let val = model.get_obj_attr_batch(attr::X, vec![x1, x2])?;
//! assert_eq!(val, [6.5, 7.0]);
//!
//! // Querying variables by name
//! assert_eq!(model.get_var_by_name(&x1_name)?, Some(x1));
//!
//! # Ok::<(), grb::Error>(())
//! ```
//!
//! ## Errors
//! Due to the nature of C APIs, almost every Gurobi routine can return an error.  Unless otherwise stated,
//! if a method or function returns a [`Result`], the error will be [`Error::FromAPI`].
#![warn(missing_docs)]
#![warn(missing_crate_level_docs)]

/// Returns the version number of Gurobi
pub fn version() -> (i32, i32, i32) {
    let (mut major, mut minor, mut technical) = (-1, -1, -1);
    unsafe { grb_sys::GRBversion(&mut major, &mut minor, &mut technical) };
    (major, minor, technical)
}

// external re-exports
#[doc(inline)]
pub use grb_macro::*;

// public modules
pub mod attribute;
pub mod callback;
pub mod constr;
pub mod expr;
pub mod parameter;
pub mod prelude;

// Public re-exports
#[doc(no_inline)]
pub use attribute::attr;
pub use expr::Expr;
#[doc(no_inline)]
pub use parameter::param;

// private modules and their re-exports
mod constants;
pub use constants::{
    ConstrSense, ModelSense, RelaxType, SOSType, Status, VarType, GRB_INFINITY as INFINITY,
};

mod env;
pub use env::{EmptyEnv, Env};

mod error;
pub use error::{Error, Result};

mod model;
pub use model::{AsyncHandle, AsyncModel, Model};

mod model_object;
pub use model_object::{Constr, ModelObject, QConstr, Var, SOS};

mod util;
