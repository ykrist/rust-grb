extern crate gurobi;

fn main() {
  let env = gurobi::Env::new("qp1.log").unwrap();
  
  // create an empty model. 
  let mut model = env.new_model("qp1", gurobi::Maximize).unwrap();

  // add & integrate new variables.
  model.add_cvar("x", 0.0, 0.0, 1.0).unwrap();
  model.add_cvar("y", 0.0, 0.0, 1.0).unwrap();
  model.add_cvar("z", 0.0, 0.0, 1.0).unwrap();
  model.update().unwrap();

  // set objective funtion:
  //   f(x,y,z) = x*x + x*y + y*y + y*z + z*z + 2*x
  //            = f_q(x,y,z) + f_l(x,y,z)
  //            
  // 1. add quad term: f_q = x*x + x*y + y*y + y*z + z*z
  model.add_qpterms(&[0, 0, 1, 1, 2], &[0, 1, 1, 2, 2], &[1., 1., 1., 1., 1.])
    .unwrap();
  // 2. add linear term: f_l = 2*x
  model.set_double_array(gurobi::DoubleAttr::Obj, 0, &[1.0, 0.0, 0.0]).unwrap();

  // add linear constraints

  //  g1(x,y,z) = x + 2*y + 3*z >= 4
  model.add_constr("c0", &[0, 1, 2], &[1., 2., 3.], gurobi::Greater, 4.0)
    .unwrap();

  //  g2(x,y,z) = x + y >= 2
  model.add_constr("c1", &[0, 1], &[1., 1.], gurobi::Greater, 1.0).unwrap();

  // optimize the model.
  model.optimize().unwrap();

  // write the model to file.
  model.write("qp.lp").unwrap();
  model.write("qp.sol").unwrap();
 
  let status = model.get_int(gurobi::IntAttr::Status).unwrap();
  assert_eq!(status, 2);
}
