// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;
use std::env::args;

fn main() {
  let env = Env::new("").unwrap();

  let mut model = Model::read_from(args().nth(1).as_ref().unwrap(), &env).unwrap();
  assert!(model.get(attr::IsMIP).unwrap() != 0, "Model is not a MIP");

  model.optimize().unwrap();

  let status = model.status().unwrap();
  assert!(status == Status::Optimal,
          "Optimization ended with status {:?}",
          status);

  // store the optimal solution.
  let orig_obj = model.get(attr::ObjVal).unwrap();
  let orig_sol: Vec<_> = model.get_vars().map(|v| v.get(&model, attr::X).unwrap()).collect();

  // disable solver output for subsequent solvers.
  model.get_env_mut().set(param::OutputFlag, 0).unwrap();

  // iterate through unfixed, binary variables in model
  let vars: Vec<_> = model.get_vars().cloned().collect();
  for (v, &orig_x) in vars.iter().zip(orig_sol.iter()) {
    let (vtype, lb, ub) = v.get_type(&model).unwrap();

    if lb == 0.0 && ub == 1.0 && (vtype == 'B' || vtype == 'I') {
      let vname = v.get(&model, attr::VarName).unwrap();

      // set variable to 1 - x, where x is its value in optimal solution
      // (it means that `negate` value of the binary variable)
      let (lb, ub, start);
      if orig_x < 0.5 {
        lb = 1.0;
        ub = 1.0;
        start = 1.0;
      } else {
        lb = 0.0;
        ub = 0.0;
        start = 0.0;
      }
      v.set(&mut model, attr::LB, lb).unwrap();
      v.set(&mut model, attr::UB, ub).unwrap();
      v.set(&mut model, attr::Start, start).unwrap();

      // update MIP start for the other variables.
      for (vv, &orig_xx) in vars.iter().zip(orig_sol.iter()) {
        if v != vv {
          vv.set(&mut model, attr::Start, orig_xx).unwrap();
        }
      }

      // solve for new value and capture sensitivity information.
      model.optimize().unwrap();
      match model.status().unwrap() {
        Status::Optimal => {
          let objval = model.get(attr::ObjVal).unwrap();
          println!("Objective sensitivity for variable {} is {}",
                   vname,
                   objval - orig_obj);
        }
        _ => {
          println!("Objective sensitivity for variable {} is infinite", vname);
        }
      }

      // restore the original variable bounds
      v.set(&mut model, attr::LB, 0.0).unwrap();
      v.set(&mut model, attr::UB, 1.0).unwrap();
    }
  }
}
