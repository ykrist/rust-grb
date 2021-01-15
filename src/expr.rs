// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use attr;
use super::{Var, Model};
use error::Result;
use itertools::*;

use std::ops::{Add, Sub, Mul, Neg, AddAssign, Div};
use std::iter::{Sum};


/// Linear expression of variables
///
/// A linear expression consists of a constant term plus a list of coefficients and variables.
#[derive(Debug, Clone, Default)]
pub struct LinExpr {
  vars: Vec<Var>,
  coeff: Vec<f64>,
  offset: f64,
}

impl<'a> From<&'a Var> for LinExpr {
  fn from(var: &Var) -> LinExpr {
    LinExpr::new() + var
  }
}

impl From<Var> for LinExpr {
  fn from(var: Var) -> LinExpr {
    LinExpr::from(&var)
  }
}

impl From<f64> for LinExpr {
  fn from(offset: f64) -> LinExpr {
    LinExpr::new() + offset
  }
}

// FIXME : LinExpr and QuadExpr make no attempt to stop duplicate variables being stored,
//  which is currently a bug, since Model doesn't check for duplicates when adding constraint/objectives
//  Now that Var implements Hash, we can replace simply wrap a HashMap<Var, f64>.  Will likely use
//  FnvHashMap due Var Hash impl simply hashing two u32s.
impl LinExpr {
  /// Create an empty linear expression.
  pub fn new() -> Self {
    LinExpr::default()
  }

  /// Add a linear term into the expression.
  pub fn add_term(mut self, coeff: f64, var: Var) -> Self {
    self.coeff.push(coeff);
    self.vars.push(var);
    self
  }

  /// Add linear terms into the expression. Panics if the lengths do not match.
  pub fn add_terms(mut self, coeffs: &[f64], vars: &[Var]) -> Self {
    assert_eq!(coeffs.len(), vars.len());
    self.coeff.extend_from_slice(coeffs);
    self.vars.extend_from_slice(vars);
    self
  }

  /// Add a constant into the expression.
  pub fn add_constant(mut self, constant: f64) -> Self {
    self.offset += constant;
    self
  }

  /// Get the constant offset
  pub fn get_offset(&self) -> f64 { self.offset }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let vars = model.get_values(attr::X, self.vars.as_slice())?;

    Ok(Zip::new((vars, self.coeff.iter())).fold(0.0, |acc, (ind, val)| acc + ind * val) + self.offset)
  }

  /// Decompose into variables, their coefficients and the offset, respectively.
  pub fn into_parts(self) -> (Vec<Var>, Vec<f64>, f64) { (self.vars, self.coeff, self.offset) }

  /// Returns an iterator over the terms excluding the offset (item type is `(&Var, &f64)`)
  pub fn iter_terms<'a>(&'a self) -> std::iter::Zip<std::slice::Iter<'a, Var>, std::slice::Iter<'a,   f64>> {
    (&self.vars).iter().zip((&self.coeff).iter())
  }
}


/// Quadratic expression of variables
///
/// A quadratic expression consists of a linear expression and a set of
/// variable-variable-coefficient triples to express the quadratic term.
#[derive(Debug, Clone, Default)]
pub struct QuadExpr {
  linexpr : LinExpr,
  qrow: Vec<Var>,
  qcol: Vec<Var>,
  qval: Vec<f64>,
}

impl QuadExpr {
  pub fn new() -> Self {
    QuadExpr::default()
  }

  #[allow(clippy::type_complexity)]
  pub fn into_parts(self) -> (Vec<Var>, Vec<Var>, Vec<f64>, Vec<Var>, Vec<f64>, f64) {
    let (lvars, lcoeff, offset) = self.linexpr.into_parts();
    (self.qrow, self.qcol, self.qval, lvars, lcoeff, offset)
  }
  /// Add a linear term into the expression.
  pub fn add_term(mut self, coeff: f64, var: Var) -> Self {
    self.linexpr = self.linexpr.add_term(coeff, var);
    self
  }

  /// Add a quadratic term into the expression.
  pub fn add_qterm(mut self, coeff: f64, row: Var, col: Var) -> Self {
    self.qval.push(coeff);
    self.qrow.push(row);
    self.qcol.push(col);
    self
  }

  /// Add a constant into the expression.
  /// FIXME this should take &mut self and possibly return &mut self
  pub fn add_constant(mut self, constant: f64) -> Self {
    self.linexpr = self.linexpr.add_constant(constant);
    self
  }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let qrow = model.get_values(attr::X, &self.qrow)?;
    let qcol = model.get_values(attr::X, &self.qcol)?;

    let val = self.linexpr.get_value(model)? +
      Zip::new((qrow.iter(), qcol.iter(), self.qval.iter())).fold(0.0, |acc, (&x, &y, &a)| acc + a * x * y);
    Ok(val)
  }
}

// Conversion into QuadExpr

impl Into<QuadExpr> for Var {
  fn into(self) -> QuadExpr { QuadExpr::new().add_term(1.0, self) }
}

impl<'a> Into<QuadExpr> for &'a Var {
  fn into(self) -> QuadExpr { QuadExpr::new().add_term(1.0, self.clone()) }
}

impl Into<QuadExpr> for LinExpr {
  fn into(self) -> QuadExpr {
    QuadExpr {
      linexpr: self,
      qrow: Vec::new(),
      qcol: Vec::new(),
      qval: Vec::new(),
    }
  }
}


// /////// Operator definition.


/// `Var` + `Var`  => `LinExpr`
impl Add for Var {
  type Output = LinExpr;
  fn add(self, rhs: Var) -> LinExpr { LinExpr::new().add_term(1.0, self).add_term(1.0, rhs) }
}

impl<'a> Add<&'a Var> for Var {
  type Output = LinExpr;
  fn add(self, rhs: &Var) -> LinExpr { LinExpr::new().add_term(1.0, self).add_term(1.0, rhs.clone()) }
}

impl<'a> Add<Var> for &'a Var {
  type Output = LinExpr;
  fn add(self, rhs: Var) -> LinExpr { LinExpr::new().add_term(1.0, self.clone()).add_term(1.0, rhs) }
}

impl<'a, 'b> Add<&'b Var> for &'a Var {
  type Output = LinExpr;
  fn add(self, rhs: &Var) -> LinExpr { LinExpr::new().add_term(1.0, self.clone()).add_term(1.0, rhs.clone()) }
}

impl Add<f64> for Var {
  type Output = LinExpr;
  fn add(self, rhs: f64) -> LinExpr { LinExpr::new() + self + rhs }
}

impl<'a> Add<f64> for &'a Var {
  type Output = LinExpr;
  fn add(self, rhs: f64) -> LinExpr { LinExpr::new() + self.clone() + rhs }
}

/// `Var` - `Var` => `LinExpr`
impl Sub for Var {
  type Output = LinExpr;
  fn sub(self, rhs: Var) -> LinExpr { LinExpr::new().add_term(1.0, self).add_term(-1.0, rhs) }
}

impl<'a> Sub<&'a Var> for Var {
  type Output = LinExpr;
  fn sub(self, rhs: &Var) -> LinExpr { LinExpr::new().add_term(1.0, self).add_term(-1.0, rhs.clone()) }
}

impl<'a> Sub<Var> for &'a Var {
  type Output = LinExpr;
  fn sub(self, rhs: Var) -> LinExpr { LinExpr::new().add_term(1.0, self.clone()).add_term(-1.0, rhs) }
}

impl<'a, 'b> Sub<&'b Var> for &'a Var {
  type Output = LinExpr;
  fn sub(self, rhs: &Var) -> LinExpr { LinExpr::new().add_term(1.0, self.clone()).add_term(-1.0, rhs.clone()) }
}

impl Sub<LinExpr> for Var {
  type Output = LinExpr;
  fn sub(self, expr: LinExpr) -> LinExpr { self + (-expr) }
}

impl<'a> Sub<LinExpr> for &'a Var {
  type Output = LinExpr;
  fn sub(self, expr: LinExpr) -> LinExpr { self.clone() + (-expr) }
}

impl Sub<Var> for f64 {
  type Output = LinExpr;
  fn sub(self, rhs: Var) -> LinExpr { LinExpr::new() + self + (-rhs) }
}

impl<'a> Sub<&'a Var> for f64 {
  type Output = LinExpr;
  fn sub(self, rhs: &Var) -> LinExpr { LinExpr::new() + self + (-rhs.clone()) }
}


/// -`Var` => `LinExpr`
impl Neg for Var {
  type Output = LinExpr;
  fn neg(self) -> LinExpr { LinExpr::new().add_term(-1.0, self) }
}

impl<'a> Neg for &'a Var {
  type Output = LinExpr;
  fn neg(self) -> LinExpr { LinExpr::new().add_term(-1.0, self.clone()) }
}


/// `Var` * `f64` => `LinExpr`
impl Mul<f64> for Var {
  type Output = LinExpr;
  fn mul(self, rhs: f64) -> Self::Output { LinExpr::new().add_term(rhs, self) }
}

impl<'a> Mul<f64> for &'a Var {
  type Output = LinExpr;
  fn mul(self, rhs: f64) -> Self::Output { LinExpr::new().add_term(rhs, self.clone()) }
}

impl Mul<Var> for f64 {
  type Output = LinExpr;
  fn mul(self, rhs: Var) -> Self::Output { LinExpr::new().add_term(self, rhs) }
}

impl<'a> Mul<&'a Var> for f64 {
  type Output = LinExpr;
  fn mul(self, rhs: &'a Var) -> Self::Output { LinExpr::new().add_term(self, rhs.clone()) }
}


/// `Var` * `Var` => `QuadExpr`
impl Mul for Var {
  type Output = QuadExpr;
  fn mul(self, rhs: Var) -> Self::Output { QuadExpr::new().add_qterm(1.0, self, rhs) }
}

impl<'a> Mul<&'a Var> for Var {
  type Output = QuadExpr;
  fn mul(self, rhs: &Var) -> Self::Output { QuadExpr::new().add_qterm(1.0, self, rhs.clone()) }
}

impl<'a> Mul<Var> for &'a Var {
  type Output = QuadExpr;
  fn mul(self, rhs: Var) -> Self::Output { QuadExpr::new().add_qterm(1.0, self.clone(), rhs) }
}

impl<'a, 'b> Mul<&'b Var> for &'a Var {
  type Output = QuadExpr;
  fn mul(self, rhs: &Var) -> Self::Output { QuadExpr::new().add_qterm(1.0, self.clone(), rhs.clone()) }
}


/// `LinExpr` + `Var` => `LinExpr`
impl Add<LinExpr> for Var {
  type Output = LinExpr;
  fn add(self, rhs: LinExpr) -> LinExpr { rhs.add_term(1.0, self) }
}

impl<'a> Add<LinExpr> for &'a Var {
  type Output = LinExpr;
  fn add(self, rhs: LinExpr) -> LinExpr { rhs.add_term(1.0, self.clone()) }
}

impl Add<Var> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: Var) -> LinExpr { self.add_term(1.0, rhs) }
}

impl<'a> Add<&'a Var> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: &'a Var) -> LinExpr { self.add_term(1.0, rhs.clone()) }
}


/// `LinExpr` + `f64` => `LinExpr`
impl Add<f64> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: f64) -> Self::Output { self.add_constant(rhs) }
}

impl Add<LinExpr> for f64 {
  type Output = LinExpr;
  fn add(self, rhs: LinExpr) -> Self::Output { rhs.add_constant(self) }
}


/// `LinExpr` - `f64` => `LinExpr`
impl Sub<f64> for LinExpr {
  type Output = LinExpr;
  fn sub(self, rhs: f64) -> Self::Output { self.add_constant(-rhs) }
}

/// `f64` - `LinExpr` => `LinExpr`
impl Sub<LinExpr> for f64 {
  type Output = LinExpr;
  fn sub(self, rhs: LinExpr) -> Self::Output {
    self + (-rhs)
  }
}


impl Add for LinExpr {
  type Output = LinExpr;
  fn add(mut self, rhs: LinExpr) -> Self::Output {
    self += rhs;
    self
  }
}

impl Neg for LinExpr {
  type Output = LinExpr;
  fn neg(mut self) -> LinExpr {
    for coeff in &mut self.coeff {
      *coeff = -*coeff;
    }
    self.offset = -self.offset;
    self
  }
}

impl AddAssign for LinExpr {
  fn add_assign(&mut self, rhs: LinExpr) {
    for (var, &coeff) in rhs.vars.into_iter().zip(rhs.coeff.iter()) {
      if let Some(idx) = self.vars.iter().position(|v| *v == var) {
        self.coeff[idx] += coeff;
      } else {
        self.vars.push(var);
        self.coeff.push(coeff);
      }
    }
    self.offset += rhs.offset;
  }
}

impl AddAssign<Var> for LinExpr {
  fn add_assign(&mut self, rhs: Var) {
    let expr: LinExpr = rhs.into();
    *self += expr;
  }
}

impl Sub for LinExpr {
  type Output = LinExpr;
  fn sub(self, rhs: LinExpr) -> Self::Output {
    self + (-rhs)
  }
}

impl Mul<f64> for LinExpr {
  type Output = LinExpr;
  fn mul(mut self, rhs: f64) -> Self::Output {
    for coeff in &mut self.coeff {
      *coeff *= rhs;
    }
    self.offset *= rhs;
    self
  }
}

impl Div<f64> for LinExpr {
  type Output = LinExpr;
  fn div(mut self, rhs: f64) -> Self::Output {
    for coeff in &mut self.coeff {
      *coeff /= rhs;
    }
    self.offset /= rhs;
    self
  }
}

impl Mul<LinExpr> for f64 {
  type Output = LinExpr;
  fn mul(self, rhs: LinExpr) -> Self::Output {
    rhs * self
  }
}

impl Mul<f64> for QuadExpr {
  type Output = QuadExpr;
  fn mul(mut self, rhs: f64) -> Self::Output {
    self.linexpr = self.linexpr * rhs;
    self.qval.iter_mut().for_each(|c| *c *= rhs);
    self
  }
}

impl Sum for LinExpr {
  fn sum<I: Iterator<Item=LinExpr>>(iter: I) -> LinExpr {
    iter.fold(LinExpr::new(), |acc, expr| acc + expr)
  }
}


impl Add<LinExpr> for QuadExpr {
  type Output = QuadExpr;
  #[allow(clippy::assign_op_pattern)]
  fn add(mut self, rhs: LinExpr) -> Self::Output {
    self.linexpr = self.linexpr + rhs;
    self
  }
}

impl Sub<LinExpr> for QuadExpr {
  type Output = QuadExpr;
  #[allow(clippy::assign_op_pattern)]
  fn sub(mut self, rhs: LinExpr) -> Self::Output {
    self.linexpr = self.linexpr - rhs;
    self
  }
}

impl Add for QuadExpr {
  type Output = QuadExpr;
  #[allow(clippy::assign_op_pattern)]
  fn add(mut self, rhs: QuadExpr) -> QuadExpr {
    self.linexpr = self.linexpr + rhs.linexpr;
    self.qrow.extend(rhs.qrow);
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval);
    self
  }
}

impl Sub for QuadExpr {
  type Output = QuadExpr;
  #[allow(clippy::assign_op_pattern)]
  fn sub(mut self, rhs: QuadExpr) -> QuadExpr {
    self.linexpr = self.linexpr - rhs.linexpr;
    self.qrow.extend(rhs.qrow);
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval.into_iter().map(|c| -c));
    self
  }
}
