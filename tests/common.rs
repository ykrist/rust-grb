use grb::prelude::*;
use std::io::BufRead;

#[allow(dead_code)]
pub fn test_instance(name: &str) -> grb::Result<Model> {
    let path = format!("{}/tests/data/{}.mps.gz", env!("CARGO_MANIFEST_DIR"), name);
    let mut m = Model::from_file(path)?;
    m.set_param(param::Threads, 1)?;
    Ok(m)
}

#[allow(dead_code)]
pub fn load_soln(m: &mut Model, name: &str) -> anyhow::Result<Vec<(Var, f64)>> {
    let path = format!("{}/tests/data/{}.sol", env!("CARGO_MANIFEST_DIR"), name);
    let solfile = std::io::BufReader::new(std::fs::File::open(path)?).lines().skip(2);
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