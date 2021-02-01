use gurobi::*;
mod utils;

fn main() -> Result<()> {
  create_model!(_g, m);
  add_var!(m, Binary, name: "x", name: "y")?;
  add_var!(m, Binary, name: "x", unknown: 30)?;
  add_var!(m, Binary, name: "x", bounds: ..=10)?;
  add_var!(m, Binary, name="x")?;
  add_var!(m)?;
  Ok(())
}
