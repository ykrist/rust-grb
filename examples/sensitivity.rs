use grb::*;

mod example_utils;
use example_utils::*;

fn main() {
    let mut model = load_model_file_from_clarg();
    assert!(
        model.get_attr(attr::IsMIP).unwrap() != 0,
        "Model is not a MIP"
    );

    model.optimize().unwrap();

    let status = model.status().unwrap();
    assert!(
        status == Status::Optimal,
        "Optimization ended with status {:?}",
        status
    );

    // store the optimal solution.
    let orig_obj = model.get_attr(attr::ObjVal).unwrap();
    let orig_sol: Vec<_> = model
        .get_vars()
        .unwrap()
        .iter()
        .map(|v| model.get_obj_attr(attr::X, v).unwrap())
        .collect();

    // disable solver output for subsequent solvers.
    model.get_env_mut().set(param::OutputFlag, 0).unwrap();

    // iterate through unfixed, binary variables in model
    let vars = model.get_vars().unwrap().to_vec();
    for (v, &orig_x) in vars.iter().zip(orig_sol.iter()) {
        let vtype = model.get_obj_attr(attr::VType, v).unwrap();
        if !(vtype == VarType::Binary || vtype == VarType::Integer) {
            continue;
        }

        let mut lb = model.get_obj_attr(attr::LB, v).unwrap();
        let mut ub = model.get_obj_attr(attr::UB, v).unwrap();

        if (lb - 0.0).abs() < 1e-6 && (ub - 1.0).abs() < 1e-6 {
            let vname = model.get_obj_attr(attr::VarName, v).unwrap();

            // set variable to 1 - x, where x is its value in optimal solution
            // (it means that `negate` value of the binary variable)
            let start;
            if orig_x < 0.5 {
                lb = 1.0;
                ub = 1.0;
                start = 1.0;
            } else {
                lb = 0.0;
                ub = 0.0;
                start = 0.0;
            }

            model.set_obj_attr(attr::LB, v, lb).unwrap();
            model.set_obj_attr(attr::UB, v, ub).unwrap();
            model.set_obj_attr(attr::Start, v, start).unwrap();

            // update MIP start for the other variables.
            for (vv, &orig_xx) in vars.iter().zip(orig_sol.iter()) {
                if v != vv {
                    model.set_obj_attr(attr::Start, v, orig_xx).unwrap();
                }
            }

            // solve for new value and capture sensitivity information.
            model.optimize().unwrap();
            match model.status().unwrap() {
                Status::Optimal => {
                    let objval = model.get_attr(attr::ObjVal).unwrap();
                    println!(
                        "Objective sensitivity for variable {} is {}",
                        vname,
                        objval - orig_obj
                    );
                }
                _ => {
                    println!("Objective sensitivity for variable {} is infinite", vname);
                }
            }

            // restore the original variable bounds
            model.set_obj_attr(attr::LB, v, 0.0).unwrap();
            model.set_obj_attr(attr::UB, v, 1.0).unwrap();
        }
    }
}
