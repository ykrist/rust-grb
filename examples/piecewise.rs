// Copyright (c) 2016 Yusuke Sasaki
//
// This software is released under the MIT License.
// See http://opensource.org/licenses/mit-license.php or <LICENSE>.

use gurobi::*;

#[allow(clippy::many_single_char_names)]
fn main() {
  let env = Env::new("piecewise.log").unwrap();
  let mut model = Model::with_env("piecewise", &env).unwrap();

  // Add variables.
  let x = model.add_var("x", Continuous, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  let y = model.add_var("y", Continuous, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  let z = model.add_var("z", Continuous, 0.0, 0.0, 1.0, &[], &[]).unwrap();
  model.update().unwrap();

  // Add constraints.
  model.add_constr("c0", c!(x + 2* y + 3*z <= 4)).unwrap();
  model.add_constr("c1", c!(x + y >= 1)).unwrap();

  // Set `convex` objective function:
  //  minimize f(x) - y + g(z)
  //    where f(x) = exp(-x),  g(z) = 2 z^2 - 4 z

  let f = |x: f64| (-x).exp();
  let g = |z: f64| 2.0 * z * z - 4.0 * z;

  let n_points: usize = 101;
  let (lb, ub) = (0.0, 1.0);

  let pt_u: Vec<f64> = (0..n_points).map(|i| lb + (ub - lb) * (i as f64) / ((n_points as f64) - 1.0)).collect();
  let pt_f: Vec<f64> = pt_u.iter().map(|&x| f(x)).collect();
  let pt_g: Vec<f64> = pt_u.iter().map(|&z| g(z)).collect();

  model.set_pwl_obj(&x, pt_u.as_slice(), pt_f.as_slice()).unwrap();
  model.set_pwl_obj(&z, pt_u.as_slice(), pt_g.as_slice()).unwrap();
  model.set_obj_attr(attr::Obj, &y, -1.0).unwrap();

  optimize_and_print_status(&mut model).unwrap();

  // Negate piecewise-linear objective function for x.
  // And then the objective function becomes non-convex.
  let pt_f: Vec<f64> = pt_f.into_iter().map(|f| -f).collect();
  model.set_pwl_obj(&x, pt_u.as_slice(), pt_f.as_slice()).unwrap();

  optimize_and_print_status(&mut model).unwrap();
}

fn optimize_and_print_status(model: &mut Model) -> Result<()> {
  model.optimize()?;

  println!("IsMIP = {}", model.get_attr(attr::IsMIP)? != 0);
  let vars = model.get_vars().unwrap();
  for v in vars {
    let vname = model.get_obj_attr(attr::VarName, v)?;
    let x = model.get_obj_attr(attr::X, v)?;
    println!("{} = {}", vname, x);
  }
  println!("Obj = {}\n", model.get_attr(attr::ObjVal)?);
  Ok(())
}
