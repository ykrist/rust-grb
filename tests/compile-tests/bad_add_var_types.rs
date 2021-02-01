use gurobi::*;
mod utils;

fn main() -> Result<()> {
  create_model!(_g, m);
  add_var!(m, Binary, name: 0u8)?;
  Ok(())
}
