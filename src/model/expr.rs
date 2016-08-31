// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use attr;
use super::{Var, Model};
use error::Result;
use itertools::*;

use std::ops::{Add, Sub, Mul, Neg};

/// Linear expression of variables
///
/// A linear expression consists of a constant term plus a list of coefficients and variables.
#[derive(Clone)]
pub struct LinExpr {
  vars: Vec<Var>,
  coeff: Vec<f64>,
  offset: f64
}

impl Into<(Vec<i32>, Vec<f64>, f64)> for LinExpr {
  fn into(self) -> (Vec<i32>, Vec<f64>, f64) {
    (self.vars.into_iter().map(|e| e.index()).collect(), self.coeff, self.offset)
  }
}


impl LinExpr {
  /// Create an empty linear expression.
  pub fn new() -> Self {
    LinExpr {
      vars: Vec::new(),
      coeff: Vec::new(),
      offset: 0.0
    }
  }

  /// Add a linear term into the expression.
  pub fn add_term(mut self, coeff: f64, var: Var) -> Self {
    self.coeff.push(coeff);
    self.vars.push(var);
    self
  }

  /// Add a constant into the expression.
  pub fn add_constant(mut self, constant: f64) -> Self {
    self.offset += constant;
    self
  }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let vars = try!(model.get_values(attr::X, self.vars.as_slice()));

    Ok(Zip::new((vars, self.coeff.iter())).fold(0.0, |acc, (ind, val)| acc + ind * val) + self.offset)
  }
}


/// Quadratic expression of variables
///
/// A quadratic expression consists of a linear expression and a set of
/// variable-variable-coefficient triples to express the quadratic term.
#[derive(Clone)]
pub struct QuadExpr {
  lind: Vec<Var>,
  lval: Vec<f64>,
  qrow: Vec<Var>,
  qcol: Vec<Var>,
  qval: Vec<f64>,
  offset: f64
}

impl Into<(Vec<i32>, Vec<f64>, Vec<i32>, Vec<i32>, Vec<f64>, f64)> for QuadExpr {
  fn into(self) -> (Vec<i32>, Vec<f64>, Vec<i32>, Vec<i32>, Vec<f64>, f64) {
    (self.lind.into_iter().map(|e| e.index()).collect(),
     self.lval,
     self.qrow.into_iter().map(|e| e.index()).collect(),
     self.qcol.into_iter().map(|e| e.index()).collect(),
     self.qval,
     self.offset)
  }
}

impl QuadExpr {
  pub fn new() -> Self {
    QuadExpr {
      lind: Vec::new(),
      lval: Vec::new(),
      qrow: Vec::new(),
      qcol: Vec::new(),
      qval: Vec::new(),
      offset: 0.0
    }
  }

  /// Add a linear term into the expression.
  pub fn add_term(mut self, coeff: f64, var: Var) -> Self {
    self.lind.push(var);
    self.lval.push(coeff);
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
  pub fn add_constant(mut self, constant: f64) -> Self {
    self.offset += constant;
    self
  }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let lind = try!(model.get_values(attr::X, self.lind.as_slice()));
    let qrow = try!(model.get_values(attr::X, self.qrow.as_slice()));
    let qcol = try!(model.get_values(attr::X, self.qcol.as_slice()));

    Ok(Zip::new((lind, self.lval.iter())).fold(0.0, |acc, (ind, val)| acc + ind * val) +
       Zip::new((qrow, qcol, self.qval.iter())).fold(0.0, |acc, (row, col, val)| acc + row * col * val) +
       self.offset)
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
      lind: self.vars,
      lval: self.coeff,
      offset: self.offset,
      qrow: Vec::new(),
      qcol: Vec::new(),
      qval: Vec::new()
    }
  }
}


/// /////// Operator definition.


// Var + Var  => LinExpr
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


/// / Var - Var  => LinExpr
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


// -Var  => LinExpr
impl Neg for Var {
  type Output = LinExpr;
  fn neg(self) -> LinExpr { LinExpr::new().add_term(-1.0, self) }
}
impl<'a> Neg for &'a Var {
  type Output = LinExpr;
  fn neg(self) -> LinExpr { LinExpr::new().add_term(-1.0, self.clone()) }
}



// Var * f64  => LinExpr
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


// Var * Var => QuadExpr
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


// LinExpr + Var  ==> LinExpr
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


// LinExpr + f64 => LinExpr
impl Add<f64> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: f64) -> Self::Output { self.add_constant(rhs) }
}
impl Add<LinExpr> for f64 {
  type Output = LinExpr;
  fn add(self, rhs: LinExpr) -> Self::Output { rhs.add_constant(self) }
}


// LinExpr - f64  => LinExpr
impl Sub<f64> for LinExpr {
  type Output = LinExpr;
  fn sub(self, rhs: f64) -> Self::Output { self.add_constant(-rhs) }
}

// f64 - LinExpr  => LinExpr
impl Sub<LinExpr> for f64 {
  type Output = LinExpr;
  fn sub(self, mut rhs: LinExpr) -> Self::Output {
    for c in rhs.coeff.iter_mut() {
      *c *= -1.0;
    }
    rhs.add_constant(-self)
  }
}


impl Add for LinExpr {
  type Output = LinExpr;
  fn add(mut self, rhs: LinExpr) -> Self::Output {
    self.vars.extend(rhs.vars);
    self.coeff.extend(rhs.coeff);
    self.offset += rhs.offset;
    self
  }
}

impl Sub for LinExpr {
  type Output = LinExpr;
  fn sub(mut self, rhs: LinExpr) -> Self::Output {
    self.vars.extend(rhs.vars);
    self.coeff.extend(rhs.coeff.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}



impl Mul<f64> for QuadExpr {
  type Output = QuadExpr;
  fn mul(mut self, rhs: f64) -> Self::Output {
    for i in 0..(self.lval.len()) {
      self.lval[i] *= rhs;
    }
    for j in 0..(self.qval.len()) {
      self.qval[j] *= rhs;
    }
    self.offset *= rhs;
    self
  }
}



impl Add<LinExpr> for QuadExpr {
  type Output = QuadExpr;
  fn add(mut self, rhs: LinExpr) -> Self::Output {
    self.lind.extend(rhs.vars);
    self.lval.extend(rhs.coeff);
    self.offset += rhs.offset;
    self
  }
}

impl Sub<LinExpr> for QuadExpr {
  type Output = QuadExpr;
  fn sub(mut self, rhs: LinExpr) -> Self::Output {
    self.lind.extend(rhs.vars);
    self.lval.extend(rhs.coeff.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}

impl Add for QuadExpr {
  type Output = QuadExpr;
  fn add(mut self, rhs: QuadExpr) -> QuadExpr {
    self.lind.extend(rhs.lind);
    self.lval.extend(rhs.lval);
    self.qrow.extend(rhs.qrow);
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval);
    self.offset += rhs.offset;
    self
  }
}

impl Sub for QuadExpr {
  type Output = QuadExpr;
  fn sub(mut self, rhs: QuadExpr) -> QuadExpr {
    self.lind.extend(rhs.lind);
    self.lval.extend(rhs.lval.into_iter().map(|c| -c));
    self.qrow.extend(rhs.qrow);
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}
