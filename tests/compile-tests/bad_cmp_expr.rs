use grb::prelude::*;
mod utils;

fn main() -> grb::Result<()> {
    create_model!(_g, m, x, y, z);
    c!(x + y == );
    c!(==);
    c!(x);
    c!(>=3);
    Ok(())
}
