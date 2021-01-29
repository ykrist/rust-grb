// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use gurobi::*;

fn main() {
  let env = gurobi::Env::new("qcp.log").unwrap();

  // create an empty model.
  let mut model = Model::with_env("qcp", &env).unwrap();

  // add & integrate new variables.
  let x = model.add_var("x", Continuous, 0.0, 0.0, INFINITY, &[], &[]).unwrap();
  let y = model.add_var("y", Continuous, 0.0, 0.0, INFINITY, &[], &[]).unwrap();
  let z = model.add_var("z", Continuous, 0.0, 0.0, INFINITY, &[], &[]).unwrap();
  model.update().unwrap();

  // set objective funtion:
  //   f(x,y,z) = x
  model.set_objective(x, Maximize).unwrap();

  // add linear constraints

  //  c0: x + y + z == 1
  model.add_constr("c0", c!(x + y + z == 1)).unwrap();

  // add quadratic constraints
  model.add_qconstr("qc0", c!(x*x + y*y <= z*z)).unwrap();
  model.add_qconstr("qc1", c!(x*x <= y*z)).unwrap();

  // optimize the model.
  model.optimize().unwrap();

  // write the model to file.
  model.write("qcp.lp").unwrap();
  model.write("qcp.sol").unwrap();
}
