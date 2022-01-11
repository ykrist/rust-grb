use std::collections::HashSet;
use grb::callback::*;
use grb::prelude::*;

#[derive(Default)]
struct Cb {
    phases: HashSet<MipPhase>
}


impl Callback for Cb {
    fn callback(&mut self, mut w: Where) -> CbResult {
        match w {
            Where::MIP(mut ctx) => {
                let phase = ctx.phase()?;
                println!("phase = {:?}", phase);
                if phase == MipPhase::NoRel {
                    ctx.proceed();
                }
                self.phases.insert(phase);
            }
            _ => {}
        };
        Ok(())
    }
}

#[test]
fn main() -> anyhow::Result<()> {
    let mut model = Model::from_file(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/sing44.mps.gz"))?;
    model.set_param(param::TimeLimit, 10.0)?;
    model.set_param(param::NoRelHeurTime, 3.0)?;
    let mut cb = Cb::default();
    model.optimize_with_callback(&mut cb)?;
    // let model = Model::from_file(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/50v-10.mps"))?;
    println!("{:?}", &cb.phases);
    assert!(cb.phases.contains(&MipPhase::NoRel));
    assert!(cb.phases.contains(&MipPhase::Search));
    Ok(())
}