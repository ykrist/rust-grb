extern crate gurobi;
use gurobi::*;

fn main() {
  let env = gurobi::Env::new("qcp1.log").unwrap();

  // create an empty model.
  let mut model = env.new_model("qcp1").unwrap();

  // add & integrate new variables.
  let x = model.add_var("x", Continuous(0.0, 1e+100)).unwrap();
  let y = model.add_var("y", Continuous(0.0, 1e+100)).unwrap();
  let z = model.add_var("z", Continuous(0.0, 1e+100)).unwrap();
  model.update().unwrap();

  // set objective funtion:
  //   f(x,y,z) = x
  model.set_objective(&x, Maximize).unwrap();

  // add linear constraints

  //  c0: x + y + z == 1
  let c0 = model.add_constr("c0", &x + &y + &z, Equal, 1.0).unwrap();

  // add quadratic constraints

  //  qc0: x^2 + y^2 - z^2 <= 0.0
  let qc0 = model.add_qconstr("qc0", &x * &x + &y * &y - &z * &z, Less, 0.0).unwrap();

  //  qc1: x^2 - y*z <= 0.0
  let qc1 = model.add_qconstr("qc1", &x * &x - &y * &z, Less, 0.0).unwrap();

  // optimize the model.
  model.optimize().unwrap();

  // write the model to file.
  model.write("qcp.lp").unwrap();
  model.write("qcp.sol").unwrap();

  let status = model.get(attr::Status).unwrap();
  assert_eq!(status, 2);
}
