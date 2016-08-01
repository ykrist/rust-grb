extern crate gurobi;
use gurobi::HasAttr;

fn main() {
  let env = gurobi::Env::new("mip1.log").unwrap();
  let mut model = env.new_model("mip1").unwrap();

  let x = model.add_var("x", gurobi::Binary, 1.0).unwrap();
  let y = model.add_var("y", gurobi::Binary, 1.0).unwrap();
  let z = model.add_var("z", gurobi::Binary, 2.0).unwrap();
  model.update().unwrap();

  model.set_objective(&[0, 1, 2],
                   &[1.0, 1.0, 2.0],
                   &[],
                   &[],
                   &[],
                   gurobi::Maximize)
    .unwrap();

  let c0 = model.add_constr("c0", &[0, 1, 2], &[1.0, 2.0, 3.0], gurobi::Less, 4.0)
    .unwrap();
  let c1 = model.add_constr("c1", &[0, 1], &[1.0, 1.0], gurobi::Greater, 1.0)
    .unwrap();

  model.optimize().unwrap();

  // fixes the model
  let model = model;

  let status = model.get(gurobi::attr::Status).unwrap();
  assert_eq!(status, 2);

  let objval = model.get(gurobi::attr::ObjVal).unwrap();
  assert!((objval - 1.0).abs() < 1e-12);

  let numvars = model.get(gurobi::attr::NumVars).unwrap() as usize;
  assert_eq!(numvars, 3);

  let xval = model.get_array(gurobi::attr::X, 0, numvars).unwrap();
  assert_eq!(xval[0], 0.0);
  assert_eq!(xval[1], 1.0);
  assert_eq!(xval[2], 0.0);

  model.write("mip1.lp").unwrap();
  model.write("mip1.sol").unwrap();
}
