// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use gurobi::*;

pub fn make_model(env: &Env) -> Result<Model> {
  // Set of worker's names
  let workers = vec!["Amy", "Bob", "Cathy", "Dan", "Ed", "Fred", "Gu"];

  // Amount each worker is paid to to work per shift
  let pays = vec![10.0, 12.0, 10.0, 8.0, 8.0, 9.0, 11.0];

  // Set of shift labels
  let shifts = vec![
    "Mon1", "Tue2", "Wed3", "Thu4", "Fri5", "Sat6", "Sun7",
    "Mon8", "Tue9", "Wed10", "Thu11", "Fri12", "Sat13", "Sun14"
  ];

  // Number of workers required for each shift
  let shift_requirements = vec![
    3.0, 2.0, 4.0, 4.0, 5.0, 6.0, 5.0,
    2.0, 2.0, 3.0, 4.0, 6.0, 7.0, 5.0
  ];

  // Worker availability: 0 if the worker is unavailable for a shift
  let availability = vec![
    vec![0, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1],
    vec![1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0],
    vec![0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1],
    vec![0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1],
    vec![1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1, 1],
    vec![1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1],
    vec![1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  ];

  let mut model = Model::with_env("assignment", env)?;

  // x[worker_idx][shift_idx]
  let mut x = Vec::with_capacity(workers.len());
  for ((worker, &pay), worker_avail) in workers.iter().zip(&pays).zip(&availability) {
    let mut xshift = Vec::new();
    for (shift, &is_avail) in shifts.iter().zip(worker_avail) {
      let vname = format!("{}.{}", worker, shift);
      xshift.push(model.add_var(&vname, Binary, pay, 0.0, is_avail as f64, &[], &[])?);
    }
    x.push(xshift);
  }
  model.update()?;
  model.set_attr(attr::ModelSense, ModelSense::Minimize.into())?;

  for (s, (shift, &requirement)) in shifts.iter().zip(shift_requirements.iter()).enumerate() {
    model.add_constr(format!("shift-{}", shift).as_str(),
                     c!(x.iter().map(|x_workers| x_workers[s]).grb_sum() == requirement))?;
  }

  Ok(model)
}
