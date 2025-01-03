use grb::*;

mod workforce;

fn main() {
    let mut env = Env::new("workforce2.log").unwrap();
    env.set(param::LogToConsole, 0).unwrap();

    let mut model = workforce::make_model(&env).unwrap();

    let mut removed = Vec::new();
    for loop_count in 0..100 {
        println!("[iteration {loop_count}]");

        model.optimize().unwrap();

        match model.status().unwrap() {
            Status::Optimal => break,

            Status::Infeasible => {
                // compute IIS.
                model.compute_iis().unwrap();

                let c = {
                    let constr = model.get_constrs().unwrap();
                    let iis = model
                        .get_obj_attr_batch(attr::IISConstr, constr.iter().copied())
                        .unwrap();

                    let iis_constrs: Vec<_> = constr
                        .iter()
                        .zip(iis.iter())
                        .filter_map(|(&c, &val)| if val == 1 { Some(c) } else { None })
                        .collect();
                    println!("number of IIS constrs = {}", iis_constrs.len());
                    iis_constrs.first().copied()
                };

                let Some(c) = c else {
                    println!("There aren't any IIS constraints in the model.");
                    break;
                };

                let cname = model.get_obj_attr(attr::ConstrName, &c).unwrap();
                model.remove(c).unwrap();
                model.update().unwrap();
                removed.push(cname);
            }

            Status::InfOrUnbd | Status::Unbounded => {
                println!("The model is unbounded.");
                return;
            }

            status => {
                println!("Optimization is stopped with status {status:?}");
                return;
            }
        }
    }

    println!("removed variables are: {removed:?}");
}
