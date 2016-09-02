// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

//! This crate provides primitive Rust APIs for Gurobi Optimizer.
//!
//! It supports some types of mathematical programming problems (e.g. Linear programming; LP,
//! Mixed Integer Linear Programming; MILP, and so on).
//!
//! ## Notices
//!
//! * Before using this crate, you should install Gurobi and obtain a license.
//!   The instruction can be found
//!   [here](http://www.gurobi.com/downloads/licenses/license-center).
//!
//! * Make sure that the environment variable `GUROBI_HOME` is set to the installation path of Gurobi
//!   (like `C:\gurobi652\win64`, `/opt/gurobi652/linux64`).
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
//!   let x1 = model.add_var("x1", Continuous(-INFINITY, INFINITY)).unwrap();
//!   let x2 = model.add_var("x2", Integer(-INFINITY, INFINITY)).unwrap();
//!
//!   // integrate all of the variables into the model.
//!   model.update().unwrap();
//!
//!   // add a linear constraint
//!   model.add_constr("c0", &x1 + 2.0 * &x2, Greater, -14.0).unwrap();
//!   model.add_constr("c1", -4.0 * &x1 - 1.0 * &x2, Less, -33.0).unwrap();
//!   model.add_constr("c2", 2.0 * &x1 + &x2, Less, 20.0).unwrap();
//!
//!   // integrate all of the constraints into the model.
//!   model.update().unwrap();
//!
//!   // set the expression of objective function.
//!   model.set_objective(8.0 * &x1 + &x2, Minimize).unwrap();
//!
//!   assert_eq!(model.get(attr::IsMIP).unwrap(), 1, "Model is not a MIP.");
//!
//!   // write constructed model to the file.
//!   model.write("logfile.lp").unwrap();
//!
//!   // optimize the model.
//!   model.optimize().unwrap();
//!   assert_eq!(model.status().unwrap(), Status::Optimal);
//!
//!   assert_eq!(model.get(attr::ObjVal).unwrap() , 59.0);
//!
//!   let val = model.get_values(attr::X, &[x1, x2]).unwrap();
//!   assert_eq!(val, [6.5, 7.0]);
//! }
//! ```

extern crate gurobi_sys as ffi;
extern crate itertools;

mod env;
mod error;
mod model;
mod util;

#[path = "param.rs"]
mod parameter;

#[path = "attr.rs"]
mod attribute;

// re-exports
pub use error::{Error, Result};

pub use env::Env;

pub use model::{Model, Var, Constr, QConstr, SOS, Proxy};
pub use model::{VarType, ConstrSense, ModelSense, SOSType, Status, RelaxType};
pub use model::callback::{Callback, Where};
pub use model::VarType::*;
pub use model::ConstrSense::*;
pub use model::ModelSense::*;
pub use model::SOSType::*;
pub use model::RelaxType::*;
pub use model::expr::{LinExpr, QuadExpr};

pub use attribute::exports as attr;
pub use parameter::exports as param;


/// Large number used in C API
pub const INFINITY: f64 = 1e100;


/// Returns the version number of Gurobi
pub fn version() -> (i32, i32, i32) {
  let (mut major, mut minor, mut technical) = (0, 0, 0);
  unsafe { ffi::GRBversion(&mut major, &mut minor, &mut technical) };
  (major, minor, technical)
}
