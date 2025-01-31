//! Most commonly used items from this crate bundled for convenient import.

pub use crate::{
    add_binvar,
    // proc macros
    add_ctsvar,
    add_intvar,
    add_var,
    attr,
    c,
    callback::{Callback, Where},
    constants::Norm,
    expr::{AttachModel, Expr, GurobiSum},
    param,
    Constr,
    ConstrSense,
    // ----------
    Env,
    GenConstr,
    Model,
    ModelObject,
    ModelSense,
    QConstr,
    RelaxType,
    SOSType,
    Status,
    Var,
    // constants
    VarType,
    INFINITY,
    SOS,
};

pub use ModelSense::*;
pub use VarType::*;
