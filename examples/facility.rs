use grb::prelude::*;

fn main() -> grb::Result<()> {
    // warehouse demand in thousands of units.
    let demand = [15f64, 18.0, 14.0, 20.0];

    // plant capacity in thousands of units.
    let capacity = [20f64, 22.0, 17.0, 19.0, 18.0];

    // fixed costs for each plant.
    let fixed_costs = [12000f64, 15000.0, 17000.0, 13000.0, 16000.0];

    // transportation costs per thousands units.
    let trans_costs = [
        [4000f64, 2000.0, 3000.0, 2500.0, 4500.0],
        [2500f64, 2600.0, 3400.0, 3000.0, 4000.0],
        [1200f64, 1800.0, 2600.0, 4100.0, 3000.0],
        [2200f64, 2600.0, 3100.0, 3700.0, 3200.0],
    ];

    let env = Env::new("facility.log")?;
    let mut model = Model::with_env("facility", &env)?;

    // plant open decision variables.
    // open[p] == 1 means that plant p is open.
    let open: Vec<Var> = fixed_costs
        .iter()
        .enumerate()
        .map(|(p, &cost)| add_binvar!(model, name: &format!("Open{p}"), obj: cost).unwrap())
        .collect();

    // transportation decision variables.
    // how much transport from a plant p to a warehouse w
    let trans_vars: Vec<Vec<Var>> = trans_costs
        .iter()
        .enumerate()
        .map(|(w, costs)| {
            costs
                .iter()
                .enumerate()
                .map(|(p, &cost)| {
                    add_ctsvar!(model, name: &format!("Trans{p}.{w}"), obj: cost).unwrap()
                })
                .collect()
        })
        .collect();

    let expr = open
        .iter()
        .chain(trans_vars.iter().flat_map(|tr| tr.iter()))
        .zip(
            fixed_costs
                .iter()
                .chain(trans_costs.iter().flat_map(|c| c.iter())),
        )
        .map(|(&x, &c)| x * c)
        .grb_sum();

    model.update()?;
    model.set_objective(expr, Minimize)?;

    for (p, (&capacity, &open)) in capacity.iter().zip(&open).enumerate() {
        let lhs = trans_vars.iter().map(|t| t[p]).grb_sum();
        model.add_constr(&format!("Capacity{p}"), c!(lhs <= capacity * open))?;
    }

    for (w, (&demand, tvars)) in demand.iter().zip(&trans_vars).enumerate() {
        model.add_constr(&format!("Demand{w}"), c!(tvars.iter().grb_sum() == demand))?;
    }

    for o in open.iter() {
        model.set_obj_attr(attr::Start, o, 1.0)?;
    }

    println!("Initial guesss:");
    let max_fixed = fixed_costs
        .iter()
        .cloned()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    for (p, (open, &cost)) in open.iter().zip(&fixed_costs).enumerate() {
        if (cost - max_fixed).abs() < 1e-4 {
            model.set_obj_attr(attr::Start, open, 0.0)?;
            println!("Closing plant {p}");
            break;
        }
    }
    println!();

    // use barrier to solve root relaxation.
    model.get_env_mut().set(param::Method, 2)?;

    // solve.
    model.optimize()?;

    // print solution.
    println!("\nTOTAL COSTS: {}", model.get_attr(attr::ObjVal)?);
    println!("SOLUTION:");
    for (p, open) in open.iter().enumerate() {
        let x = model.get_obj_attr(attr::X, open)?;
        if x > 0.9 {
            println!("Plant {p} is open");
            for (w, trans) in trans_vars.iter().enumerate() {
                let t = model.get_obj_attr(attr::X, &trans[p])?;
                if t > 0.0 {
                    println!("  Transport {t} units to warehouse {w}");
                }
            }
        } else {
            println!("Plant {p} is closed!");
        }
    }

    model.write("facility.lp")?;
    model.write("facility.sol")?;

    Ok(())
}
