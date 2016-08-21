extern crate gurobi;
use gurobi::*;

fn main() {
  let env = Env::new("mip.log").unwrap();
  let mut model = env.new_model("mip").unwrap();

  let x = model.add_var("x", Binary).unwrap();
  let y = model.add_var("y", Binary).unwrap();
  let z = model.add_var("z", Binary).unwrap();
  model.update().unwrap();

  model.set_objective(x + y + 2.0 * z, Maximize).unwrap();

  let _ = model.add_constr("c0", x + 2.0 * y + 3.0 * z, Less, 4.0).unwrap();

  let _ = model.add_constr("c1", x + y, Greater, 1.0).unwrap();

  model.optimize().unwrap();

  let status = model.get(attr::Status).unwrap();
  assert_eq!(status, 2);

  let objval = model.get(attr::ObjVal).unwrap();
  assert!((objval - 1.0).abs() < 1e-12);

  let numvars = model.get(attr::NumVars).unwrap() as usize;
  assert_eq!(numvars, 3);

  let x = x.get(&model, attr::X).unwrap();
  assert_eq!(x, 0.0);

  let y = y.get(&model, attr::X).unwrap();
  assert_eq!(y, 1.0);

  let z = z.get(&model, attr::X).unwrap();
  assert_eq!(z, 0.0);

  model.write("mip.lp").unwrap();
  model.write("mip.sol").unwrap();
}
