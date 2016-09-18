// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;

macro_rules! def_var {
  ($model:expr) => ();
  ($model:expr,) => ();

  ($model:expr, $name:ident : binary, $($t:tt)*) => {
    let $name = $model.add_var(stringify!($name), Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
    def_var!($model, $($t)*);
  };
  
  ($model:expr, $name:ident : integer [$lb:expr, $ub:expr], $($t:tt)*) => {
    let $name = $model.add_var(stringify!($name), Integer, 0.0, ($lb).into(), ($ub).into(), &[], &[]).unwrap();
    def_var!($model, $($t)*);
  };

  ($model:expr, $name:ident : real [$lb:expr, $ub:expr], $($t:tt)*) => {
    let $name = $model.add_var(stringify!($name), Continuous, 0.0, ($lb).into(), ($ub).into(), &[], &[]).unwrap();
    def_var!($model, $($t)*);
  };
}

macro_rules! add_constr {
  ($model:expr) => ();
  ($model:expr,) => ();

  ($model:expr, $name:ident : $lhs:tt <= $rhs:tt , $($s:tt)*) => {
    $model.add_constr(stringify!($name), ($lhs), Less, ($rhs).into()).unwrap();
    add_constr!($model, $($s)*);
  };

  ($model:expr, $name:ident : $lhs:tt == $rhs:tt , $($s:tt)*) => {
    $model.add_constr(stringify!($name), ($lhs), Equal, ($rhs).into()).unwrap();
    add_constr!($model, $($s)*);
  };

  ($model:expr, $name:ident : $lhs:tt >= $rhs:tt , $($s:tt)*) => {
    $model.add_constr(stringify!($name), ($lhs), Greater, ($rhs).into()).unwrap();
    add_constr!($model, $($s)*);
  };
}

macro_rules! set_objective {
  ($model:expr, expr: ($($t:tt)*), sense: $sense:ident) => {
    $model.set_objective($($t)*, $sense).unwrap();
  }
}

macro_rules! def_model {
  { env: $env:ident,  name: $name:expr,  vars: { $($t:tt)* },  objective: { $($o:tt)* },  constrs: { $($c:tt)* } } => { {
    let mut model = Model::new($name, &$env).unwrap();

    def_var!(model, $($t)*);
    model.update().unwrap();

    add_constr!(model, $($c)*);
    model.update().unwrap();

    set_objective!(model, $($o)*);
    model.update().unwrap();

    model
  } }
}

fn main() {
  let env = Env::new("mip_build.log").unwrap();

  let model = def_model! {
    env: env,
    name: "mip_build",
    vars: {
      x: binary,
      y: binary,
      z: binary,
      s: integer[0, 2],
      t: real[0, 10],
    },
    objective: {
      expr: (&x + &y + 2.0 * &z),
      sense: Maximize
    },
    constrs: {
      c0: (&x + 2.0 * &y + 3.0 * &z) <= 4,
      c1: (&x + &y) <= 1,
      c2: (1.0 * &s) == 0,
      c3: (1.0 * &t) == 0,
    }
  };

  model.write("mip.lp").unwrap();
}
