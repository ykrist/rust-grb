extern crate gurobi;
use gurobi::Attr;

fn main() {
  let env = gurobi::Env::new("qp.log").unwrap();
  
  // create an empty model. 
  let mut model = env.new_model("qp").unwrap();

  // add & integrate new variables.
  let x = model.add_var("x", gurobi::Continuous(0.0, 1.0), 0.0).unwrap();
  let y = model.add_var("y", gurobi::Continuous(0.0, 1.0), 0.0).unwrap();
  let z = model.add_var("z", gurobi::Continuous(0.0, 1.0), 0.0).unwrap();
  model.update().unwrap();

  // set objective funtion:
  //   f(x,y,z) = x*x + x*y + y*y + y*z + z*z + 2*x
  //            = f_q(x,y,z) + f_l(x,y,z)
  // quad term: f_q = x*x + x*y + y*y + y*z + z*z
  // linear term: f_l = 2*x
  model.set_objective(&[0,1,2], &[1.0, 0.0, 0.0],
      &[0, 0, 1, 1, 2], &[0, 1, 1, 2, 2], &[1., 1., 1., 1., 1.], gurobi::ModelSense::Maximize)
    .unwrap();

  // add linear constraints

  //  g1(x,y,z) = x + 2*y + 3*z >= 4
  let c0 = model.add_constr("c0", &[0, 1, 2], &[1., 2., 3.], gurobi::Greater, 4.0)
    .unwrap();

  //  g2(x,y,z) = x + y >= 2
  let c1 = model.add_constr("c1", &[0, 1], &[1., 1.], gurobi::Greater, 1.0).unwrap();

  // optimize the model.
  model.optimize().unwrap();

  // write the model to file.
  model.write("qp.lp").unwrap();
  model.write("qp.sol").unwrap();
 
  let status = model.get(gurobi::IntAttr::Status).unwrap();
  assert_eq!(status, 2);
}
