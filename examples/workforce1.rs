extern crate gurobi;
extern crate itertools;

use std::iter::repeat;
use gurobi::*;
use itertools::*;

fn main() {
  // Set of worker's names
  let workers = vec!["Amy", "Bob", "Cathy", "Dan", "Ed", "Fred", "Gu"];

  // Amount each worker is paid to to work per shift
  let pays = vec![10.0, 12.0, 10.0, 8.0, 8.0, 9.0, 11.0];

  // Set of shift labels
  let shifts = vec!["Mon1", "Tue2", "Wed3", "Thu4", "Fri5", "Sat6", "Sun7", "Mon8", "Tue9", "Wed10", "Thu11", "Fri12",
                    "Sat13", "Sun14"];

  // Number of workers required for each shift
  let shift_requirements = vec![3.0, 2.0, 4.0, 4.0, 5.0, 6.0, 5.0, 2.0, 2.0, 3.0, 4.0, 6.0, 7.0, 5.0];

  // Worker availability: 0 if the worker is unavailable for a shift
  let availability = vec![
     vec![ 0, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1 ],
     vec![ 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0 ],
     vec![ 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1 ],
     vec![ 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1 ],
     vec![ 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 1 ],
     vec![ 1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1 ],
     vec![ 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 ],
   ];

  let env = Env::new("workforce1.log").unwrap();
  let mut model = env.new_model("assignment").unwrap();

  let mut x = Vec::new();
  for (worker, availability) in Zip::new((workers.iter(), availability.iter())) {

    let mut xshift = Vec::new();
    for (shift, &availability) in Zip::new((shifts.iter(), availability.iter())) {
      let vname = format!("{}.{}", worker, shift);
      let v = model.add_var(vname.as_str(), Continuous(-INFINITY, availability as f64)).unwrap();
      xshift.push(v);
    }

    x.push(xshift);
  }
  model.update().unwrap();

  let objterm = pays.iter().map(|pay| repeat(pay).take(shifts.len()));

  let objexpr = Zip::new((x.iter().flatten(), objterm.flatten())).fold(LinExpr::new(),
                                                                       |expr, (ref x, &c)| expr.term((*x).clone(), c));
  model.set_objective(objexpr, Minimize).unwrap();

  for (s, (shift, &requirement)) in shifts.iter().zip(shift_requirements.iter()).enumerate() {
    model.add_constr(format!("c.{}", shift).as_str(),
                  x.iter().map(|ref x| x[s].clone()).fold(LinExpr::new(), |expr, x| expr.term(x, 1.0)),
                  Equal,
                  requirement)
      .unwrap();
  }

  model.write("assignment.lp").unwrap();
  model.optimize().unwrap();

  let status = model.status().unwrap();
  if status == model::Status::Infeasible {
    // compute IIS.
    model.compute_iis().unwrap();
    model.write("assignment.ilp").unwrap();

    println!("The following constraint(s) cannot be satisfied:");
    for c in model.get_iis_constrs().unwrap().iter() {
      let cname = c.get(&model, attr::ConstrName).unwrap();
      println!("{}", cname);
    }
  }
}
