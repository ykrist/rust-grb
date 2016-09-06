// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;

macro_rules! def_var {
  ($model:expr, $name:ident : binary) => {
    let $name = $model.add_var(stringify!($name), Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    $model.update().unwrap();
  };
  
  ($model:expr, $name:ident : integer [$lb:expr, $ub:expr]) => {
    let $name = $model.add_var(stringify!($name), Integer, 0.0, ($lb).into(), ($ub).into(), &[], &[]).unwrap();
    $model.update().unwrap();
  };

  ($model:expr, $name:ident : real [$lb:expr, $ub:expr]) => {
    let $name = $model.add_var(stringify!($name), Continuous, 0.0, ($lb).into(), ($ub).into(), &[], &[]).unwrap();
    $model.update().unwrap();
  };

  ($model:expr, $name:ident : binary; $($t:tt)*) => {
    let $name = $model.add_var(stringify!($name), Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    def_var!($model, $($t)*);
    $model.update().unwrap();
  };
  
  ($model:expr, $name:ident : integer [$lb:expr, $ub:expr]; $($t:tt)*) => {
    let $name = $model.add_var(stringify!($name), Integer, 0.0, ($lb).into(), ($ub).into(), &[], &[]).unwrap();
    def_var!($model, $($t)*);
    $model.update().unwrap();
  };

  ($model:expr, $name:ident : real [$lb:expr, $ub:expr]; $($t:tt)*) => {
    let $name = $model.add_var(stringify!($name), Continuous, 0.0, ($lb).into(), ($ub).into(), &[], &[]).unwrap();
    def_var!($model, $($t)*);
    $model.update().unwrap();
  };
}

macro_rules! add_constr {
  { $model:expr, $name:ident : $lhs:tt <= $rhs:tt; } => {
    $model.add_constr(stringify!($name), $lhs, Less, ($rhs).into()).unwrap();
    $model.update().unwrap();
  };

  { $model:expr, $name:ident : $lhs:tt == $rhs:tt; } => {
    $model.add_constr(stringify!($name), $lhs, Equal, ($rhs).into()).unwrap();
    $model.update().unwrap();
  };

  { $model:expr, $name:ident : $lhs:tt >= $rhs:tt; } => {
    $model.add_constr(stringify!($name), $lhs, Greater, ($rhs).into()).unwrap();
    $model.update().unwrap();
  };

  ($model:expr, $name:ident : $lhs:tt <= $rhs:tt; $($s:tt)*) => {
    $model.add_constr(stringify!($name), $lhs, Less, ($rhs).into()).unwrap();
    add_constr!($model, $($s)*);
    $model.update().unwrap();
  };

  ($model:expr, $name:ident : $lhs:tt == $rhs:tt; $($s:tt)*) => {
    $model.add_constr(stringify!($name), $lhs, Equal, ($rhs).into()).unwrap();
    add_constr!($model, $($s)*);
    $model.update().unwrap();
  };

  ($model:expr, $name:ident : $lhs:tt >= $rhs:tt; $($s:tt)*) => {
    $model.add_constr(stringify!($name), $lhs, Greater, ($rhs).into()).unwrap();
    add_constr!($model, $($s)*);
    $model.update().unwrap();
  };
}

fn main() {
  let env = Env::new("mip.log").unwrap();
  let mut model = Model::new("mip", &env).unwrap();

  def_var!(model, x:binary; y:binary; z:binary; s:integer[0,2]; t:real[0,10]);

  model.set_objective(&x + &y + 2.0 * &z, Maximize).unwrap();

  add_constr! { model,
    c0: (&x + 2.0 * &y + 3.0 * &z) <= 4;
    c1: (&x + &y) <= 1;
  }

  model.update().unwrap();
  model.write("mip.lp").unwrap();
}
