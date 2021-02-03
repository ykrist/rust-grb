use grb::prelude::*;

fn main() -> grb::Result<()> {
  // create an empty model.
  let mut model = Model::new("qcp")?;

  // add & integrate new variables.
  let x = add_ctsvar!(model, name: "x")?;
  let y = add_ctsvar!(model, name: "y")?;
  let z = add_ctsvar!(model, name: "z")?;
  // model.update().unwrap();

  // set objective funtion:
  //   f(x,y,z) = x
  model.set_objective(x, Maximize)?;

  // add linear constraints
  //  c0: x + y + z == 1
  model.add_constr("c0", c!(x + y + z == 1))?;

  // add quadratic constraints
  model.add_qconstr("qc0", c!(x*x + y*y <= z*z))?;
  model.add_qconstr("qc1", c!(x*x <= y*z))?;

  // optimize the model.
  model.optimize()?;

  // write the model to file.
  model.write("qcp.lp")?;
  model.write("qcp.sol")?;

  Ok(())
}
