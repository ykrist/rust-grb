use grb::*;

fn main() {
  let env = Env::new("sos.log").unwrap();
  let mut model = Model::with_env("sos", &env).unwrap();

  let x0 = model.add_var("x0", Continuous, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  let x1 = model.add_var("x1", Continuous, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  let x2 = model.add_var("x2", Continuous, 0.0, 0.0, 2.0, &[], &[]).unwrap();
  model.update().unwrap();

  model.set_objective(2*x0 + x1 + x2, Maximize).unwrap();

  // [x0 = 0] or [x1 = 0]
  model.add_sos(&[x0, x1], &[1.0, 2.0], SOSType1).unwrap();

  // [x0 = 0] or [x2 = 0]
  model.add_sos(&[x0, x2], &[1.0, 2.0], SOSType1).unwrap();

  model.optimize().unwrap();

  model.write("sos.lp").unwrap();
  model.write("sos.sol").unwrap();

  let obj = model.get_attr(attr::ObjVal).unwrap();
  assert_eq!(obj.round() as isize, 3);
  let get_value = |var| model.get_obj_attr(attr::X, &var).unwrap();
  let x0 = get_value(x0);
  let x1 = get_value(x1);
  let x2 = get_value(x2);

  assert_eq!(x0.round() as isize, 0);
  assert_eq!(x1.round() as isize, 1);
  assert_eq!(x2.round() as isize, 2);
}
