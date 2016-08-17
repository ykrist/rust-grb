extern crate gurobi;

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
  let expr = gurobi::QuadExpr::new(&[x, y, z], &[1.0, 0.0, 0.0], &[], &[], &[], 0.0).unwrap();
  model.set_objective(expr, gurobi::Maximize).unwrap();

  // add linear constraints

  //  c0: x + y + z == 1
  let c0 = gurobi::LinExpr::new(&[x, y, z], &[1., 1., 1.], 0.0).unwrap();
  let c0 = model.add_constr("c0", c0, gurobi::Equal, 1.0).unwrap();

  // add quadratic constraints

  //  qc0: x^2 + y^2 - z^2 <= 0.0
  let qc0 = gurobi::QuadExpr::new(&[], &[], &[x, y, z], &[x, y, z], &[1., 1., -1.0], 0.0).unwrap();
  let qc0 = model.add_qconstr("qc0", qc0, gurobi::Less, 0.0).unwrap();

  //  qc1: x^2 - y*z <= 0.0
  let qc1 = gurobi::QuadExpr::new(&[], &[], &[x, y], &[x, z], &[1., -1.0], 0.0).unwrap();
  let qc1 = model.add_qconstr("qc1", qc1, gurobi::Less, 0.0).unwrap();

  let _ = model.get(gurobi::attr::ModelSense).unwrap();
  let _ = model.get(gurobi::attr::ObjVal).unwrap();

  // optimize the model.
  model.optimize().unwrap();

  // write the model to file.
  model.write("qcp.lp").unwrap();
  model.write("qcp.sol").unwrap();

  let status = model.get(gurobi::attr::Status).unwrap();
  assert_eq!(status, 2);
}
