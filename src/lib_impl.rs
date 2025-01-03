#[path = "ffi.rs"]
pub(crate) mod ffi;

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
#[path = "attribute.rs"]
pub mod attribute;
#[path = "callback.rs"]
pub mod callback;
#[path = "constr.rs"]
pub mod constr;
#[path = "expr.rs"]
pub mod expr;
#[path = "parameter.rs"]
pub mod parameter;
#[path = "prelude.rs"]
pub mod prelude;

// Public re-exports
#[doc(no_inline)]
pub use attribute::attr;
pub use expr::Expr;
#[doc(no_inline)]
pub use parameter::param;

// private modules and their re-exports
#[path = "constants.rs"]
pub(crate) mod constants;
pub use constants::{
    ConstrSense, GenConstrType, ModelSense, RelaxType, SOSType, Status, VarType,
    GRB_INFINITY as INFINITY,
};

#[path = "env.rs"]
mod env;
pub use env::{EmptyEnv, Env};

#[path = "error.rs"]
mod error;
pub use error::{Error, Result};

#[path = "model.rs"]
mod model;
pub use model::{AsyncHandle, AsyncModel, Model};

#[path = "model_object.rs"]
pub(crate) mod model_object;
pub use model_object::{Constr, GenConstr, ModelObject, QConstr, Var, SOS};

#[path = "util.rs"]
pub(crate) mod util;
