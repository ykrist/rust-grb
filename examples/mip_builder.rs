// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;

pub struct VarBuilder<'a> {
  model: &'a mut Model,
  name: String,
  vtype: VarType,
  obj: f64,
  lb: f64,
  ub: f64,
  constrs: Vec<Constr>,
  columns: Vec<f64>
}

impl<'a> VarBuilder<'a> {
  pub fn new(model: &'a mut Model) -> VarBuilder<'a> {
    VarBuilder {
      model: model,
      name: "".to_string(),
      vtype: Continuous,
      obj: 0.0,
      lb: 0.0,
      ub: INFINITY,
      constrs: Vec::new(),
      columns: Vec::new()
    }
  }

  pub fn name(mut self, name: &str) -> VarBuilder<'a> {
    self.name = name.to_owned();
    self
  }

  pub fn binary(mut self) -> VarBuilder<'a> {
    self.vtype = Binary;
    self.lb = 0.0;
    self.ub = 1.0;
    self
  }

  pub fn integer(mut self, lb: f64, ub: f64) -> VarBuilder<'a> {
    self.vtype = Integer;
    self.lb = lb;
    self.ub = ub;
    self
  }

  pub fn continuous(mut self, lb: f64, ub: f64) -> VarBuilder<'a> {
    self.vtype = Continuous;
    self.lb = lb;
    self.ub = ub;
    self
  }

  pub fn commit(self) -> Result<Var> {
    self.model.add_var(&self.name,
                       self.vtype,
                       self.obj,
                       self.lb,
                       self.ub,
                       self.constrs.as_slice(),
                       self.columns.as_slice())
  }
}

macro_rules! add_var {
  ($model:expr; name:$name:ident , binary)
    => ( VarBuilder::new(&mut $model).name(stringify!($name)).binary().commit().unwrap() );
  
  ($model:expr; name:$name:ident , integer [$lb:expr, $ub:expr])
    => ( VarBuilder::new(&mut $model).name(stringify!($name)).integer(($lb).into(), ($ub).into()).commit().unwrap() );

  ($model:expr; name:$name:ident , continuous [$lb:expr, $ub:expr])
    => ( VarBuilder::new(&mut $model).name(stringify!($name)).continuous(($lb).into(), ($ub).into()).commit().unwrap() );
}


fn main() {
  let env = Env::new("mip.log").unwrap();
  let mut model = Model::new("mip", &env).unwrap();

  let x = add_var!(model; name:x, binary);
  let y = add_var!(model; name:y, binary);
  let z = add_var!(model; name:z, binary);
  let s = add_var!(model; name:s, integer[0, 2]);
  let t = add_var!(model; name:t, continuous[0, 10]);
  let u = VarBuilder::new(&mut model).name("u").continuous(-10.0, 10.0).commit().unwrap();
  model.update().unwrap();

  model.set_objective(&x + &y + 2.0 * &z, Maximize).unwrap();
  model.add_constr("c0", &x + 2.0 * &y + 3.0 * &z, Less, 4.0).unwrap();
  model.add_constr("c1", &x + &y, Greater, 1.0).unwrap();

  model.update().unwrap();
  model.write("mip.lp").unwrap();
}
