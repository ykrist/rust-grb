extern crate gurobi;

fn main() {
  // the name of log file must not be 'mip1.log' (why?)
  let logfilename = "mip_1.log";
  let env = gurobi::Env::new(logfilename).unwrap();
  assert_eq!(env.get_str_param(gurobi::StringParam::LogFile).unwrap(), logfilename);

  let mut model = env.new_model("mip1", gurobi::Maximize).unwrap();

  model.add_var("x", gurobi::Binary, 1.0).unwrap();
  model.add_var("y", gurobi::Binary, 1.0).unwrap();
  model.add_var("z", gurobi::Binary, 2.0).unwrap();
  model.update().unwrap();

  model.add_constr("c0", &[0, 1, 2], &[1., 2., 3.], gurobi::Less, 4.0)
    .unwrap();
  model.add_constr("c1", &[0, 1], &[1., 1.], gurobi::Greater, 1.0).unwrap();

  model.optimize().unwrap();

  // fixes the model
  let model = model;

  let status = model.get_int(gurobi::IntAttr::Status).unwrap();
  assert_eq!(status, 2);

  let objval = model.get_double(gurobi::DoubleAttr::ObjVal).unwrap();
  assert!((objval - 1.0).abs() < 1e-12);

  let numvars = model.get_int(gurobi::IntAttr::NumVars).unwrap() as usize;
  assert_eq!(numvars, 3);

  let xval = model.get_double_array(gurobi::DoubleAttr::X, 0, numvars).unwrap();
  assert_eq!(xval[0], 0.0);
  assert_eq!(xval[1], 1.0);
  assert_eq!(xval[2], 0.0);

  model.write("mip1.lp").unwrap();
}
