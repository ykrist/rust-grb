//! This crate provides Rust bindings for Gurobi Optimizer.  It currently requires Gurobi 9.5
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
#![warn(rustdoc::missing_crate_level_docs)]
// TODO: fix the doc links to reference the 9.5 manual

// The whole lib is wrapped in this cfg-if block, so in the likely scenario a user forgets to set a feature flag,
// they only see a single, relevant error.
cfg_if::cfg_if! {
    if #[cfg(not(any(feature = "gurobi12", feature = "gurobi11", feature = "gurobi10")))] {
        compile_error!(
            concat!(
                "The grb crate requires one of the following feature flags to be set:\n",
                "- gurobi12\n",
                "- gurobi11\n",
                "- gurobi10\n",
                "The flag should match the major version of Gurobi, for example (in Cargo.toml):\n\n",
                "grb = {..., features = ['gurobi12']}\n\n",
                "for Gurobi 12.X.\n\n",
                "If multiple feature flags are set, the highest version one is used, i.e. ",
                "setting gurobi12 and gurobi10 is equivalent to only setting gurobi12."
            )
        );
    } else {
        mod lib_impl;
        pub use lib_impl::*;
    }
}
