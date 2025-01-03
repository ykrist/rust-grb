use grb::callback::*;
use grb::prelude::*;
use std::collections::HashSet;

mod common;
use common::*;

#[derive(Default)]
struct Cb {
    phases: HashSet<MipPhase>,
}

impl Callback for Cb {
    fn callback(&mut self, w: Where) -> CbResult {
        if let Where::MIP(mut ctx) = w {
            let phase = ctx.phase()?;
            println!("phase = {phase:?}");
            if phase == MipPhase::NoRel {
                ctx.proceed();
            }
            self.phases.insert(phase);
        };
        Ok(())
    }
}

#[test]
fn main() -> anyhow::Result<()> {
    let mut model = test_instance("sing44")?;
    model.set_param(param::TimeLimit, 10.0)?;
    model.set_param(param::NoRelHeurTime, 3.0)?;
    let mut cb = Cb::default();
    model.optimize_with_callback(&mut cb)?;
    println!("{:?}", &cb.phases);
    assert!(cb.phases.contains(&MipPhase::NoRel));
    assert!(cb.phases.contains(&MipPhase::Search));
    Ok(())
}
