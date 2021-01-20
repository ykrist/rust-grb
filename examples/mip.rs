// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;

fn main() {
  let env = Env::new("mip.log").unwrap();
  let mut model = Model::new("mip", &env).unwrap();

  let x = model.add_var("x", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  let y = model.add_var("y", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  let z = model.add_var("z", Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  model.update().unwrap();

  model.set_objective(&x + &y + 2.0 * &z, Minimize).unwrap();

  model.add_constr("c0", &x + 2.0 * &y + 3.0 * &z, Less, 4.0).unwrap();

  model.add_constr("c1", &x + &y, Greater, 1.0).unwrap();

  model.optimize().unwrap();

  let status = model.get_attr(attr::Status).unwrap();
  assert_eq!(status, 2);

  let objval = model.get_attr(attr::ObjVal).unwrap();
  assert_eq!(objval.round() as isize, 1);
  let numvars = model.get_attr(attr::NumVars).unwrap() as usize;
  assert_eq!(numvars, 3);

  let get_value = | var | model.get_obj_attr(attr::X, var).unwrap();
  assert_eq!(get_value(&x).round() as isize, 0);
  assert_eq!(get_value(&y).round() as isize, 1);
  assert_eq!(get_value(&z).round() as isize, 0);

  model.write("mip.lp").unwrap();
  model.write("mip.sol").unwrap();
}
