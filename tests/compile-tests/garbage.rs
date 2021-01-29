use gurobi::*;
mod utils;

fn main() -> Result<()> {
  let _g = gag::Gag::stdout();
  create_model!(_g, m, x, y, z);

  c!(x + y + 2*z + 1 <= 0 in 1..0);
  c!(z + y in 0..z);
  c!(y in x..z);

  Ok(())
}
