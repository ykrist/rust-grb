//! This module contains the structs passed to the [`Model::add_constr(s)`](crate::Model::add_constr) and [`Model::add_range(s)`](crate::Model::add_constr) methods.
//!
//! The structs themselves are usually constructed using the [`c!(...)`](crate::c) macro.
use crate::prelude::*;
use crate::Result;
use crate::expr::{LinExpr, QuadExpr};

/// A inequality constraint (linear or quadratic).  Creating this object does not automatically add the constraint to a model.
/// Instead, it should be passed to [`Model::add_constr`](crate::Model::add_constr) or [`Model::add_constrs`](crate::Model::add_constrs).
///
/// Usually created with an invocation of `c!(...)`.
#[derive(Clone)]
pub struct IneqExpr {
  pub lhs : Expr,
  pub sense: ConstrSense,
  pub rhs : Expr,
}

impl IneqExpr {
  pub(crate) fn into_normalised_linear(self) -> Result<(LinExpr, ConstrSense, f64)> {
    let IneqExpr {lhs, rhs, sense} = self;
    let mut lhs : LinExpr = (lhs - rhs).into_linexpr()?;
    let rhs = -lhs.set_offset(0.0);
    Ok((lhs, sense, rhs))
  }

  pub(crate) fn into_normalised_quad(self) -> (QuadExpr, ConstrSense, f64) {
    let IneqExpr {lhs, rhs, sense} = self;
    let mut lhs = (lhs - rhs).into_quadexpr();
    let rhs = -lhs.set_offset(0.0);
    (lhs, sense, rhs)
  }
}

/// A linear range constraint expression.  Creating this object does not automatically add the constraint to a model.
/// Instead, it should be passed to [`Model::add_range`](crate::Model::add_range) or [`Model::add_ranges`](crate::Model::add_ranges).
///
/// Usually created with an invocation of `c!(...)`.
/// Note that `expr` must be linear.
#[derive(Clone)]
pub struct RangeExpr {
  pub expr: Expr,
  pub ub : f64,
  pub lb : f64,
}

impl RangeExpr {
  pub(crate) fn into_normalised(self) -> Result<(LinExpr, f64, f64)> {
    let RangeExpr{expr, mut ub,  mut lb} = self;
    let mut expr = expr.into_linexpr()?;
    let offset = expr.set_offset(0.0);
    ub -= offset;
    lb -= offset;
    Ok((expr, lb, ub))
  }
}
