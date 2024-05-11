#![deny(deprecated)]

use grb::callback::*;
use grb::prelude::*;

fn callback(w: Where) -> CbResult {
    match w {
        Where::MIPSol(ctx) => {
            ctx.add_cut(c!(0 == 0))?;
        }
        Where::MIPNode(ctx) => {
            ctx.add_cut(c!(0 == 0))?;
        }
        _ => {}
    }
    Ok(())
}

fn main() {}
