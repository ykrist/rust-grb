use grb::prelude::*;

fn main() -> grb::Result<()> {
    let env = Env::new("qp.log")?;

    // create an empty model.
    let mut model = Model::with_env("qp", &env)?;

    // add new variables.
    let x = add_ctsvar!(model, name: "x", bounds: 0..1)?;
    let y = add_ctsvar!(model, name: "y", bounds: 0..1)?;
    let z = add_ctsvar!(model, name: "z", bounds: 0..1)?;
    // model.update()?;

    // set objective funtion:
    model.set_objective(
        x * x + x * y + y * y + y * z + 2 * (z * z) + 2 * x,
        Minimize,
    )?;

    // add linear constraints

    model.add_constr("c0", c!(x + 2 * y + 3 * z >= 4))?;
    model.add_constr("c1", c!(x + y >= 1))?;

    // optimize the model.
    model.optimize()?;

    // write the model to file.
    model.write("qp.lp")?;
    model.write("qp.sol")?;
    Ok(())
}
