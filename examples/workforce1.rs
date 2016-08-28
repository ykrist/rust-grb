// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;

mod workforce;
use workforce::make_model;

fn main() {
  let mut env = Env::new("workforce1.log").unwrap();
  env.set(param::LogToConsole, 0).unwrap();

  let mut model = make_model(&env).unwrap();
  model.optimize().unwrap();

  let status = model.status().unwrap();
  if status == Status::Infeasible {
    model.compute_iis().unwrap();

    println!("The following constraint(s) cannot be satisfied:");
    for c in model.get_constrs().filter(|c| c.get(&model, attr::IISConstr).unwrap() != 0) {
      println!("  - {}", c.get(&model, attr::ConstrName).unwrap());
    }
  }
}
