// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

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
    let constr = model.get_constrs().unwrap();
    let iis_vals = model.get_obj_attr_batch(attr::IISConstr, constr).unwrap();
    let iis_constr : Vec<_> = constr.iter().zip(iis_vals.into_iter())
      .filter_map(|(&c, i)| if i == 1 { Some(c) } else { None })
      .collect();
    let iis_names = model.get_obj_attr_batch(attr::ConstrName, &iis_constr).unwrap();

    for name in iis_names {
      println!(" - {}", name);
    }

  }
}
