use crate::{Expr, Result, ConstrSense};
use crate::expr::{LinExpr, QuadExpr};

pub struct ConstrExpr {
  pub lhs : Expr,
  pub sense: ConstrSense,
  pub rhs : Expr,
}

impl ConstrExpr {
  pub(crate) fn into_normalised_linear(self) -> Result<(LinExpr, ConstrSense, f64)> {
    let ConstrExpr{lhs, rhs, sense} = self;
    let lhs : LinExpr = (lhs - rhs).into_linexpr()?;
    let rhs = -lhs.get_offset();
    Ok((lhs, sense, rhs))
  }

  pub(crate) fn into_normalised_quad(self) -> (QuadExpr, ConstrSense, f64) {
    let ConstrExpr{lhs, rhs, sense} = self;
    let lhs = (lhs - rhs).into_quadexpr();
    let rhs = -lhs.get_offset();
    (lhs, sense, rhs)
  }
}
