use grb::prelude::*;
mod utils;

fn main() -> grb::Result<()> {
  create_model!(_g, m, x, y, z);
  c!(x + y == 2 == 1 - z);
  c!(x + y - (4 >= 1 - z));
  c!(x + y >= 1 - z >= 43);
  Ok(())
}
