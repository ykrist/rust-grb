//! This crate provides Rust bindings for Gurobi Optimizer.  It currently requires Gurobi 9.0 or higher.
//!
//! ## Installing
//!
//! * Before using this crate, you should install Gurobi and obtain a [license](http://www.gurobi.com/downloads/licenses/license-center).
//!
//! * Make sure that the environment variable `GUROBI_HOME` is set to the installation path of Gurobi
//!   (like `C:\gurobi911\win64` or `/opt/gurobi911/linux64`).  If you are using the Conda package
//!   from the Gurobi channel, the build script will fall back to `GUROBI_HOME=${CONDA_PREFIX}`, so you
//!   should not set `GUROBI_HOME`.
//!
//! ## Quick Start
//! The example below sets up and solves a Mixed Integer Program (MIP).  Additional examples covering the more specific aspects of this crate's API can
//! be found [here](https://github.com/ykrist/rust-grb/tree/master/examples).
//!
//! The documention for [`Model`] contains most of the details for defining, solving and querying models.
//! ```
//! use grb::prelude::*;
//!
//! let mut model = Model::new("model1")?;
//!
//! // add decision variables with no bounds
//! let x1 = add_ctsvar!(model, name: "x1", bounds: ..)?;
//! let x2 = add_intvar!(model, name: "x2", bounds: ..)?;
//!
//! // add linear constraints
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
//! model.write("model.lp")?;
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
// TODO: fix the doc links to reference the 9.5 manual
use grb_sys2 as ffi;

/// Returns the version number of Gurobi
pub fn version() -> (i32, i32, i32) {
    let (mut major, mut minor, mut technical) = (-1, -1, -1);
    unsafe { ffi::GRBversion(&mut major, &mut minor, &mut technical) };
    (major, minor, technical)
}

/// Convenience wrapper around [`Model::add_var`]; adds a new variable to a `Model` object.  The macro keyword arguments are
/// optional.
///
/// # Syntax
/// The syntax of macro is two positional arguments followed by any number of named arguments:
/// ```text
/// add_var!(MODEL, VAR_TYPE, NAMED_ARG1: VAL1, NAMED_ARG2: VAL2, ...)
/// ```
/// `MODEL` should be an instance of a `Model`.
///
/// `VAR_TYPE` should be the variable type - a variant of [`VarType`].
///
/// The named arguments are described below.
///
/// | Name     | Type                                                      | `Model::add_var` argument |
/// | -------- | -------------------------------------------------------   | --------------------------- |
/// | `name`   | Anything that implements `AsRef<str>` (&str, String, etc) | `name`                      |
/// | `obj`    | Anything that can be cast to a `f64`                      | `obj`                       |
/// | `bounds` | A range expression, see below                             | `ub` & `lb`                 |
///
/// The `bounds` argument takes a value of the form `LB..UB` where `LB` and `UB` are the upper and lower bounds of the variable.
///  `LB` and `UB` can be   left off as well, so `..UB` (short for `-INFINITY..UB`), `LB..` (short for `LB..INFINITY`) and `..`
/// are also valid values.
///
///
///
/// [`Model::add_var`]: struct.Model.html#method.add_var
/// [`VarType`]: enum.VarType.html
/// ```
/// use grb::prelude::*;
/// let mut model = Model::new("Model").unwrap();
/// add_var!(model, Continuous, name: "name", obj: 0.0, bounds: -10..10)?;
/// add_var!(model, Integer, bounds: 0..)?;
/// add_var!(model, Continuous, name: &format!("X[{}]", 42))?;
/// # Ok::<(), grb::Error>(())
/// ```
///
#[doc(inline)]
pub use grb_macro::add_var;

/// Equivalent to calling [`add_var!`]`(model, Continuous, ...)`
#[doc(inline)]
pub use grb_macro::add_ctsvar;

/// Equivalent to calling [`add_var!`]`(model, Binary, ...)`
#[doc(inline)]
pub use grb_macro::add_binvar;

/// Equivalent to calling [`add_var!`]`(model, Integer, ...)`
#[doc(inline)]
pub use grb_macro::add_intvar;

/// A proc-macro for creating constraint objects.
///
/// # Syntax
/// ## Inequality constraints
/// To create an `IneqExpr` object for a linear or quadratic constraint, the syntax is
/// ```text
/// c!( LHS CMP RHS )
/// ```
/// `LHS` and `RHS` should be valid algebraic expressions involving `Var` objects and numeric constants.
/// For example, if `x`, `y` and `z` are `Var` objects and `vars` is an `Vec<Var>` objects, these are valid:
/// ```
/// # use grb::prelude::*;
/// # fn f(x: Var, y: Var, z: Var, vars: Vec<Var>){
///   c!(vars.iter().grb_sum() == x );
///   c!( x + 1/2 == 1.4*y - 2*z );
///   c!( 2*x >= z*y );
///   c!( 2*x >= 7*(z*y) ); // note the brackets on the non-linear term when a coefficient is present
/// # }
/// ```
/// but the following are not:
/// ```compile_fail
/// # use grb::*;
/// # fn f(x: Var, y: Var, z: Var){
///   c!(vars.iter().sum() == x ); // cannot infer type on sum() call
///   c!( 2*x >= z >= y ); // chained comparison
///   c!( 2*x >= 7*z*y ); // no brackets around var*var when a coefficient is present
/// # }
/// ```
/// The macro expands `c!( LHS == RHS )` to:
/// ```
/// # let LHS = 0;
/// # let RHS = 0;
/// grb::constr::IneqExpr {
///   lhs: grb::Expr::from(LHS),
///   sense: grb::ConstrSense::Equal,
///   rhs: grb::Expr::from(RHS),
/// };
/// ```
///
/// ## Range constraints
/// To create a `RangeExpr` object for a range constraint, use the syntax
/// ```text
/// c!( EXPR in LB..UB )
/// c!( EXPR in LB.. )
/// c!( EXPR in ..UB )
/// c!( EXPR in .. )
/// ```
/// where `EXPR` is a valid expression, like `LHS` and `RHS` above.  Additionally, `EXPR` must be linear,
/// although this is not checked at compile-time.
///
/// `LB` and `UB` can be any expression that evaluates to type that can be cast to a `f64` using
/// the `as` operator. For example, the following are valid (variables have the meaning as above):
/// ```
/// # use grb::prelude::*;
/// # fn f(x: Var, y: Var, z: Var, vars: Vec<Var>){
///   c!( x - y + 2*z in 0..200 );
///   c!( x - y + 2*z in 1.. );
///   c!( x - y in (1.0/3.0)..(1<<4));
/// # }
/// ```
///
#[doc(inline)]
pub use grb_macro::c;


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
