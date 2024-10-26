use grb::callback::*;

mod common;
use common::*;

fn callback(w: Where) -> CbResult {
    if let Where::IIS(ctx) = w {
        println!("min constraints = {}", ctx.constr_min()?);

        if ctx.runtime()? > 2.0 {
            ctx.terminate();
        }
    }

    Ok(())
}

#[test]
#[ignore] // TODO: not in gzip format
fn main() -> anyhow::Result<()> {
    let mut model = test_instance("neos859080")?;

    model.optimize()?;
    model.compute_iis_with_callback(&mut callback)?;

    Ok(())
}
