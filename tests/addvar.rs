use gurobi::*;

fn main() -> Result<()> {
  let q = 3.9;
  let mut model = Model::new("")?;
  add_var!(model, Binary, name: "x", obj: q/5.0)?;
  add_var!(model, Binary, obj: q, name: "x", bounds: 0..10)?;
  add_var!(model, Binary, bounds: 0..10, name: "x", obj: q)?;
  add_binvar!(model)?;
  add_ctsvar!(model)?;
  add_intvar!(model)?;
  add_ctsvar!(model, name: "x1", bounds: ..).unwrap();
  add_intvar!(model, name: "x2", bounds: ..).unwrap();
  Ok(())
}