extern crate gurobi;

fn main() {
  // the name of log file must not be 'mip1.log' (why?)
  let logfilename = "mip_1.log";
  let env = gurobi::Env::new(logfilename).unwrap();
  assert_eq!(env.get_str_param("LogFile").unwrap(), logfilename);

  let mut model = env.new_model("mip1").unwrap();

  model.add_bvar("x", 0.0).unwrap();
  model.add_bvar("y", 0.0).unwrap();
  model.add_bvar("z", 0.0).unwrap();
  model.update().unwrap();

  model.optimize().unwrap();

  let status = model.get_int(gurobi::IntAttr::Status).unwrap();
  println!("Status: {:?}", status);

  model.write("mip1.lp").unwrap();
}
