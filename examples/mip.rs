use grb::*;

fn main() {
  let env = Env::new("mip.log").unwrap();
  let mut model = Model::with_env("mip", &env).unwrap();

  let x = add_binvar!(model, name: "x").unwrap();
  let y = add_binvar!(model, name: "y").unwrap();
  let z = add_binvar!(model, name: "z").unwrap();
  model.update().unwrap();

  model.set_objective(x + y + 2 * z, Minimize).unwrap();

  model.add_constr("c0", c!(x + 2 * y + 3 * z <= 4)).unwrap();
  model.add_constr("c1", c!(x + y >= 1)).unwrap();
  model.add_range("range", c!(x + 2.6*y in 1..10)).unwrap();

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
