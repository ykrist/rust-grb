#![deny(deprecated)]

use grb::prelude::*;
use grb::callback::*;

fn callback(w: Where) -> CbResult {
  match w {
    Where::MIPSol(ctx) => {
      if let Where::MIPSol(ctx) = w {
        ctx.add_cut(c!(0 == 0))?;
      }
    },
    Where::MIPNode(ctx) => {
      ctx.add_cut(c!(0 == 0))?;
    }
  }
  Ok(())
}

fn main() {}

