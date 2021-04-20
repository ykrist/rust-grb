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
    expr::{Expr, GurobiSum, AttachVarNames},
    param,
    Constr,
    ConstrSense,
    Env,
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

#[allow(deprecated)]
pub use crate::expr::AttachModel;

pub use ModelSense::*;
pub use VarType::*;
