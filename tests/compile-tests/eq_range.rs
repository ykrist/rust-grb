use gurobi::*;
mod utils;

fn main() -> Result<()> {
  let _g = gag::Gag::stdout();
  create_model!(_g, m, x, y, z);
  c!(x + y + 2*z + 1 in 0..=1);
  c!(x + y + 2*z + 1 in ..=1);
  Ok(())
}
