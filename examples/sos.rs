use grb::prelude::*;

fn main() -> grb::Result<()> {
  let mut model = Model::new("sos")?;

  let x0 = add_ctsvar!(model, name: "x0")?;
  let x1 = add_ctsvar!(model, name: "x1")?;
  let x2 = add_ctsvar!(model, name: "x2")?;
  model.update()?;

  model.set_objective(2*x0 + x1 + x2, Maximize)?;

  // [x0 = 0] or [x1 = 0]
  model.add_sos([(x0, 1.0), (x1, 2.0)].iter().copied(), SOSType::Ty1)?;

  // [x0 = 0] or [x2 = 0]
  model.add_sos([(x0, 1.0), (x2, 2.0)].iter().copied(), SOSType::Ty1)?;

  model.optimize()?;

  model.write("sos.lp")?;
  model.write("sos.sol")?;

  let obj = model.get_attr(attr::ObjVal)?;
  assert_eq!(obj.round() as isize, 3);
  let get_value = |var| model.get_obj_attr(attr::X, &var);
  let x0 = get_value(x0)?;
  let x1 = get_value(x1)?;
  let x2 = get_value(x2)?;

  assert_eq!(x0.round() as isize, 0);
  assert_eq!(x1.round() as isize, 1);
  assert_eq!(x2.round() as isize, 2);
  Ok(())
}
