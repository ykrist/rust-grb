use std::io::BufRead;

use grb::prelude::*;
use grb::callback::*;

struct Cb {
    feas_soln: Vec<(Var, f64)>,
    infeas_soln: Vec<(Var, f64)>,
    tests: [bool; 4]
}

impl Cb {
    fn new(feas_soln: Vec<(Var, f64)>, infeas_soln: Vec<(Var, f64)>) -> Self {
        Cb { feas_soln, infeas_soln, tests: [false; 4] }
    }

    fn check_once(&mut self, idx: usize) -> bool {
        if self.tests[idx] {
            return false
        }
        println!("running check {}", idx);
        self.tests[idx] = true;
        true
    }
}

impl Callback for Cb {
    fn callback(&mut self, w: Where) -> CbResult {
        match w {
            Where::MIPNode(ctx) => {
                println!("MIPNODE");
                if ctx.status()? != Status::Infeasible {
                    if self.check_once(0) {
                        let x = ctx.set_solution(self.feas_soln.iter().copied())?;
                        assert!(x.is_some());
                    } else if self.check_once(1) {
                        let x = ctx.set_solution(self.infeas_soln.iter().copied())?;
                        assert_eq!(x, None);
                    } else {
                        if self.tests.iter().all(|x| *x) {
                            ctx.terminate()
                        }
                    }
                }
            },
            Where::MIP(ctx) => {
                println!("MIP");
                if self.check_once(2) {
                    let x = ctx.set_solution(self.feas_soln.iter().copied())?;
                    assert_eq!(x, None);
                } else if self.check_once(3) {
                    let x = ctx.set_solution(self.infeas_soln.iter().copied())?;
                    assert_eq!(x, None);
                }
            },
            _ => {},
        }
        Ok(())
    }
}

const MODEL_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/traininstance2.mps.gz");
const SOLN_FILE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/traininstance2.sol");

fn load_soln(m: &mut Model) -> anyhow::Result<Vec<(Var, f64)>> {
    let solfile = std::io::BufReader::new(std::fs::File::open(SOLN_FILE)?).lines().skip(2);
    let mut sol = Vec::new();

    for line in solfile {
        let l = line?;
        // dbg!(&l);
        let mut line = l.split_whitespace();
        let varname = line.next().unwrap();
        let val: f64 = line.next().unwrap().parse()?;
        let var = m.get_var_by_name(varname)?.unwrap();
        sol.push((var, val));
    }
    Ok(sol)
}

#[test]
fn main() -> anyhow::Result<()> {
    let mut m = Model::from_file(MODEL_FILE)?;
    m.set_param(param::Seed, 1337)?;
    m.set_param(param::Presolve, 0)?;
    m.set_param(param::Threads, 1)?;
    m.set_param(param::Heuristics, 0.0)?;

    let infeas_soln = m.get_vars()?
        .iter()
        .copied()
        .zip(std::iter::repeat(-1.0))
        .collect();
    let feas_soln = load_soln(&mut m)?;
    
    let mut cb = Cb::new(feas_soln, infeas_soln);
    m.optimize_with_callback(&mut cb)?;
    
    Ok(())
}