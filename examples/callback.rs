use grb::prelude::*;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
mod example_utils;
use example_utils::*;

fn main() -> grb::Result<()> {
    let mut model = load_model_file_from_clarg();
    model.set_param(param::Heuristics, 0.0)?;
    let vars = model.get_vars()?.to_vec();

    let mut callback = {
        let mut lastiter = -INFINITY;
        let mut lastnode = -INFINITY;

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("cb.log")
            .unwrap();
        let mut writer = BufWriter::new(file);

        move |w: Where| {
            use Where::*;
            match w {
                // Periodic polling callback
                Polling(_) => {
                    // Ignore polling callback
                }

                // Currently performing presolve
                PreSolve(ctx) => {
                    println!("@PreSolve");
                    let (coldel, rowdel) = (ctx.col_del()?, ctx.row_del()?);
                    if coldel > 0 || rowdel > 0 {
                        println!(
                            "**** {} columns and {} rows removed so far. ****",
                            coldel, rowdel
                        );
                    }
                }

                // Currently in simplex
                Simplex(ctx) => {
                    let itrcnt = ctx.iter_cnt()?;
                    if itrcnt - lastiter >= 100.0 {
                        lastiter = itrcnt;
                        let ch = match ctx.is_perturbed()? {
                            0 => ' ',
                            1 => 'S',
                            _ => 'P',
                        };
                        println!(
                            "@Simplex: itrcnt={}, objval={}{}, priminf={}, dualinf={}.",
                            itrcnt,
                            ctx.obj_val()?,
                            ch,
                            ctx.prim_inf()?,
                            ctx.dual_inf()?
                        );
                    }
                }

                // Currently in MIP
                MIP(ctx) => {
                    let (objbst, objbnd, solcnt, nodcnt) = (
                        ctx.obj_best()?,
                        ctx.obj_bnd()?,
                        ctx.sol_cnt()?,
                        ctx.node_cnt()?,
                    );

                    if nodcnt - lastnode >= 100.0 {
                        lastnode = nodcnt;
                        println!("@MIP: nodcnt={}, actnodes={}, itrcnt={}, objbst={}, objbnd={}, solcnt={}, cutcnt={}.",
                     nodcnt,
                     ctx.node_left()?,
                     ctx.iter_cnt()?,
                     objbst,
                     objbnd,
                     solcnt,
                     ctx.cut_cnt()?);
                    }

                    if (objbst - objbnd).abs() < 0.1 * (1.0 + objbst.abs()) {
                        println!("Stop early - 10% gap achived");
                        ctx.terminate();
                    }

                    if nodcnt >= 10000.0 && solcnt != 0 {
                        println!("Stop early - 10000 nodes explored");
                        ctx.terminate();
                    }
                }

                // Found a new MIP incumbent
                MIPSol(ctx) => {
                    println!("@MIPSol: ");
                    let x = ctx.get_solution(&vars)?;
                    println!(
                        "**** New solution at node {}, obj {}, sol {}, x[0] = {} ****",
                        ctx.node_cnt()?,
                        ctx.obj()?,
                        ctx.sol_cnt()?,
                        x[0]
                    );
                }

                // Currently exploring a MIP node
                MIPNode(ctx) => {
                    println!("@MIPNode");
                    println!("**** NEW NODE! ****");
                    let x = ctx.get_solution(&vars)?;
                    if ctx.status()? == Status::Optimal {
                        println!("  relaxation solution = {:?}", x);
                        let obj = ctx.set_solution(vars.iter().zip(x))?;
                        // Should not return None - we didn't modify the solution
                        assert!(obj.is_some());
                    }
                }

                // Currently in barrier
                Barrier(ctx) => {
                    println!("@Barrier: itrcnt={}, primobj={}, dualobj={}, priminf={}, dualinf={}, compl={}.",
                   ctx.iter_cnt()?,
                   ctx.prim_obj()?,
                   ctx.dual_obj()?,
                   ctx.prim_inf()?,
                   ctx.dual_inf()?,
                   ctx.compl_viol()?)
                }

                // Printing a log message
                Message(ctx) => {
                    writer.write_all(ctx.message()?.as_bytes())?;
                    writer.write_all(&[b'\n'])?;
                }

                _ => {}
            }

            Ok(())
        }
    };

    model.optimize_with_callback(&mut callback)?;

    println!("\nOptimization complete");
    if model.get_attr(attr::SolCount)? == 0 {
        println!(
            "No solution found. optimization status = {:?}",
            model.status()
        );
    } else {
        println!(
            "Solution found. objective = {}",
            model.get_attr(attr::ObjVal)?
        );
        for v in model.get_vars().unwrap() {
            let vname = model.get_obj_attr(attr::VarName, v)?;
            let value = model.get_obj_attr(attr::X, v)?;
            if value > 1e-10 {
                println!("  {}: {}", vname, value);
            }
        }
    }
    Ok(())
}
