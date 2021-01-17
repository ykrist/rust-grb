// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.
//
use std::ops::{Add, Sub, Mul, Neg};
use std::iter::{Sum};
use fnv::FnvHashMap;
use std::fmt;
use std::fmt::Write;

use crate::{Var, Model, Result, Error};
use crate::attr;


#[derive(Debug, Clone)]
pub enum Expr {
  Linear(LinExpr),
  Quad(QuadExpr),
  Term(f64, Var),
  QTerm(f64, Var, Var),
  Constant(f64)
}

impl Expr {
  fn upgrade(self) -> Expr {
    use self::Expr::*;
    match self {
      Constant(x) => Linear(LinExpr::new()) + Constant(x),
      Term(a,x) => Linear(LinExpr::new()) + Term(a,x),
      QTerm(a, x, y) => Quad(QuadExpr::new()) + QTerm(a, x, y),
      Linear(e) => QuadExpr{linexpr: e, qcoeffs: FnvHashMap::default()}.into(),
      Quad(_) => unreachable!()
    }
  }
  pub fn into_quadexpr(self) -> QuadExpr {
    use self::Expr::*;
    match self {
      Quad(e) => e,
      other => other.upgrade().into_quadexpr()
    }
  }

  pub fn into_linexpr(self) -> Result<LinExpr> {
    use self::Expr::*;
    match self {
      Quad(..) | QTerm(..) => Err(Error::AlgebraicError("expression contains quadratic terms".to_string())),
      Linear(e) => Ok(e),
      other => other.upgrade().into_linexpr()
    }
  }
}


impl Default for Expr {
  fn default() -> Self { Expr::Constant(0.0) }
}

/// Linear expression of variables
///
/// A linear expression consists of a constant term plus a list of coefficients and variables.
#[derive(Debug, Clone, Default)]
pub struct LinExpr {
  coeff: FnvHashMap<Var, f64>,
  offset: f64,
}


/// Quadratic expression of variables
///
/// A quadratic expression consists of a linear expression and a set of
/// variable-variable-coefficient triples to express the quadratic term.
#[derive(Debug, Clone, Default)]
pub struct QuadExpr {
  linexpr : LinExpr,
  qcoeffs: FnvHashMap<(Var,Var), f64>
}


impl From<Var> for Expr {
  fn from(var: Var) -> Expr { Expr::Term(1.0, var) }
}

impl From<f64> for Expr {
  fn from(val: f64) -> Expr { Expr::Constant(val) }
}

impl From<LinExpr> for Expr {
  fn from(val: LinExpr) -> Expr { Expr::Linear(val) }
}

impl From<QuadExpr> for Expr {
  fn from(val: QuadExpr) -> Expr { Expr::Quad(val) }
}

impl<T: Clone + Into<Expr>> From<&T> for Expr {
  fn from(val: &T) -> Expr { val.clone().into() }
}


impl LinExpr {
  /// Create an empty linear expression.
  pub fn new() -> Self {
    LinExpr::default()
  }

  pub fn is_empty(&self) -> bool {
    self.offset.abs() < f64::EPSILON && self.coeff.is_empty()
  }

  /// Add a linear term into the expression.
  pub fn add_term(&mut self, coeff: f64, var: Var) -> &mut Self {
    self.coeff.entry(var).and_modify(|c| *c += coeff).or_insert(coeff);
    self
  }

  /// Add a constant into the expression.
  pub fn add_constant(&mut self, constant: f64) -> &mut Self {
    self.offset += constant;
    self
  }

  /// Get the constant offset
  pub fn get_offset(&self) -> f64 { self.offset }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let coeff = self.coeff.values();
    let vars : Vec<_> = self.coeff.keys().cloned().collect();
    let vals = model.get_values(attr::X, &vars)?;
    let total = coeff.zip(vals.into_iter()).map(|(&a, x)| a*x).sum::<f64>() + self.offset;
    Ok(total)
  }

  /// Decompose into variables, their coefficients and the offset, respectively.
  pub fn into_parts(self) -> (FnvHashMap<Var, f64>, f64) { (self.coeff, self.offset) }

  /// Returns an iterator over the terms excluding the offset (item type is `(&Var, &f64)`)
  pub fn iter_terms<'a>(&'a self) -> std::collections::hash_map::Iter<'a, Var, f64> {
    self.coeff.iter()
  }

  /// Multiply expression by a scalar
  pub fn mul_scalar(&mut self, val: f64) -> &mut Self {
    self.coeff.iter_mut().for_each(|(_, a)| *a *= val);
    self
  }

  pub(crate) fn get_coeff_indices(&self, model: &Model) -> Result<(Vec<i32>, Vec<f64>)> {
    let mut vinds = Vec::with_capacity(self.coeff.len());
    let mut coeff = Vec::with_capacity(self.coeff.len());
    for (x,&a) in &self.coeff {
      vinds.push(model.get_index(x)?);
      coeff.push(a);
    }
    Ok((vinds, coeff))
  }

  pub fn sparsify(&mut self) {
    self.coeff.retain(|_, a| a.abs() > f64::EPSILON);
  }
}

impl QuadExpr {
  pub fn new() -> Self {
    QuadExpr::default()
  }

  pub fn is_empty(&self) -> bool {
    self.qcoeffs.is_empty() && self.linexpr.is_empty()
  }

  #[allow(clippy::type_complexity)]
  pub fn into_parts(self) -> (FnvHashMap<(Var, Var), f64>, FnvHashMap<Var, f64>, f64) {
    let (lcoeff, offset) = self.linexpr.into_parts();
    (self.qcoeffs, lcoeff, offset)
  }
  /// Add a linear term into the expression.
  pub fn add_term(&mut self, coeff: f64, var: Var) -> &mut Self {
    self.linexpr.add_term(coeff, var);
    self
  }

  /// Add a quadratic term into the expression.
  pub fn add_qterm(&mut self, coeff: f64, rowvar: Var, colvar: Var) -> &mut Self {
    if rowvar.id() > colvar.id() { // we don't bother checking the model_id here, it gets check when this object is passed to the model
      return self.add_qterm(coeff, colvar, rowvar)
    }
    self.qcoeffs.entry((rowvar, colvar)).and_modify(|c| *c += coeff)
        .or_insert(coeff);
    self
  }

  /// Add a constant into the expression.
  pub fn add_constant(&mut self, constant: f64) -> &mut Self {
    self.linexpr.add_constant(constant);
    self
  }

  /// Get the offset value (constant)
  pub fn get_offset(&self) -> f64 { self.linexpr.get_offset() }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let coeff = self.qcoeffs.values();
    let mut rowvars = Vec::with_capacity(self.qcoeffs.len());
    let mut colvars = Vec::with_capacity(self.qcoeffs.len());
    for (x,y) in self.qcoeffs.keys().cloned() {
      rowvars.push(x);
      colvars.push(y);
    }
    let rowvals = model.get_values(attr::X, &rowvars)?;
    let colvals = model.get_values(attr::X, &colvars)?;
    let total = coeff.zip(rowvals.into_iter())
        .zip(colvals.into_iter())
        .map(|((&a, x), y)| a*x*y).sum::<f64>()  + self.linexpr.get_value(model)?;
    Ok(total)
  }

  /// Multiply expression by a scalar
  pub fn mul_scalar(&mut self, val: f64) -> &mut Self {
    self.linexpr.mul_scalar(val);
    self.qcoeffs.iter_mut().for_each(|(_, a)| *a *= val);
    self
  }

  pub(crate) fn get_coeff_indices(&self, model: &Model) -> Result<(Vec<i32>, Vec<i32>, Vec<f64>, Vec<i32>, Vec<f64>)> {
    let (linds, lcoeffs) = self.linexpr.get_coeff_indices(model)?;
    let mut rowinds = Vec::with_capacity(self.qcoeffs.len());
    let mut colinds = Vec::with_capacity(self.qcoeffs.len());
    let mut coeff = Vec::with_capacity(self.qcoeffs.len());
    for ((x,y),&a) in &self.qcoeffs {
      rowinds.push(model.get_index(x)?);
      colinds.push(model.get_index(y)?);
      coeff.push(a);
    }
    Ok((rowinds, colinds, coeff, linds, lcoeffs))
  }

  pub fn sparsify(&mut self) {
    self.linexpr.sparsify();
    self.qcoeffs.retain(|_, a| a.abs() > f64::EPSILON);
  }
}


impl Add for Expr {
  type Output = Self;
  fn add(self, rhs: Self) -> Self {
    use self::Expr::*;
    match (self, rhs) {
      (Constant(a), Constant(b)) => Constant(a + b),
      (Constant(c), Term(a, x)) => {
        let mut e = LinExpr::new();
        e.add_constant(c);
        e.add_term(a, x);
        e.into()
      }
      (Constant(c), QTerm(a, x, y)) => {
        let mut e = QuadExpr::new();
        e.add_qterm(a, x, y);
        e.add_constant(c);
        e.into()
      }
      (Constant(c), Linear(mut e)) => {
        e.add_constant(c);
        e.into()
      }
      (Constant(c), Quad(mut e)) => {
        e.add_constant(c);
        e.into()
      }
      (Term(a,x), Term(b,y)) => {
        let mut e = LinExpr::new();
        e.add_term(a,x);
        e.add_term(b,y);
        e.into()
      }
      (Term(a, x), QTerm(b, y1, y2)) => {
        let mut e = QuadExpr::new();
        e.add_term(a,x);
        e.add_qterm(b,y1,y2);
        e.into()
      }
      (Term(a, x), Linear(mut e)) => {
        e.add_term(a, x);
        e.into()
      }
      (Term(a, x), Quad(mut e)) => {
        e.add_term(a,x);
        e.into()
      }
      (QTerm(a,x1, x2), QTerm(b, y1,y2)) => {
        let mut e = QuadExpr::new();
        e.add_qterm(a, x1, x2);
        e.add_qterm(b, y1, y2);
        e.into()
      }
      (QTerm(a, x1, x2), Linear(e)) => {
        let mut e = QuadExpr{ linexpr: e, qcoeffs: FnvHashMap::default() };
        e.add_qterm(a, x1, x2);
        e.into()
      }
      (QTerm(a, x1, x2), Quad(mut e)) => {
        e.add_qterm(a, x1, x2);
        e.into()
      }
      (Linear(mut e1), Linear(e2)) => {
        let (coeffs, c) = e2.into_parts();
        e1.add_constant(c);
        for (x, a) in coeffs {
          e1.add_term(a, x);
        }
        e1.into()
      }
      (Linear(le), Quad(mut qe)) => {
        qe.linexpr = (Linear(qe.linexpr) + Linear(le)).into_linexpr().unwrap();
        qe.into()
      }
      (Quad(mut e1), Quad(e2)) => {
        let (qcoeffs, lcoeffs, c) = e2.into_parts();
        for ((x,y),a) in qcoeffs {
          e1.add_qterm(a,x,y);
        }
        e1.linexpr = (Linear(e1.linexpr) + Linear(LinExpr{ coeff: lcoeffs, offset: c})).into_linexpr().unwrap();
        e1.into()
      }
      // swap operands
      (lhs, rhs) => { rhs + lhs }
    }
  }
}


impl Sub for Expr {
  type Output = Self;
  fn sub(self, rhs: Self) -> Self { self + (-rhs) }
}


impl Add for Var {
  type Output = Expr;
  fn add(self, rhs: Self) -> Expr {
    let lhs : Expr = self.into();
    let rhs : Expr = rhs.into();
    lhs + rhs
  }
}

impl Mul for Var {
  type Output = Expr;
  fn mul(self, rhs: Self) -> Expr {
    Expr::QTerm(1.0, self, rhs)
  }
}


impl Sub for Var {
  type Output = Expr;
  fn sub(self, rhs: Self) -> Expr { self + (-rhs) }
}


impl Add for &Var {
  type Output = Expr;
  fn add(self, rhs: &Var) -> Expr { self.clone() + rhs.clone() }
}


impl Mul for &Var {
  type Output = Expr;
  fn mul(self, rhs: &Var) -> Expr { self.clone() * rhs.clone() }
}


impl Sub for &Var {
  type Output = Expr;
  fn sub(self, rhs: &Var) -> Expr {self.clone() - rhs.clone()}
}



impl Mul<f64> for Expr {
  type Output = Expr;
  fn mul(self, rhs: f64) -> Expr {
    use self::Expr::*;
    match self {
      Constant(a) => Constant(a * rhs),
      Term(a, x) => Term(a*rhs, x),
      QTerm(a, x, y) => QTerm(a*rhs, x, y),
      Linear(mut e) => {
        e.mul_scalar(rhs);
        e.into()
      }
      Quad(mut e) => {
        e.mul_scalar(rhs);
        e.into()
      }
    }
  }
}

impl Mul<Expr> for f64 {
  type Output = Expr;
  fn mul(self, rhs: Expr) -> Expr { rhs*self }
}

macro_rules! impl_f64_mul_expr_for_unwrapped {
  ($($t:ty),+) => {
    $(
      impl Mul<$t> for f64 {
        type Output = Expr;
        fn mul(self, rhs: $t) -> Expr { self * <$t as Into<Expr>>::into(rhs) }
      }

      impl Mul<&$t> for f64 {
        type Output = Expr;
        fn mul(self, rhs: &$t) -> Expr { self * <$t as Into<Expr>>::into(rhs.clone()) }
      }

      impl Mul<f64> for $t {
        type Output = Expr;
        fn mul(self, rhs: f64) -> Expr { rhs*self }
      }

      impl Mul<f64> for &$t {
        type Output = Expr;
        fn mul(self, rhs: f64) -> Expr { rhs*self.clone() }
      }

    )+
  }
}

impl_f64_mul_expr_for_unwrapped!( Var, LinExpr, QuadExpr );

macro_rules! impl_add_expr_for_unwrapped {
  ($($t:ty),+) => {
    $(
      impl Add<$t> for Expr {
        type Output = Expr;
        fn add(self, rhs: $t) -> Expr { self + <$t as Into<Expr>>::into(rhs) }
      }

      impl Add<&$t> for Expr {
        type Output = Expr;
        fn add(self, rhs: &$t) -> Expr { self + rhs.clone() }
      }

      impl Add<Expr> for $t {
        type Output = Expr;
        fn add(self, rhs: Expr) -> Expr { rhs + self }
      }

      impl Add<Expr> for &$t {
        type Output = Expr;
        fn add(self, rhs: Expr) -> Expr { rhs + self.clone() }
      }
    )+
  }
}

impl_add_expr_for_unwrapped!( f64, Var, LinExpr, QuadExpr );

impl Neg for Var {
  type Output = Expr;
  fn neg(self) -> Expr { Expr::Term(-1.0, self) }
}

impl Neg for &Var {
  type Output = Expr;
  fn neg(self) -> Expr { -self.clone() }
}

impl Neg for Expr {
  type Output = Expr;
  fn neg(self) -> Expr {
    use self::Expr::*;
    match self {
      Constant(a) => Constant(-a),
      Term(a,x) => Term(-a, x),
      QTerm(a, x, y) => QTerm(-a, x, y),
      other => -1.0*other
    }
  }
}

impl<A: Into<Expr>> Sum<A> for Expr {
  fn sum<I>(mut iter: I) -> Expr where I: Iterator<Item=A> {
    if let Some(total) = iter.next() {
      let mut total: Expr = total.into();
      for x in iter {
        total = total + x.into();
      }
      total
    } else {
      Expr::Constant(0.0)
    }
  }
}


pub struct Attached<'a, T> {
  inner: &'a T,
  model: &'a Model,
}

pub trait AttachModel {
  fn attach<'a>(&'a self, model: &'a Model) -> Attached<'a, Self> where Self: Sized {
    Attached{ inner: self, model }
  }
}

impl AttachModel for LinExpr {}
impl AttachModel for QuadExpr {}
impl AttachModel for Expr {}

fn float_fmt_helper(x: f64, ignore_val: f64) -> (Option<f64>, bool) {
  let positive = x > -f64::EPSILON;
  if (x-ignore_val) < f64::EPSILON {
    (None, positive)
  } else if positive {
    (Some(x), positive)
  } else {
    (Some(-x), positive)
  }
}

impl From<Error> for fmt::Error {
  fn from(err: Error) -> fmt::Error {
    eprintln!("fmt error cause by: {}", err);
    fmt::Error{}
  }
}

impl fmt::Debug for Attached<'_, LinExpr> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.inner.is_empty() {
      return f.write_str("<empty LinExpr>")
    }

    let (offset, positive) = float_fmt_helper(self.inner.get_offset(), 0.0);

    let mut is_first_term = false;
    if let Some(offset) = offset {
      f.write_fmt(format_args!("{}", if positive {offset} else { -offset }))?;
    } else {
      is_first_term = true;
    }

    for (var, &coeff) in self.inner.iter_terms() {
      let varname = var.get(&self.model, attr::VarName)?;
      let (coeff, positive) = float_fmt_helper(coeff, 1.0);

      // write the operator with the previous term
      if !is_first_term {
        f.write_str(if positive {" + "} else {" - "})?;
      } else  {
        is_first_term = false;
        if !positive {
          f.write_char('-')?;
        }
      }
      if let Some(coeff) = coeff {
        f.write_fmt(format_args!("{} {}", coeff, varname))?;
      } else {
        f.write_str(&varname)?;
      }
    }
    Ok(())
  }
}

impl fmt::Debug for Attached<'_, QuadExpr> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.inner.is_empty() {
      return f.write_str("<empty QuadExpr>")
    }

    let mut is_first_term = false;
    if self.inner.linexpr.is_empty() {
      is_first_term = true
    } else {
      self.inner.linexpr.attach(self.model).fmt(f)?;
    }

    for ((x,y), &coeff) in &self.inner.qcoeffs {
      let xname = x.get(self.model, attr::VarName)?;
      let yname = y.get(self.model, attr::VarName)?;
      let (coeff, positive) = float_fmt_helper(coeff, 1.0);
      if is_first_term {
        is_first_term = false;
        if !positive {
          f.write_char('-')?;
        }
      } else {
        f.write_str(if positive {" + "} else {" - "})?;
      }
      if let Some(coeff) = coeff {
        f.write_fmt(format_args!("{} {}*{}", coeff, xname, yname))?;
      } else {
        f.write_fmt(format_args!("{}*{}", xname, yname))?;
      }
    }
    Ok(())
  }
}

impl fmt::Debug for Attached<'_, Expr> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use self::Expr::*;
    match &self.inner {
      Constant(a) => f.write_fmt(format_args!("{}", a)),
      Term(a,x) => {
        let varname = x.get(self.model, attr::VarName)?;
        if (a-1.0).abs() < f64::EPSILON {
          f.write_fmt(format_args!("{}", varname))
        } else {
          f.write_fmt(format_args!("{} {}", a, varname))
        }
      },
      QTerm(a,x, y) => {
        let xname = x.get(self.model, attr::VarName)?;
        let yname = y.get(self.model, attr::VarName)?;
        if (a-1.0).abs() < f64::EPSILON {
          f.write_fmt(format_args!("{}*{}", xname, yname))
        } else {
          f.write_fmt(format_args!("{} {}*{}", a, xname, yname))
        }
      }
      Linear(e) => e.attach(self.model).fmt(f),
      Quad(e) => e.attach(self.model).fmt(f),
    }
  }
}


#[allow(unused_variables)]
#[cfg(test)]
mod tests {
  use super::*;
  use env::Env;
  use model::VarType::Binary;

  macro_rules! make_model_with_vars {
    ($model:ident, $($var:ident),+) => {
      let env = Env::new("gurobi.log").unwrap();
      let mut $model = Model::new("test", &env).unwrap();
      $(
        let $var = $model.add_var(stringify!($var), Binary, 0.0, 0.0, 0.0, &[], &[]).unwrap();
      )+
    }
  }

  #[test]
  fn simple() {
    make_model_with_vars!(model, x, y);
    let e = x.clone() * y.clone() + 1.0 + x.clone() + (2.0*y.clone());
    e.into_linexpr().unwrap_err(); // should be quadratic
  }

  #[test]
  fn nested() {
    make_model_with_vars!(model, x, y);
    let e = (x.clone() * y.clone())*3.0 + 2.0*(x.clone() + 2.0*y.clone());
  }

  #[test]
  fn subtract() {
    make_model_with_vars!(model, x, y, z);
    let _ = x.clone() - y.clone();
    let e = y.clone()*x.clone() - x.clone()*y.clone();
    dbg!(e.attach(&model));
    let mut e = e.into_quadexpr();
    assert!(!e.is_empty());
    e.sparsify();
    assert!(e.is_empty());
  }

  #[test]
  fn negate() {
    make_model_with_vars!(model, x);
    let q = -x.clone();
    let y = -q;
    if let Expr::Term(a, var) = y {
      assert_eq!(x, var);
      assert_eq!(a, 1.0);
    } else {
      panic!("{:?}", y);
    }
    let q = -(x.clone()*x.clone());
    eprintln!("{:?}", q.attach(&model));
  }

  #[test]
  fn summation() {
    make_model_with_vars!(model, x,y,z);
    let vars = [x.clone(),y.clone(),z.clone(),x.clone()];
    let e : Expr = vars.iter().cloned().sum();
    eprintln!("{:?}", &e);
    let e = e.into_linexpr().unwrap();
    assert_eq!(e.coeff.len(), 3);

    let vars = [2.0*x.clone(),-y.clone(),-z.clone(),0.2*x.clone()];
    let e : Expr = vars.iter().cloned().sum();
    let e = e.into_linexpr().unwrap();
    assert_eq!(e.coeff.len(), 3);
  }

  #[test]
  fn linexpr_debug_fmt() {
    make_model_with_vars!(m, x, y);
    let e = 2.0 * y.clone();
    let s = format!("{:?}", e.attach(&m));
    assert_eq!("2 y", s.to_string());
    eprintln!("{}", s);
    eprintln!("{:?}", (x.clone()*y.clone() + 2.0*(x.clone()*x.clone())).attach(&m));
  }
}
