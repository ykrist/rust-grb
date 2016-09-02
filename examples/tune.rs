// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;
use std::env::args;

fn main() {
  let mut env = Env::new("tune.log").unwrap();
  // set the number of improved parameter sets.
  env.set(param::TuneResults, 1).unwrap();

  let mut model = Model::read_from(args().nth(1).as_ref().unwrap(), &env).unwrap();

  model.tune().unwrap();

  let tune_cnt = model.get(attr::TuneResultCount).unwrap();
  if tune_cnt > 0 {
    model.get_tune_result(0).unwrap();
    model.write("tune.prm").unwrap();

    model.optimize().unwrap();
  }
}
