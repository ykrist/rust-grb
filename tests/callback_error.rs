use grb::prelude::*;
use grb::callback::{CbResult, Callback, Where};

struct ErrorCb {}
struct PanicCb {}

impl Callback for ErrorCb {
  fn callback(&mut self, _: Where) -> CbResult {
    anyhow::bail!("ruh roh")
  }
}

impl Callback for PanicCb {
  fn callback(&mut self, _: Where) -> CbResult {
    panic!("ruh roh")
  }
}

fn run(mut cb: impl Callback) -> grb::Result<()> {
  let mut m = Model::new("")?;
  add_ctsvar!(m)?;
  let result = m.optimize_with_callback(&mut cb);
  match result {
    Err(grb::Error::FromAPI(_, 10011)) => {},
    Err(e) => panic!("unexpected error: {}", e),
    Ok(()) => panic!("expected error"),
  }
  m.write("callback_error.lp")?;
  Ok(())
}

#[test]
fn cb_panics() -> grb::Result<()> {
  run(PanicCb{})
}

#[test]
fn cb_errors() -> grb::Result<()> {
  run(ErrorCb{})
}
