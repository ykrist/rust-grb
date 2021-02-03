use grb::prelude::*;

fn main() -> grb::Result<()> {
  let env = Env::new("mip.log")?;
  let mut model = Model::with_env("mip", &env)?;

  let x = add_binvar!(model, name: "x")?;
  let y = add_binvar!(model, name: "y")?;
  let z = add_binvar!(model, name: "z")?;
  model.update()?;

  model.set_objective(x + y + 2 * z, Minimize)?;

  model.add_constr("c0", c!(x + 2 * y + 3 * z <= 4))?;
  model.add_constr("c1", c!(x + y >= 1))?;
  let (range_var, _range_constr) = model.add_range("range", c!(x + 2.6*y in 1..10))?;
  model.update()?;
  model.set_obj_attr(attr::VarName, &range_var, "range-variable".to_string())?;
  model.optimize()?;

  assert_eq!(model.status()?, Status::Optimal);

  let objval = model.get_attr(attr::ObjVal)?;
  assert_eq!(objval.round() as isize, 1);
  let numvars = model.get_attr(attr::NumVars)?;
  assert_eq!(numvars, 4); // Note - the add_range method adds a variable as well

  let get_value = | var | model.get_obj_attr(attr::X, var);
  assert_eq!(get_value(&x)?.round() as isize, 0);
  assert_eq!(get_value(&y)?.round() as isize, 1);
  assert_eq!(get_value(&z)?.round() as isize, 0);

  model.write("mip.lp")?;
  model.write("mip.sol")?;

  Ok(())
}
