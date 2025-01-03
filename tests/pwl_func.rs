use grb::prelude::*;

fn f(u: f64) -> f64 {
    u.exp()
}
fn g(u: f64) -> f64 {
    u.sqrt()
}

fn print_sol(m: &Model) -> anyhow::Result<()> {
    // XXX: use GRBgetdblarray
    let xs: Vec<_> = m
        .get_vars()?
        .iter()
        .map(|v| m.get_obj_attr(attr::X, v))
        .collect::<Result<_, _>>()?;

    println!("x = {}, u = {}", xs[0], xs[2]);
    println!("y = {}, v = {}", xs[1], xs[3]);

    // Calculate violation of exp(x) + 4 sqrt(y) <= 9
    let vio = f(xs[0]) + 4. * g(xs[1]) - 9.;
    let vio = if vio < 0. { 0. } else { vio };
    println!("Vio = {vio}");

    Ok(())
}

const INTERVAL: f64 = 1e-3;

fn setup() -> anyhow::Result<(Model, [Var; 4])> {
    let mut model = Model::new("")?;

    let x = add_ctsvar!(model, obj: 2.0, name: "x")?;
    let y = add_ctsvar!(model, obj: 1.0, name: "y")?;
    let u = add_ctsvar!(model, name: "u")?;
    let v = add_ctsvar!(model, name: "v")?;

    model.set_attr(attr::ModelSense, Maximize)?;

    model.add_constr("c1", c!(u + 4 * v <= 9))?;

    Ok((model, [x, y, u, v]))
}

#[test]
/// Approach 1) PWL constraint approach
fn pwl_genconstr() -> anyhow::Result<()> {
    let (mut model, [x, y, u, v]) = setup()?;

    let x_max = 9.0f64.log10();
    let len = (x_max / INTERVAL).ceil() as usize + 1;

    let x_points = (0..len).map(|i| i as f64 * INTERVAL);
    let u_points = (0..len).map(|i| f(i as f64 * INTERVAL));

    model.add_genconstr_pwl("gc1", x, u, x_points.zip(u_points))?;

    let y_max = (9.0f64 / 4.).powi(2);
    let len = (y_max / INTERVAL).ceil() as usize + 1;

    let y_points = (0..len).map(|i| i as f64 * INTERVAL);
    let v_points = (0..len).map(|i| g(i as f64 * INTERVAL));

    model.add_genconstr_pwl("gc2", y, v, y_points.zip(v_points))?;

    model.optimize()?;
    print_sol(&model)?;

    Ok(())
}

#[test]
/// Approach 2) General function constraint approach
/// with auto PWL translation by Gurobi
fn function_genconstr() -> anyhow::Result<()> {
    let (mut model, [x, y, u, v]) = setup()?;

    model.add_genconstr_natural_exp("gcf1", x, u, "")?;
    model.add_genconstr_pow("gcf2", y, v, 0.5, "")?;

    // Use the equal piece length approach with the length = INTERVAL
    model.set_param(param::FuncPieces, 1)?;
    model.set_param(param::FuncPieceLength, INTERVAL)?;

    // Optimize the model and print solution
    model.optimize()?;
    print_sol(&model)?;

    // Zoom in, use optimal solution to reduce the ranges and use a smaller
    // pclen=1e-5 to solve it
    let xs = model.get_obj_attr_batch(attr::X, [x, y])?;

    let t = xs[0] - 0.01;
    let t = if t < 0.0 { 0.0 } else { t };

    model.set_obj_attr(attr::LB, &y, t)?;
    model.set_obj_attr(attr::UB, &x, xs[0] + 0.01)?;
    model.set_obj_attr(attr::UB, &y, xs[1] + 0.01)?;

    model.update()?;

    model.reset()?;

    model.set_param(param::FuncPieceLength, 1e-5)?;

    // Optimize the model and print solution
    model.optimize()?;
    print_sol(&model)?;

    Ok(())
}
