use grb::prelude::*;

#[allow(clippy::many_single_char_names)]
fn main() -> grb::Result<()> {
    let mut model = Model::new("piecewise")?;

    // Add variables.
    let x = add_ctsvar!(model, name: "x", bounds: 0..1)?;
    let y = add_ctsvar!(model, name: "y", bounds: 0..1)?;
    let z = add_ctsvar!(model, name: "z", bounds: 0..1)?;
    model.update()?;

    // Add constraints.
    model.add_constr("c0", c!(x + 2 * y + 3 * z <= 4))?;
    model.add_constr("c1", c!(x + y >= 1))?;

    // Set `convex` objective function:
    //  minimize f(x) - y + g(z)
    //    where f(x) = exp(-x),  g(z) = 2 z^2 - 4 z

    let f = |x: f64| (-x).exp();
    let g = |z: f64| 2.0 * z * z - 4.0 * z;

    let n_points: usize = 101;
    let (lb, ub) = (0.0, 1.0);

    let pt_u: Vec<f64> = (0..n_points)
        .map(|i| lb + (ub - lb) * (i as f64) / ((n_points as f64) - 1.0))
        .collect();

    model.set_pwl_obj(&x, pt_u.iter().map(|&u| (u, f(u))))?;
    model.set_pwl_obj(&z, pt_u.iter().map(|&u| (u, g(u))))?;
    model.set_obj_attr(attr::Obj, &y, -1.0)?;

    optimize_and_print_status(&mut model)?;

    // Negate piecewise-linear objective function for x.
    // And then the objective function becomes non-convex.
    model.set_pwl_obj(&x, pt_u.iter().map(|&u| (u, -f(u))))?;

    optimize_and_print_status(&mut model)
}

fn optimize_and_print_status(model: &mut Model) -> grb::Result<()> {
    model.optimize()?;

    println!("IsMIP = {}", model.get_attr(attr::IsMIP)? != 0);
    let vars = model.get_vars()?;
    for v in vars {
        let vname = model.get_obj_attr(attr::VarName, v)?;
        let x = model.get_obj_attr(attr::X, v)?;
        println!("{} = {}", vname, x);
    }
    println!("Obj = {}\n", model.get_attr(attr::ObjVal)?);
    Ok(())
}
