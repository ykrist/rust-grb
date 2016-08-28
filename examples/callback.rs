// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

extern crate gurobi;
use gurobi::*;


fn main() {
  let mut env = Env::new("callback.log").unwrap();
  env.set(param::OutputFlag, 0).unwrap();
  env.set(param::Heuristics, 0.0).unwrap();

  let mut model = env.read_model(&std::env::args().nth(1).unwrap()).unwrap();

  let callback = {
    let mut lastiter = -INFINITY;
    let mut lastnode = -INFINITY;
    let vars: Vec<_> = model.get_vars().cloned().collect();

    move |ctx: Callback| {
      use gurobi::Where::*;
      match ctx.get_where() {
        // Periodic polling callback
        Polling => {
          // Ignore polling callback
        }

        // Currently performing presolve
        PreSolve { coldel, rowdel, .. } => {
          println!("@PreSolve");
          if coldel > 0 || rowdel > 0 {
            println!("**** {} columns and {} rows are removed. ****",
                     coldel,
                     rowdel);
          }
        }

        // Currently in simplex
        Simplex { ispert, itrcnt, objval, priminf, dualinf } => {
          if itrcnt - lastiter >= 100.0 {
            lastiter = itrcnt;
            let ch = match ispert {
              0 => ' ',
              1 => 'S',
              _ => 'P'
            };
            println!("@Simplex: itrcnt={}, objval={}{}, priminf={}, dualinf={}.",
                     itrcnt,
                     objval,
                     ch,
                     priminf,
                     dualinf);
          }
        }

        // Currently in MIP
        MIP { solcnt, cutcnt, objbst, objbnd, nodcnt, nodleft: actnodes, itrcnt } => {
          if nodcnt - lastnode >= 100.0 {
            lastnode = nodcnt;
            println!("@MIP: nodcnt={}, actnodes={}, itrcnt={}, objbst={}, objbnd={}, solcnt={}, cutcnt={}.",
                     nodcnt,
                     actnodes,
                     itrcnt,
                     objbst,
                     objbnd,
                     solcnt,
                     cutcnt);
          }

          if (objbst - objbnd).abs() < 0.1 * (1.0 + objbst.abs()) {
            println!("Stop early - 10% gap achived");
            ctx.terminate();
          }

          if nodcnt >= 10000.0 && solcnt != 0.0 {
            println!("Stop early - 10000 nodes explored");
            ctx.terminate();
          }
        }

        // Found a new MIP incumbent
        MIPSol { solcnt, obj, nodcnt, .. } => {
          println!("@MIPSol: ");
          let x = try!(ctx.get_solution(vars.as_slice()));
          println!("**** New solution at node {}, obj {}, sol {}, x[0] = {} ****",
                   nodcnt,
                   obj,
                   solcnt,
                   x[0]);
        }

        // Currently exploring a MIP node
        MIPNode { .. } => {
          println!("@MIPNode");
          println!("**** NEW NODE! ****");
          let x = try!(ctx.get_node_rel(vars.as_slice()));
          println!("  relaxed solution = {:?}", x);
          try!(ctx.set_solution(vars.as_slice(), x.as_slice()));
        }

        // Currently in barrier
        Barrier { itrcnt, primobj, dualobj, priminf, dualinf, compl } => {
          println!("@Barrier: itrcnt={}, primobj={}, dualobj={}, priminf={}, dualinf={}, compl={}.",
                   itrcnt,
                   primobj,
                   dualobj,
                   priminf,
                   dualinf,
                   compl);
        }

        // Printing a log message
        Message(message) => {
          println!("@Message: {}", message);
        }
      }

      Ok(())
    }
  };
  model.optimize_with_callback(callback).unwrap();

  println!("\nOptimization complete");
  if model.get(attr::SolCount).unwrap() == 0 {
    println!("No solution found. optimization status = {:?}",
             model.status());
  } else {
    println!("Solution found. objective = {}",
             model.get(attr::ObjVal).unwrap());
    for v in model.get_vars() {
      let vname = v.get(&model, attr::VarName).unwrap();
      let value = v.get(&model, attr::X).unwrap();
      if value > 1e-25 {
        println!("  {}: {}", vname, value);
      }
    }
  }
}
