
fn print_gurobi_confs() {
  use std::env;
  use std::path::{Path, PathBuf};

  let gurobi_home = env::var("GUROBI_HOME").unwrap();
  assert!(Path::new(gurobi_home.as_str()).exists());

  let mut libpath: PathBuf = PathBuf::from(gurobi_home);
  libpath.push("lib");

  println!("cargo:rustc-link-search=native={}", libpath.to_str().unwrap());
}

fn main() {
  print_gurobi_confs();

  println!("cargo:rustc-link-lib={}", "gurobi65")
}
