//! This module contains the structs passed to the [`Model::add_constr(s)`](crate::Model::add_constr) and [`Model::add_range(s)`](crate::Model::add_constr) methods.
//!
//! The structs themselves are usually constructed using the [`c!(...)`](crate::c) macro.
use crate::expr::{LinExpr, QuadExpr};
use crate::prelude::*;
use crate::Result;
use std::collections::HashMap;
use std::hash::BuildHasher;

/// A inequality constraint (linear or quadratic).  Creating this object does not automatically add the constraint to a model.
/// Instead, it should be passed to [`Model::add_constr`](crate::Model::add_constr) or [`Model::add_constrs`](crate::Model::add_constrs).
///
/// Usually created with an invocation of [`c!`]`(...)`.
#[derive(Debug, Clone)]
pub struct IneqExpr {
    /// Left-hand side
    pub lhs: Expr,
    /// Direction of the inequality, or if it the constraint is an equality
    pub sense: ConstrSense,
    /// Right-hand side
    pub rhs: Expr,
}

impl IneqExpr {
    pub(crate) fn into_normalised_linear(self) -> Result<(LinExpr, ConstrSense, f64)> {
        let IneqExpr { lhs, rhs, sense } = self;
        let mut lhs: LinExpr = (lhs - rhs).into_linexpr()?;
        let rhs = -lhs.set_offset(0.0);
        Ok((lhs, sense, rhs))
    }

    pub(crate) fn into_normalised_quad(self) -> (QuadExpr, ConstrSense, f64) {
        let IneqExpr { lhs, rhs, sense } = self;
        let mut lhs = (lhs - rhs).into_quadexpr();
        let rhs = -lhs.set_offset(0.0);
        (lhs, sense, rhs)
    }

    /// Evaluate the LHS and RHS of the constraint, given an assignment of variable values.
    /// Returns a tuple `(lhs, rhs)`.
    ///
    /// # Panics
    /// This function will panic if a variable in the expression is missing from the `var_values` map.
    pub fn evaluate<V: Copy + Into<f64>, S: BuildHasher>(&self, var_values: &HashMap<Var, V, S>) -> (f64, f64) {
        (self.lhs.evaluate(var_values), self.rhs.evaluate(var_values))
    }
}


/// A linear range constraint expression.  Creating this object does not automatically add the constraint to a model.
/// Instead, it should be passed to [`Model::add_range`](crate::Model::add_range) or [`Model::add_ranges`](crate::Model::add_ranges).
///
/// Usually created with an invocation of `c!(...)`.
/// Note that `expr` must be linear.
#[derive(Debug, Clone)]
pub struct RangeExpr {
    /// The linear expression of variables to constrain
    pub expr: Expr,
    /// The maximum value of the expression
    pub ub: f64,
    /// The minimum value of the expression
    pub lb: f64,
}

impl RangeExpr {
    pub(crate) fn into_normalised(self) -> Result<(LinExpr, f64, f64)> {
        let RangeExpr {
            expr,
            mut ub,
            mut lb,
        } = self;
        let mut expr = expr.into_linexpr()?;
        let offset = expr.set_offset(0.0);
        ub -= offset;
        lb -= offset;
        Ok((expr, lb, ub))
    }

    /// Evaluate the LHS of the range constraint, given an assignment of variable values.
    ///
    /// # Panics
    /// This function will panic if a variable in the expression is missing from the `var_values` map.
    pub fn evaluate<V: Copy + Into<f64>, S: BuildHasher>(&self, var_values: &HashMap<Var, V, S>) -> f64 {
        self.expr.evaluate(var_values)
    }
}

// TODO: support for general PWL constraints