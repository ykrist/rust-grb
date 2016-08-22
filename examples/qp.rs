extern crate gurobi;
use gurobi::*;

fn main() {
  let env = Env::new("qp.log").unwrap();

  // create an empty model.
  let mut model = env.new_model("qp").unwrap();

  // add & integrate new variables.
  let x = model.add_var("x", Continuous(0.0, 1.0)).unwrap();
  let y = model.add_var("y", Continuous(0.0, 1.0)).unwrap();
  let z = model.add_var("z", Continuous(0.0, 1.0)).unwrap();
  model.update().unwrap();

  // set objective funtion:
  //   f(x,y,z) = x*x + x*y + y*y + y*z + z*z + 2*x
  model.set_objective(&x * &x + &x * &y + &y * &y + &y * &z + &z * &z + 2.0 * &x, Maximize).unwrap();

  // add linear constraints

  //  g1(x,y,z) = x + 2*y + 3*z >= 4
  let _ = model.add_constr("c0", &x + 2.0 * &y + 3.0 * &z, Greater, 4.0).unwrap();

  //  g2(x,y,z) = x + y >= 2
  let _ = model.add_constr("c1", &x + &y, Greater, 1.0).unwrap();

  // optimize the model.
  model.optimize().unwrap();

  // write the model to file.
  model.write("qp.lp").unwrap();
  model.write("qp.sol").unwrap();

  let status = model.get(attr::Status).unwrap();
  assert_eq!(status, 2);
}
