// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
extern crate itertools;
use gurobi::*;
use itertools::*;

fn main() {
  // warehouse demand in thousands of units.
  let demand = vec![15f64, 18.0, 14.0, 20.0];

  // plant capacity in thousands of units.
  let capacity = vec![20f64, 22.0, 17.0, 19.0, 18.0];

  // fixed costs for each plant.
  let fixed_costs = vec![12000f64, 15000.0, 17000.0, 13000.0, 16000.0];

  // transportation costs per thousands units.
  let trans_costs = vec![vec![4000f64, 2000.0, 3000.0, 2500.0, 4500.0],
                         vec![2500f64, 2600.0, 3400.0, 3000.0, 4000.0],
                         vec![1200f64, 1800.0, 2600.0, 4100.0, 3000.0],
                         vec![2200f64, 2600.0, 3100.0, 3700.0, 3200.0]];

  let env = Env::new("facility.log").unwrap();
  let mut model = Model::new("facility", &env).unwrap();

  // plant open decision variables.
  // open[p] == 1 means that plant p is open.
  let open: Vec<Var> =
    (0..(fixed_costs.len())).map(|p| model.add_var(&format!("Open{}", p), Binary, 0.0, 0.0, 1.0, &[], &[]).unwrap()).collect();

  // transportation decision variables.
  // how much transport from a plant p to a warehouse w
  let transport: Vec<Vec<_>> = trans_costs.iter()
    .enumerate()
    .map(|(w, costs)| {
      (0..(costs.len()))
        .map(|p| model.add_var(&format!("Trans{}.{}", p, w), Continuous, 0.0, 0.0, INFINITY, &[], &[]).unwrap())
        .collect()
    })
    .collect();

  let expr = Zip::new((open.iter().chain(Itertools::flatten(transport.iter())),
                       fixed_costs.iter().chain(Itertools::flatten(trans_costs.iter()))))
    .fold(LinExpr::new(), |expr, (x, &c)| expr + c * x);
  model.set_objective(expr, Minimize).unwrap();
  model.update().unwrap();

  for (p, (&capacity, open)) in Zip::new((&capacity, &open)).enumerate() {
    let lhs = transport.iter().map(|t| &t[p]).fold(LinExpr::new(), |expr, t| expr + t);
    model.add_constr(&format!("Capacity{}", p), lhs - capacity * open, Less, 0.0).unwrap();
  }

  for (w, (&demand, transport)) in Zip::new((&demand, &transport)).enumerate() {
    let lhs = transport.iter().fold(LinExpr::new(), |expr, t| expr + t);
    model.add_constr(&format!("Demand{}", w), lhs, Equal, demand).unwrap();
  }

  for o in open.iter() {
    o.set(&mut model, attr::Start, 1.0).unwrap();
  }

  println!("Initial guesss:");
  let max_fixed = fixed_costs.iter().cloned().fold(-1. / 0., f64::max);
  for (p, (open, &cost)) in Zip::new((&open, &fixed_costs)).enumerate() {
    if cost == max_fixed {
      open.set(&mut model, attr::Start, 0.0).unwrap();
      println!("Closing plant {}", p);
      break;
    }
  }
  println!("");

  // use barrier to solve root relaxation.
  model.get_env_mut().set(param::Method, 2).unwrap();

  // solve.
  model.optimize().unwrap();

  // print solution.
  println!("\nTOTAL COSTS: {}", model.get(attr::ObjVal).unwrap());
  println!("SOLUTION:");
  for (p, open) in open.iter().enumerate() {
    let x = open.get(&model, attr::X).unwrap();
    if x == 1.0 {
      println!("Plant {} is open", p);
      for (w, trans) in transport.iter().enumerate() {
        let t = trans[p].get(&model, attr::X).unwrap();
        if t > 0.0 {
          println!("  Transport {} units to warehouse {}", t, w);
        }
      }
    } else {
      println!("Plant {} is closed!", p);
    }
  }

  model.write("facility.lp").unwrap();
  model.write("facility.sol").unwrap();
}
