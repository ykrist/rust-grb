use super::ffi;
use std::ops::{Add, Sub, Mul, Neg};
use model::Var;


#[derive(Copy,Clone)]
pub struct LinTerm(Var, f64);

#[derive(Copy,Clone)]
pub struct QuadTerm(Var, Var, f64);


pub struct LinExpr {
  vars: Vec<i32>,
  coeff: Vec<f64>,
  offset: f64
}


impl LinExpr {
  pub fn new() -> LinExpr {
    LinExpr {
      vars: Vec::new(),
      coeff: Vec::new(),
      offset: 0.0
    }
  }

  pub fn num_vars(&self) -> usize { self.vars.len() }

  pub fn vars_slice(&self) -> &[ffi::c_int] { self.vars.as_slice() }

  pub fn coeff_slice(&self) -> &[ffi::c_double] { self.coeff.as_slice() }

  pub fn get_offset(&self) -> f64 { self.offset }

  pub fn term(mut self, term: LinTerm) -> LinExpr {
    self.vars.push((term.0).0);
    self.coeff.push(term.1);
    self
  }

  pub fn offset(mut self, offset: f64) -> LinExpr {
    self.offset += offset;
    self
  }
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



pub struct QuadExpr {
  lind: Vec<i32>,
  lval: Vec<f64>,
  qrow: Vec<i32>,
  qcol: Vec<i32>,
  qval: Vec<f64>,
  offset: f64
}

impl QuadExpr {
  pub fn new() -> QuadExpr {
    QuadExpr {
      lind: Vec::new(),
      lval: Vec::new(),
      qrow: Vec::new(),
      qcol: Vec::new(),
      qval: Vec::new(),
      offset: 0.0
    }
  }

  pub fn term(mut self, term: LinTerm) -> QuadExpr {
    self.lind.push((term.0).0);
    self.lval.push(term.1);
    self
  }

  pub fn qterm(mut self, term: QuadTerm) -> QuadExpr {
    self.qrow.push((term.0).0);
    self.qcol.push((term.1).0);
    self.qval.push(term.2);
    self
  }

  pub fn offset(mut self, offset: f64) -> QuadExpr {
    self.offset += offset;
    self

  }

  pub fn get_offset(&self) -> f64 { self.offset }

  pub fn lind_slice(&self) -> &[ffi::c_int] { self.lind.as_slice() }
  pub fn qrow_slice(&self) -> &[ffi::c_int] { self.qrow.as_slice() }
  pub fn qcol_slice(&self) -> &[ffi::c_int] { self.qcol.as_slice() }

  pub fn lval_slice(&self) -> &[ffi::c_double] { self.lval.as_slice() }
  pub fn qval_slice(&self) -> &[ffi::c_double] { self.qval.as_slice() }

  pub fn lin_len(&self) -> usize { self.lval.len() }
  pub fn quad_len(&self) -> usize { self.qval.len() }
}



impl Neg for LinTerm {
  type Output = LinTerm;
  fn neg(self) -> LinTerm { LinTerm(self.0, -self.1) }
}

impl Mul<f64> for Var {
  type Output = LinTerm;
  fn mul(self, rhs: f64) -> LinTerm { LinTerm(self, rhs) }
}

impl Mul<Var> for f64 {
  type Output = LinTerm;
  fn mul(self, rhs: Var) -> LinTerm { LinTerm(rhs, self) }
}

impl Add<LinTerm> for LinTerm {
  type Output = LinExpr;
  fn add(self, rhs: LinTerm) -> LinExpr { LinExpr::new() + self + rhs }
}

impl Sub<LinTerm> for LinTerm {
  type Output = LinExpr;
  fn sub(self, rhs: LinTerm) -> LinExpr { LinExpr::new() + self - rhs }
}

impl Add<QuadTerm> for LinTerm {
  type Output = QuadExpr;
  fn add(self, rhs: QuadTerm) -> QuadExpr { QuadExpr::new() + self + rhs }
}

impl Sub<QuadTerm> for LinTerm {
  type Output = QuadExpr;
  fn sub(self, rhs: QuadTerm) -> QuadExpr { QuadExpr::new() + self - rhs }
}

impl Into<QuadExpr> for LinTerm {
  fn into(self) -> QuadExpr { QuadExpr::new().term(self) }
}

impl Neg for QuadTerm {
  type Output = QuadTerm;
  fn neg(self) -> QuadTerm { QuadTerm(self.0, self.1, -self.2) }
}

impl Mul for Var {
  type Output = QuadTerm;
  fn mul(self, rhs: Var) -> QuadTerm { QuadTerm(self, rhs, 1.0) }
}

impl Mul<f64> for QuadTerm {
  type Output = QuadTerm;
  fn mul(self, rhs: f64) -> QuadTerm { QuadTerm(self.0, self.1, self.2 * rhs) }
}

impl Mul<QuadTerm> for f64 {
  type Output = QuadTerm;
  fn mul(self, rhs: QuadTerm) -> QuadTerm { QuadTerm(rhs.0, rhs.1, self * rhs.2) }
}

impl Add for QuadTerm {
  type Output = QuadExpr;
  fn add(self, rhs: QuadTerm) -> QuadExpr { QuadExpr::new() + self + rhs }
}

impl Sub for QuadTerm {
  type Output = QuadExpr;
  fn sub(self, rhs: QuadTerm) -> QuadExpr { QuadExpr::new() + self - rhs }
}

impl Add<LinTerm> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: LinTerm) -> LinExpr { self.term(rhs) }
}

impl Sub<LinTerm> for LinExpr {
  type Output = LinExpr;
  fn sub(self, rhs: LinTerm) -> LinExpr { self.term(-rhs) }
}

impl Add<f64> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: f64) -> LinExpr { self.offset(rhs) }
}

impl Sub<f64> for LinExpr {
  type Output = LinExpr;
  fn sub(self, rhs: f64) -> LinExpr { self.offset(-rhs) }
}

impl Add<QuadTerm> for LinExpr {
  type Output = QuadExpr;
  fn add(self, rhs: QuadTerm) -> QuadExpr { Into::<QuadExpr>::into(self).qterm(rhs) }
}

impl Sub<QuadTerm> for LinExpr {
  type Output = QuadExpr;
  fn sub(self, rhs: QuadTerm) -> QuadExpr { Into::<QuadExpr>::into(self).qterm(-rhs) }
}

impl Add<LinTerm> for QuadExpr {
  type Output = QuadExpr;
  fn add(self, rhs: LinTerm) -> QuadExpr { self.term(rhs) }
}

impl Sub<LinTerm> for QuadExpr {
  type Output = QuadExpr;
  fn sub(self, rhs: LinTerm) -> QuadExpr { self.term(-rhs) }
}

impl Add<QuadTerm> for QuadExpr {
  type Output = QuadExpr;
  fn add(self, rhs: QuadTerm) -> QuadExpr { self.qterm(rhs) }
}

impl Sub<QuadTerm> for QuadExpr {
  type Output = QuadExpr;
  fn sub(self, rhs: QuadTerm) -> QuadExpr { self.qterm(-rhs) }
}

impl Add<f64> for QuadExpr {
  type Output = QuadExpr;
  fn add(self, rhs: f64) -> QuadExpr { self.offset(rhs) }
}

impl Sub<f64> for QuadExpr {
  type Output = QuadExpr;
  fn sub(self, rhs: f64) -> QuadExpr { self.offset(-rhs) }
}
