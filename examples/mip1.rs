extern crate gurobi;

fn main() {
    let env = gurobi::Env::new("mip1.log").unwrap();
}
