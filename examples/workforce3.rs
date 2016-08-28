// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
extern crate itertools;
use gurobi::*;
use itertools::*;

mod workforce;
use workforce::make_model;

fn main() {
  let mut env = Env::new("workforce3.log").unwrap();
  env.set(param::LogToConsole, 0).unwrap();

  let mut model = make_model(&env).unwrap();
  model.optimize().unwrap();

  match model.status().unwrap() {
    Status::Infeasible => {
      let mut model = model.copy().unwrap();
      model.set(attr::ModelName, "assignment_relaxed".to_owned()).unwrap();

      // do relaxation.
      let constrs = model.get_constrs().cloned().collect_vec();
      let slacks = {
        let (_, svars, _, _) = model.feas_relax(RelaxType::Linear,
                      false,
                      &[],
                      &[],
                      &[],
                      &constrs[..],
                      RepeatN::new(1.0, constrs.len()).collect_vec().as_slice())
          .unwrap();
        svars.cloned().collect_vec()
      };
      model.optimize().unwrap();

      println!("slack variables: ");
      for slack in slacks {
        let value = slack.get(&model, attr::X).unwrap();
        let vname = slack.get(&model, attr::VarName).unwrap();
        if value > 1e-6 {
          println!("  * {} = {}", vname, value);
        }
      }
    }

    Status::Optimal => {
      println!("The model is feasible and optimized.");
    }

    Status::InfOrUnbd | Status::Unbounded => {
      println!("The model is unbounded.");
    }

    status => {
      println!("Optimization is stopped with status {:?}", status);
    }
  }
}
