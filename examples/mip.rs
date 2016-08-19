extern crate gurobi;
use gurobi::*;
use gurobi::core::Tensor;

fn main() {
  let env = Env::new("mip.log").unwrap();
  let mut model = env.new_model("mip").unwrap();

  let x = model.add_vars("x", Binary, ()).unwrap();
  let y = model.add_vars("y", Binary, ()).unwrap();
  let z = model.add_vars("z", Binary, ()).unwrap();
  model.update().unwrap();

  model.set_objective(1.0 * x.clone() + 1.0 * y.clone() + 2.0 * z.clone(),
                   Maximize)
    .unwrap();

  let _ = model.add_constrs("c0",
                 1.0 * x.clone() + 2.0 * y.clone() + 3.0 * z.clone(),
                 Less,
                 4.0)
    .unwrap();

  let _ = model.add_constrs("c1", 1.0 * x.clone() + 1.0 * y.clone(), Greater, 1.0).unwrap();

  model.optimize().unwrap();

  let status = model.get(attr::Status).unwrap();
  assert_eq!(status, 2);

  let objval = model.get(attr::ObjVal).unwrap();
  assert!((objval - 1.0).abs() < 1e-12);

  let numvars = model.get(attr::NumVars).unwrap() as usize;
  assert_eq!(numvars, 3);

  let x = x.get(&model, attr::X).unwrap();
  assert_eq!(x.at(()).cloned().unwrap(), 0.0);

  let y = y.get(&model, attr::X).unwrap();
  assert_eq!(y.at(()).cloned().unwrap(), 1.0);

  let z = z.get(&model, attr::X).unwrap();
  assert_eq!(z.at(()).cloned().unwrap(), 0.0);

  model.write("mip.lp").unwrap();
  model.write("mip.sol").unwrap();
}
