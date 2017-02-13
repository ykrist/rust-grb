use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn gurobi_home() -> String {
  let var = env::var("GUROBI_HOME").expect("failed to retrieve the value of GUROBI_HOME");
  if !Path::new(&var).exists() {
    panic!("GUROBI_HOME is invalid path");
  }
  var
}

fn append_path(addpath: PathBuf) {
  let path = env::var_os("PATH").expect("failed to retrieve the value of PATH");
  let mut paths: Vec<_> = env::split_paths(&path).collect();

  paths.push(addpath);

  let new_path = env::join_paths(paths).unwrap();
  env::set_var("PATH", &new_path);
}

fn get_version_triple() -> (i32, i32, i32) {
  append_path(PathBuf::from(gurobi_home()).join("bin"));

  let output = Command::new("gurobi_cl").arg("--version").output().expect("failed to execute gurobi_cl");
  let verno: Vec<_> = String::from_utf8_lossy(&output.stdout)
    .into_owned()
    .split_whitespace()
    .nth(3)
    .expect("failed to get version string")
    .split(".")
    .map(|s| s.parse().expect("failed to parse version tuple"))
    .collect();

  (verno[0], verno[1], verno[2])
}

fn main() {
  let gurobi_home = gurobi_home();
  let libpath: PathBuf = PathBuf::from(gurobi_home).join("lib");

  let (major, minor, _) = get_version_triple();
  let libname = format!("gurobi{}{}", major, minor);

  println!("cargo:rustc-link-search=native={}", libpath.display());
  println!("cargo:rustc-link-lib={}", libname);
}
