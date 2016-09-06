// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;

macro_rules! add_var {
  ($model:expr, $name:ident : binary) => { {
    let $name = $model.add_var(stringify!($name), Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    $name
  } };
  
  ($model:expr, $name:ident : integer [$lb:expr, $ub:expr]) => { {
    let $name = $model.add_var(stringify!($name), Integer, 0.0, ($lb).into(), ($ub).into(), &[], &[]).unwrap();
    $name
  } };

  ($model:expr, $name:ident : real [$lb:expr, $ub:expr]) => { {
    let $name = $model.add_var(stringify!($name), Continuous, 0.0, ($lb).into(), ($ub).into(), &[], &[]).unwrap();
    $name
  } };
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

  let x = add_var!(model, x: binary);
  let y = add_var!(model, y: binary);
  let z = add_var!(model, z: binary);
  let s = add_var!(model, s: integer[0, 2]);
  let t = add_var!(model, t: real[0, 10]);
  model.update().unwrap();

  add_constr!{model,
    c0: (&x + 2.0 * &y + 3.0 * &z) <= 4;
    c1: (&x + &y) <= 1;
  }

  model.set_objective(&x + &y + 2.0 * &z, Maximize).unwrap();

  model.update().unwrap();
  model.write("mip.lp").unwrap();
}
