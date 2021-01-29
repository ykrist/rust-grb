use crate::{Expr, Result, ConstrSense};
use crate::expr::{LinExpr, QuadExpr};

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
