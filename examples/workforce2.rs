// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;

mod workforce;
use workforce::make_model;

fn main() {
  let mut env = Env::new("workforce2.log").unwrap();
  env.set(param::LogToConsole, 0).unwrap();

  let mut model = make_model(&env).unwrap();

  let mut removed = Vec::new();
  for loop_count in 0..100 {
    println!("[iteration {}]", loop_count);

    model.optimize().unwrap();

    match model.status().unwrap() {
      Status::Optimal => break,

      Status::Infeasible => {
        // compute IIS.
        model.compute_iis().unwrap();

        let c = {
          let iis_constrs: Vec<_> =
            model.get_constrs().filter(|c| c.get(&model, attr::IISConstr).unwrap() != 0).collect();
          println!("number of IIS constrs = {}", iis_constrs.len());
          iis_constrs.into_iter().nth(0).cloned()
        };

        match c {
          Some(c) => {
            let cname = c.get(&model, attr::ConstrName).unwrap();
            model.remove(c);
            model.update().unwrap();
            removed.push(cname);
          }
          None => {
            println!("There are any IIS constraints in the model.");
            break;
          }
        }
      }

      Status::InfOrUnbd | Status::Unbounded => {
        println!("The model is unbounded.");
        return;
      }

      status => {
        println!("Optimization is stopped with status {:?}", status);
        return;
      }
    }
  }

  println!("removed variables are: {:?}", removed);
}
