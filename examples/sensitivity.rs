// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;
use std::env::args;
use std::mem::transmute;

fn main() {
  let mut env = Env::new("").unwrap();
  env.set(param::LogToConsole, 0).unwrap();
  let mut model = env.read_model(args().nth(1).as_ref().unwrap()).unwrap();
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
  // env.set(param::OutputFlag, 0).unwrap();

  // iterate through unfixed, binary variables in model
  let vars: Vec<_> = model.get_vars().cloned().collect();
  for (v, &orig_x) in vars.iter().zip(orig_sol.iter()) {
    let lb = v.get(&model, attr::LB).unwrap();
    let ub = v.get(&model, attr::UB).unwrap();
    let vtype = v.get(&model, attr::VType).unwrap();
    let vtype = unsafe { transmute::<_, u8>(vtype) } as char;

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
