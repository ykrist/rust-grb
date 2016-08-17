extern crate gurobi;
use gurobi::*;

fn main() {
  let env = Env::new("mip.log").unwrap();
  let mut model = env.new_model("mip").unwrap();

  let x = model.add_var("x", Binary, 1.0).unwrap();
  let y = model.add_var("y", Binary, 1.0).unwrap();
  let z = model.add_var("z", Binary, 2.0).unwrap();
  model.update().unwrap();

  model.set_objective(1.0 * x + 1.0 * y + 2.0 * z, Maximize).unwrap();

  let _ = model.add_constr("c0", 1.0 * x + 2.0 * y + 3.0 * z, Less, 4.0).unwrap();

  let _ = model.add_constr("c1", 1.0 * x + 1.0 * y, Greater, 1.0).unwrap();

  model.optimize().unwrap();

  let status = model.get(attr::Status).unwrap();
  assert_eq!(status, 2);

  let objval = model.get(attr::ObjVal).unwrap();
  assert!((objval - 1.0).abs() < 1e-12);

  let numvars = model.get(attr::NumVars).unwrap() as usize;
  assert_eq!(numvars, 3);

  let xval = model.get_array(attr::X, 0, numvars).unwrap();
  assert_eq!(xval[0], 0.0);
  assert_eq!(xval[1], 1.0);
  assert_eq!(xval[2], 0.0);

  model.write("mip.lp").unwrap();
  model.write("mip.sol").unwrap();
}
