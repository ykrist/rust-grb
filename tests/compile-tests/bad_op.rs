use gurobi::*;
mod utils;

fn main() -> Result<()> {
  let _g = gag::Gag::stdout();
  create_model!(_g, m, x, y, z);
  c!(x + y + 2*z + 1);
  c!(2*z);
  c!(2/z);
  Ok(())
}
