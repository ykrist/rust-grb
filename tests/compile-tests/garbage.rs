use grb::*;
mod utils;

fn main() -> Result<()> {
  create_model!(_g, m, x, y, z);

  c!(x + y + 2*z + 1 <= 0 in 1..0);
  c!(z + y in 0..z);
  c!(z + y in (x + 1)..);
  c!(y in x..z);

  Ok(())
}
