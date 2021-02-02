#![allow(clippy::many_single_char_names)]
use std::ops::{Add, Sub, Mul, Neg};
use std::iter::{Sum};
use std::fmt;
use std::fmt::Write;
use fnv::FnvHashMap;

use crate::{Var, Model, Result, Error};
use crate::attr;


/// An algbraic expression of variables.
#[derive(Debug, Clone)]
pub enum Expr {
  /// A quadratic expression
  Quad(QuadExpr),
  /// A linear expression
  Linear(LinExpr),
  /// A single quadratic term
  QTerm(f64, Var, Var),
  /// A single linear term
  Term(f64, Var),
  /// A constant term
  Constant(f64)
}


impl Expr {
  fn into_higher_order(self) -> Expr {
    use self::Expr::*;
    match self {
      Constant(x) => Linear(LinExpr::new()) + Constant(x),
      Term(a,x) => Linear(LinExpr::new()) + Term(a,x),
      QTerm(a, x, y) => Quad(QuadExpr::new()) + QTerm(a, x, y),
      Linear(e) => QuadExpr{linexpr: e, qcoeffs: FnvHashMap::default()}.into(),
      Quad(_) => unreachable!()
    }
  }

  pub fn is_linear(&self) -> bool {
    !matches!(self, Expr::QTerm(..) | Expr::Quad(..))
  }

  /// Transform into a [`QuadExpr`], possibly with no quadratic terms)
  pub fn into_quadexpr(self) -> QuadExpr {
    use self::Expr::*;
    match self {
      Quad(e) => e,
      other => other.into_higher_order().into_quadexpr()
    }
  }

  /// Transform into a [`QuadExpr`], possibly with no quadratic terms)
  ///
  /// # Errors
  /// Returns an [`Error::AlgebraicError`] if `Expr` is not linear.
  pub fn into_linexpr(self) -> Result<LinExpr> {
    use self::Expr::*;
    match self {
      Quad(..) | QTerm(..) => Err(Error::AlgebraicError("expression contains quadratic terms".to_string())),
      Linear(e) => Ok(e),
      other => other.into_higher_order().into_linexpr()
    }
  }
}


impl Default for Expr {
  fn default() -> Self { Expr::Constant(0.0) }
}

/// Linear expression of variables
///
/// Represents an affine expression of variables: a constant term plus variables multiplied by coefficients.
///
/// A `LinExpr` object is typically created using [`Expr::into_linexpr`]. Most [`Model`] methods take
/// [`Expr`] as arguments instead of `LinExpr`, so converting to `LinExpr` is rarely needed.
#[derive(Debug, Clone, Default)]
pub struct LinExpr {
  coeff: FnvHashMap<Var, f64>,
  offset: f64,
}



/// Quadratic expression of variables
///
/// Represents an linear summation of quadratic terms, plus a linear expression.
///
/// A `QuadExpr` object is typically created using [`Expr::into_quadexpr`]. Most [`Model`] methods take
/// [`Expr`] as arguments instead of `QuadExpr`, so converting to `QuadExpr` is rarely needed.
#[derive(Debug, Clone, Default)]
pub struct QuadExpr {
  linexpr : LinExpr,
  qcoeffs: FnvHashMap<(Var,Var), f64>
}


impl From<Var> for Expr {
  fn from(var: Var) -> Expr { Expr::Term(1.0, var) }
}



macro_rules! impl_all_primitives {
    ($macr:ident; $($args:tt),*) => {
      $macr!{f64 $(,$args)*}
      $macr!{f32 $(,$args)*}
      $macr!{u8 $(,$args)*}
      $macr!{u16 $(,$args)*}
      $macr!{u32 $(,$args)*}
      $macr!{u64 $(,$args)*}
      $macr!{usize $(,$args)*}
      $macr!{i8 $(,$args)*}
      $macr!{i16 $(,$args)*}
      $macr!{i32 $(,$args)*}
      $macr!{i64 $(,$args)*}
      $macr!{isize $(,$args)*}
    };
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

  /// Set the constant offset,  returning the old one
  pub fn set_offset(&mut self, val: f64) -> f64 {
    std::mem::replace(&mut self.offset, val)
  }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let coeff = self.coeff.values();
    let vars : Vec<_> = self.coeff.keys().cloned().collect();
    let vals = model.get_obj_attr_batch(attr::X, &vars)?;
    let total = coeff.zip(vals.into_iter()).map(|(&a, x)| a*x).sum::<f64>() + self.offset;
    Ok(total)
  }

  /// Decompose into variables, their coefficients and the offset, respectively.
  pub fn into_parts(self) -> (FnvHashMap<Var, f64>, f64) { (self.coeff, self.offset) }

  /// number of linear terms in the expression (excluding the constant)
  pub fn n_terms(&self) -> usize { self.coeff.len() }

  /// Returns an iterator over the terms excluding the offset (item type is `(&Var, &f64)`)
  pub fn iter_terms(&self) -> std::collections::hash_map::Iter<Var, f64> {
    self.coeff.iter()
  }

  /// Multiply expression by a scalar
  pub fn mul_scalar(&mut self, val: f64) -> &mut Self {
    self.offset *= val;
    self.coeff.iter_mut().for_each(|(_, a)| *a *= val);
    self
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
  pub fn into_parts(self) -> (FnvHashMap<(Var, Var), f64>, LinExpr) {
    (self.qcoeffs, self.linexpr)
  }
  /// Add a linear term into the expression.
  pub fn add_term(&mut self, coeff: f64, var: Var) -> &mut Self {
    self.linexpr.add_term(coeff, var);
    self
  }

  /// Add a quadratic term into the expression.
  pub fn add_qterm(&mut self, coeff: f64, rowvar: Var, colvar: Var) -> &mut Self {
    if rowvar.id > colvar.id { // we don't bother checking the model_id here, it gets check when this object is passed to the model
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

  /// Set the constant offset,  returning the old one
  pub fn set_offset(&mut self, val: f64) -> f64 {
    self.linexpr.set_offset(val)
  }

  /// Get actual value of the expression.
  pub fn get_value(&self, model: &Model) -> Result<f64> {
    let coeff = self.qcoeffs.values();
    let mut rowvars = Vec::with_capacity(self.qcoeffs.len());
    let mut colvars = Vec::with_capacity(self.qcoeffs.len());
    for (x,y) in self.qcoeffs.keys().cloned() {
      rowvars.push(x);
      colvars.push(y);
    }
    let rowvals = model.get_obj_attr_batch(attr::X, &rowvars)?;
    let colvals = model.get_obj_attr_batch(attr::X, &colvars)?;
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

  /// number of linear terms in the expression (excluding the constant)
  pub fn n_terms(&self) -> usize { self.linexpr.n_terms() }

  /// Returns an iterator over the terms excluding the offset (item type is `(&Var, &f64)`)
  pub fn iter_terms(&self) -> std::collections::hash_map::Iter<Var, f64> {
    self.linexpr.iter_terms()
  }

  /// number of quadtratic terms in the expression
  pub fn n_qterms(&self) -> usize { self.qcoeffs.len() }

  /// Returns an iterator over the terms excluding the offset (item type is `(&Var, &f64)`)
  pub fn iter_qterms(&self) -> std::collections::hash_map::Iter<(Var, Var), f64> { self.qcoeffs.iter() }

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
        let (qcoeffs, linexpr) = e2.into_parts();
        for ((x,y),a) in qcoeffs {
          e1.add_qterm(a,x,y);
        }
        e1.linexpr = (Linear(e1.linexpr) + Linear(linexpr)).into_linexpr().unwrap();
        e1.into()
      }
      // swap operands
      (lhs, rhs) => { rhs + lhs }
    }
  }
}


macro_rules! impl_from_prim_for_expr {
    ($t:ty) => {
      impl From<$t> for Expr {
        fn from(val: $t) -> Expr { Expr::Constant(val as f64) }
      }
    };
}

impl_all_primitives!(impl_from_prim_for_expr; );

impl From<LinExpr> for Expr {
  fn from(val: LinExpr) -> Expr { Expr::Linear(val) }
}

impl From<QuadExpr> for Expr {
  fn from(val: QuadExpr) -> Expr { Expr::Quad(val) }
}

impl<T: Copy + Into<Expr>> From<&T> for Expr {
  fn from(val: &T) -> Expr { (*val).into() }
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


macro_rules! impl_mul_t_expr {
  ($p:ty, $($t:ty),+) => {
    impl Mul<$p> for Expr {
      type Output = Expr;
      fn mul(self, rhs: $p) -> Expr {
        use self::Expr::*;
        let rhs = rhs as f64;
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

    impl Mul<Expr> for $p {
      type Output = Expr;
      fn mul(self, rhs: Expr) -> Expr { rhs*self }
    }

    $(
      impl Mul<$t> for $p {
        type Output = Expr;
        fn mul(self, rhs: $t) -> Expr { self * <$t as Into<Expr>>::into(rhs) }
      }

      impl Mul<$p> for $t {
        type Output = Expr;
        fn mul(self, rhs: $p) -> Expr { rhs*self }
      }

    )+
  };
}

impl_all_primitives!(impl_mul_t_expr; Var, LinExpr, QuadExpr );

macro_rules! impl_add_nonprim_expr {
  ($($t:ty),+) => {
    $(
      impl Add<$t> for Expr {
        type Output = Expr;
        fn add(self, rhs: $t) -> Expr { self + Expr::from(rhs) }
      }


      impl Add<Expr> for $t {
        type Output = Expr;
        fn add(self, rhs: Expr) -> Expr { rhs + self }
      }

    )+
  }
}


macro_rules! impl_add_prim_t {
  ($p:ty, $($t:ty),+) => {
    $(
      impl Add<$p> for $t {
        type Output = Expr;
        fn add(self, rhs: $p) -> Expr { Expr::from(self) + Expr::from(rhs) }
      }

      impl Add<$t> for $p {
        type Output = Expr;
        fn add(self, rhs: $t) -> Expr { Expr::from(rhs) + Expr::from(self) }
      }
    )+
  }
}

impl_add_nonprim_expr!(Var, LinExpr, QuadExpr );
impl_all_primitives!(impl_add_prim_t; Expr, Var, LinExpr, QuadExpr );

macro_rules! impl_sub_nonprim_expr {
    ($($t:ty),+) => {
    $(
      impl Sub<$t> for Expr {
        type Output = Expr;
        fn sub(self, rhs : $t) -> Expr { self + (-Expr::from(rhs))}
      }

      impl Sub<Expr> for $t {
        type Output = Expr;
        fn sub(self, rhs: Expr) -> Expr { Expr::from(self) + (-rhs) }
      }
    )+
  };
}

macro_rules! impl_sub_prim_t {
  ($p:ty, $($t:ty),+) => {
    $(
      impl Sub<$p> for $t {
        type Output = Expr;
        fn sub(self, rhs: $p) -> Expr { Expr::from(self) + -Expr::from(rhs) }
      }

      impl Sub<$t> for $p {
        type Output = Expr;
        fn sub(self, rhs: $t) -> Expr { Expr::from(self) + -Expr::from(rhs)  }
      }
    )+
  }
}

impl_sub_nonprim_expr!(Var, LinExpr, QuadExpr );
impl_all_primitives!(impl_sub_prim_t; Expr, Var, LinExpr, QuadExpr);

impl Neg for Var {
  type Output = Expr;
  fn neg(self) -> Expr { Expr::Term(-1.0, self) }
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
    let mut total = iter.next().map_or(Expr::Constant(0.0), |x| x.into());
    for x in iter {
      total = total + x.into();
    }
    total
  }
}


/// Convenience trait for summing over iterators to produce a concrete type.
/// Analogous to `collect_vec` from the `itertools` crate.
pub trait GurobiSum {
  fn grb_sum(self) -> Expr;
}

impl<T,I> GurobiSum for I where
    T: Into<Expr>,
    I: Iterator<Item=T>
{
  fn grb_sum(self) -> Expr { self.sum() }
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
  if (x-ignore_val).abs() < f64::EPSILON {
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
      let varname = self.model.get_obj_attr(attr::VarName, var)?;
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
      let xname = self.model.get_obj_attr(attr::VarName, x)?;
      let yname = self.model.get_obj_attr(attr::VarName, y)?;
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
        let varname = self.model.get_obj_attr(attr::VarName, x)?;
        if (a-1.0).abs() < f64::EPSILON {
          f.write_fmt(format_args!("{}", varname))
        } else {
          f.write_fmt(format_args!("{} {}", a, varname))
        }
      },
      QTerm(a,x, y) => {
        let xname = self.model.get_obj_attr(attr::VarName, x)?;
        let yname = self.model.get_obj_attr(attr::VarName, y)?;
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
  use crate::{Binary, Model};

  macro_rules! make_model_with_vars {
    ($model:ident, $($var:ident),+) => {
      let mut $model = Model::new("test").unwrap();
      $(
        let $var = $model.add_var(stringify!($var), Binary, 0.0, 0.0, 0.0, &[], &[]).unwrap();
      )+
      $model.update().unwrap(); // necessary to retrieve variable attributes
    }
  }

  #[test]
  fn simple() {
    make_model_with_vars!(model, x, y);
    let e : Expr = x * y + 1 + x + 2.0*y;
    e.into_linexpr().unwrap_err(); // should be quadratic
  }

  #[test]
  fn nested() {
    make_model_with_vars!(model, x, y);
    let e = (x * y)*3 + 2*(x + 2.0*y);
  }

  #[test]
  fn multiplication_commutes() {
    make_model_with_vars!(model, x, y, z);
    let _ = x - y;
    let e = y*x - x*y;
    dbg!(e.attach(&model));
    let mut e = e.into_quadexpr();
    assert!(!e.is_empty());
    e.sparsify();
    assert!(e.is_empty());
  }


  #[test]
  fn multiplication() {
    make_model_with_vars!(model, x, y);
    let e = 2*x;
    let e = x*x;
    let e = 2*(x*x);
  }

  #[test]
  fn addition() {
    make_model_with_vars!(model, x, y);
    let e = 2 + x;
    let e = x + y;
    let e = x + x;
    let e = x + 2.8*y + 2*x;
  }


  #[test]
  fn subtraction() {
    make_model_with_vars!(model, x, y);
    let e = 2 - x;
    let mut e = (x - x).into_linexpr().unwrap();
    e.sparsify();
    assert!(e.is_empty());
    let e = 2 * x - y - x;

    let e1 : Expr = 2*x + 1.0*y;
    let e2 : Expr = 4 - 3*y;
    let e : LinExpr = (e1 - e2).into_linexpr().unwrap();
    assert!((e.get_offset() - -4.0).abs() < f64::EPSILON);

    for (&var, &coeff) in e.iter_terms() {
      if var == x { assert!((coeff - 2.0) < f64::EPSILON) }
      if var == x { assert!((coeff - 4.0) < f64::EPSILON) }
    }


  }

  #[test]
  fn negate() {
    make_model_with_vars!(model, x);
    let q = -x;
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

    let vars = [
      2*x,
      -y,
      -z,
      0.2*x
    ];
    let e : Expr = vars.iter().cloned().sum();
    let e = e.into_linexpr().unwrap();
    assert_eq!(e.coeff.len(), 3);
  }

  #[test]
  fn linexpr_debug_fmt() {
    make_model_with_vars!(m, x, y);
    let e = 2usize * y;
    let s = format!("{:?}", e.attach(&m));
    assert_eq!("2 y", s.to_string());
    eprintln!("{}", s);
    let e = x*y - 2.0f64 *(x*x);
    eprintln!("{:?}", e.attach(&m));
  }
}
