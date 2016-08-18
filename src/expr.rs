use std::ops::{Add, Sub, Mul};
use model::Var;


pub struct LinExpr {
  pub vars: Vec<i32>,
  pub coeff: Vec<f64>,
  pub offset: f64
}


impl LinExpr {
  pub fn new() -> LinExpr {
    LinExpr {
      vars: Vec::new(),
      coeff: Vec::new(),
      offset: 0.0
    }
  }

  pub fn term(mut self, v: Var, c: f64) -> LinExpr {
    self.vars.push(v.0);
    self.coeff.push(c);
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
  pub lind: Vec<i32>,
  pub lval: Vec<f64>,
  pub qrow: Vec<i32>,
  pub qcol: Vec<i32>,
  pub qval: Vec<f64>,
  pub offset: f64
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

  pub fn term(mut self, var: Var, coeff: f64) -> QuadExpr {
    self.lind.push(var.0);
    self.lval.push(coeff);
    self
  }

  pub fn qterm(mut self, row: Var, col: Var, coeff: f64) -> QuadExpr {
    self.qrow.push(row.0);
    self.qcol.push(col.0);
    self.qval.push(coeff);
    self
  }

  pub fn offset(mut self, offset: f64) -> QuadExpr {
    self.offset += offset;
    self
  }
}


impl Mul<f64> for Var {
  type Output = LinExpr;
  fn mul(self, rhs: f64) -> LinExpr { LinExpr::new().term(self, rhs) }
}

impl Mul<Var> for f64 {
  type Output = LinExpr;
  fn mul(self, rhs: Var) -> LinExpr { LinExpr::new().term(rhs, self) }
}


impl Mul<Var> for Var {
  type Output = QuadExpr;
  fn mul(self, rhs: Var) -> QuadExpr { QuadExpr::new().qterm(self, rhs, 1.0) }
}

impl Mul<f64> for QuadExpr {
  type Output = QuadExpr;
  fn mul(mut self, rhs: f64) -> QuadExpr {
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


impl Add<f64> for LinExpr {
  type Output = LinExpr;
  fn add(self, rhs: f64) -> LinExpr { self.offset(rhs) }
}

impl Sub<f64> for LinExpr {
  type Output = LinExpr;
  fn sub(self, rhs: f64) -> LinExpr { self.offset(-rhs) }
}


impl Add for LinExpr {
  type Output = LinExpr;
  fn add(mut self, rhs: LinExpr) -> LinExpr {
    self.vars.extend(rhs.vars);
    self.coeff.extend(rhs.coeff);
    self.offset += rhs.offset;
    self
  }
}

impl Sub for LinExpr {
  type Output = LinExpr;
  fn sub(mut self, rhs: LinExpr) -> LinExpr {
    self.vars.extend(rhs.vars);
    self.coeff.extend(rhs.coeff.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}


impl Add<LinExpr> for QuadExpr {
  type Output = QuadExpr;
  fn add(mut self, rhs: LinExpr) -> QuadExpr {
    self.lind.extend(rhs.vars);
    self.lval.extend(rhs.coeff);
    self.offset += rhs.offset;
    self
  }
}

impl Sub<LinExpr> for QuadExpr {
  type Output = QuadExpr;
  fn sub(mut self, rhs: LinExpr) -> QuadExpr {
    self.lind.extend(rhs.vars);
    self.lval.extend(rhs.coeff.into_iter().map(|c| -c));
    self.offset -= rhs.offset;
    self
  }
}


impl Add for QuadExpr {
  type Output = QuadExpr;
  fn add(mut self, rhs:QuadExpr) -> QuadExpr {
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
    self.lval.extend(rhs.lval);
    self.qrow.extend(rhs.qrow.into_iter().map(|c|-c));
    self.qcol.extend(rhs.qcol);
    self.qval.extend(rhs.qval.into_iter().map(|c|-c));
    self.offset -= rhs.offset;
    self
  }
}
