use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{self, Error, ErrorKind};

fn gurobi_home() -> io::Result<String> {
  env::var("GUROBI_HOME")
    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
    .and_then(|p| if Path::new(p.as_str()).exists() {
      Ok(p)
    } else {
      Err(Error::new(ErrorKind::Other, "".to_owned()))
    })
}

fn append_path(addpath: &str) {
  if let Some(path) = env::var_os("PATH") {
    let mut paths = env::split_paths(&path).collect::<Vec<_>>();
    paths.push(PathBuf::from(addpath));
    let new_path = env::join_paths(paths).unwrap();
    env::set_var("PATH", &new_path);
  }
}

fn get_version_triple() -> (i32, i32, i32) {
  let mut binpath = PathBuf::from(gurobi_home().unwrap());
  binpath.push("bin");
  append_path(binpath.to_str().unwrap());

  let output = Command::new("gurobi_cl").arg("--version").output().expect("failed to execute gurobi_cl");
  let verno: Vec<_> = String::from_utf8(output.stdout)
    .unwrap()
    .split_whitespace()
    .nth(3)
    .unwrap()
    .split(".")
    .map(|s| s.parse().unwrap())
    .collect();

  (verno[0], verno[1], verno[2])
}

fn main() {
  if let Ok(gurobi_home) = gurobi_home() {
    let mut libpath: PathBuf = PathBuf::from(gurobi_home);
    libpath.push("lib");
    let libpath = libpath.to_str().unwrap();

    let (major, minor, _) = get_version_triple();
    let libname = format!("gurobi{}{}", major, minor);

    println!("cargo:rustc-link-search=native={}", libpath);
    println!("cargo:rustc-link-lib={}", libname);
  }
}
