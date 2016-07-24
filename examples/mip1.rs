extern crate gurobi;

fn main() {
    let env = gurobi::Env::new("mip1.log").unwrap();
    let mut model = gurobi::Model::new(&env).unwrap();

    model.optimize().unwrap();
}
