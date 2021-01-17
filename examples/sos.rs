// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;

fn main() {
  let env = Env::new("sos.log").unwrap();
  let mut model = Model::new("sos", &env).unwrap();

  let x0 = model.add_var("x0", Continuous, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  let x1 = model.add_var("x1", Continuous, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  let x2 = model.add_var("x2", Continuous, 0.0, 0.0, 2.0, &[], &[]).unwrap();
  model.update().unwrap();

  model.set_objective(2.0 * &x0 + 1.0 * &x1 + 1.0 * &x2, Maximize).unwrap();

  // [x0 = 0] or [x1 = 0]
  model.add_sos(&[x0.clone(), x1.clone()], &[1.0, 2.0], SOSType1).unwrap();

  // [x0 = 0] or [x2 = 0]
  model.add_sos(&[x0.clone(), x2.clone()], &[1.0, 2.0], SOSType1).unwrap();

  model.optimize().unwrap();

  model.write("sos.lp").unwrap();
  model.write("sos.sol").unwrap();

  let obj = model.get_attr(attr::ObjVal).unwrap();
  assert_eq!(obj.round() as isize, 3);
  let x0 = x0.get(&model, attr::X).unwrap();
  let x1 = x1.get(&model, attr::X).unwrap();
  let x2 = x2.get(&model, attr::X).unwrap();

  assert_eq!(x0.round() as isize, 0);
  assert_eq!(x1.round() as isize, 1);
  assert_eq!(x2.round() as isize, 2);
}
