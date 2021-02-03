use grb::prelude::*;
mod utils;

fn main() -> grb::Result<()> {
  create_model!(_g, m, x, y, z);
  c!(x + y == 1 - z);
  c!(x + y >= 1 - z);
  c!(x + y <= 1 - z);

  c!(x - y in 0..1);
  c!(x in ..1);
  c!(y - x in ..);
  c!(x in -2.3..1);
  Ok(())
}
