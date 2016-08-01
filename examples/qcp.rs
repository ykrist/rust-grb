extern crate gurobi;
use gurobi::HasAttr;

fn main() {
  let env = gurobi::Env::new("qcp1.log").unwrap();

  // create an empty model.
  let mut model = env.new_model("qcp1").unwrap();

  // add & integrate new variables.
  let x = model.add_var("x", gurobi::Continuous(0.0, 1e+100), 0.0).unwrap();
  let y = model.add_var("y", gurobi::Continuous(0.0, 1e+100), 0.0).unwrap();
  let z = model.add_var("z", gurobi::Continuous(0.0, 1e+100), 0.0).unwrap();
  model.update().unwrap();

  // set objective funtion:
  //   f(x,y,z) = x
  model.set_objective(&[0, 1, 2], &[1.0, 0.0, 0.0], &[], &[], &[], gurobi::Maximize).unwrap();

  // add linear constraints

  //  c0: x + y + z == 1
  let c0 = model.add_constr("c0", &[0, 1, 2], &[1., 1., 1.], gurobi::Equal, 1.0)
    .unwrap();

  // add quadratic constraints

  //  qc0: x^2 + y^2 - z^2 <= 0.0
  let qc0 = model.add_qconstr("qc0",
                 &[],
                 &[],
                 &[0, 1, 2],
                 &[0, 1, 2],
                 &[1., 1., -1.0],
                 gurobi::Less,
                 0.0)
    .unwrap();

  //  qc1: x^2 - y*z <= 0.0
  let qc1 = model.add_qconstr("qc1",
                 &[],
                 &[],
                 &[0, 1],
                 &[0, 2],
                 &[1., -1.0],
                 gurobi::Less,
                 0.0)
    .unwrap();

  let _ = model.get(gurobi::IntAttr::ModelSense).unwrap();
  let _ = model.get(gurobi::DoubleAttr::ObjVal).unwrap();

  // optimize the model.
  model.optimize().unwrap();

  // write the model to file.
  model.write("qcp.lp").unwrap();
  model.write("qcp.sol").unwrap();

  let status = model.get(gurobi::IntAttr::Status).unwrap();
  assert_eq!(status, 2);
}
