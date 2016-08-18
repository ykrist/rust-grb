extern crate gurobi;
use gurobi::*;

fn main() {
  let env = gurobi::Env::new("qcp1.log").unwrap();

  // create an empty model.
  let mut model = env.new_model("qcp1").unwrap();

  // add & integrate new variables.
  let x = model.add_vars("x", Continuous(0.0, 1e+100), ()).unwrap();
  let y = model.add_vars("y", Continuous(0.0, 1e+100), ()).unwrap();
  let z = model.add_vars("z", Continuous(0.0, 1e+100), ()).unwrap();
  model.update().unwrap();

  // set objective funtion:
  //   f(x,y,z) = x
  model.set_objective(1.0 * x.clone(), Maximize).unwrap();

  // add linear constraints

  //  c0: x + y + z == 1
  let _ = model.add_constrs("c0", 1.0 * x.clone() + 1.0 * y.clone() + 1.0 * z.clone(), Equal, 1.0).unwrap();

  // add quadratic constraints

  //  qc0: x^2 + y^2 - z^2 <= 0.0
  let _ = model.add_qconstrs("qc0", x.clone() * x.clone() + y.clone() * y.clone() - z.clone() * z.clone(), Less, 0.0).unwrap();

  //  qc1: x^2 - y*z <= 0.0
  let _ = model.add_qconstrs("qc1", x.clone() * x.clone() - y.clone() * z.clone(), Less, 0.0).unwrap();

  let _ = model.get(attr::ModelSense).unwrap();
  let _ = model.get(attr::ObjVal).unwrap();

  // optimize the model.
  model.optimize().unwrap();

  // write the model to file.
  model.write("qcp.lp").unwrap();
  model.write("qcp.sol").unwrap();

  let status = model.get(attr::Status).unwrap();
  assert_eq!(status, 2);
}
