// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
extern crate itertools;

use std::iter::repeat;
use gurobi::*;
use self::itertools::*;

pub fn make_model(env: &Env) -> Result<Model> {
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

  let mut model = try!(Model::new("assignment", &env));

  let mut x = Vec::new();
  for (worker, availability) in Zip::new((workers.iter(), availability.iter())) {
    let mut xshift = Vec::new();
    for (shift, &availability) in Zip::new((shifts.iter(), availability.iter())) {
      let vname = format!("{}.{}", worker, shift);
      let v = try!(model.add_var(vname.as_str(), Continuous, 0.0, -INFINITY, availability as f64, &[], &[]));
      xshift.push(v);
    }
    x.push(xshift);
  }
  try!(model.update());

  let objterm = pays.iter().map(|pay| repeat(pay).take(shifts.len()));
  let objexpr = Zip::new((Itertools::flatten(x.iter()), Itertools::flatten(objterm)))
                  .fold(LinExpr::new(), |expr, (x, &c)| expr + c * x);
  try!(model.set_objective(objexpr, Minimize));

  for (s, (shift, &requirement)) in shifts.iter().zip(shift_requirements.iter()).enumerate() {
    try!(model.add_constr(format!("c.{}", shift).as_str(),
                          x.iter().map(|ref x| &x[s]).fold(LinExpr::new(), |expr, x| expr + x),
                          Equal,
                          requirement));
  }

  Ok(model)
}
