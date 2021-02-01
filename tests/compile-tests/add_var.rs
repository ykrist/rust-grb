use gurobi::*;
mod utils;

fn main() -> Result<()> {
  create_model!(_g, m);
  add_var!(m, Binary)?;
  add_var!(m, Continuous, name: "x")?;
  add_var!(m, Continuous, obj: 1, name: "y")?;
  add_var!(m, Continuous, name: "y", obj: 1)?;
  add_var!(m, Binary, name: "x", bounds: ..100)?;
  add_var!(m, Binary, bounds: 0..30, name: "egg", obj: 4)?;
  Ok(())
}
