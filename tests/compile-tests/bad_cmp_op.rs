use grb::prelude::*;
mod utils;

fn main() -> grb::Result<()> {
  create_model!(_g, m, x, y, z);
  c!(x + y != 1 - z);
  c!(x + y < 1 - z);
  c!(x + y > 1 - z);
  Ok(())
}
