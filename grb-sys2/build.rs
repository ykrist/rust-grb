use std::env;
use std::path::PathBuf;

use anyhow::Context;

const DEFAULT_GUROBI_LIBNAME: &str = "gurobi100";

fn try_gurobi_home() -> anyhow::Result<PathBuf> {
  let path = env::var("GUROBI_HOME").context("unable to retrieve value of GUROBI_HOME")?;

  // You cannot unset environment variables in the config.toml so this is the next best thing.

  if path.is_empty() {
    anyhow::bail!("GUROBI_HOME is empty")
  }

  let mut path: PathBuf = path.into();

  if cfg!(target_os = "windows") {
    path.push("bin");
  } else {
    path.push("lib");
  }

  path.canonicalize().with_context(|| {
    format!(
      "GUROBI_HOME points to {} which doesn't exist",
      path.display()
    )
  })
}

fn try_guess_libname(path: &PathBuf) -> anyhow::Result<String> {
  for f in std::fs::read_dir(path)? {
    let f = f?.file_name();
    if !f.is_ascii() {
      continue;
    }
    let f = f.as_encoded_bytes();
    if f.starts_with(b"lib") && f.ends_with(b".so") && !f.ends_with(b"_light.so") {
      return Ok(String::from_utf8(f[3..f.len() - 3].to_vec()).unwrap());
    }
  }
  anyhow::bail!("no lib*.so matches found in {}", path.display())
}

fn get_lib_name(gurobi_home: Option<&PathBuf>) -> String {
  if let Ok(value) = env::var("GUROBI_LIBNAME") {
    return value;
  }
  if cfg!(target_os = "linux") {
    match gurobi_home {
      Some(p) => match try_guess_libname(p) {
        Ok(name) => {
          println!("inferring libname {name:?}, set GUROBI_LIBNAME to override");
          return name;
        }
        Err(e) => {
          println!("cargo:warning={}", e);
        }
      },
      None => {}
    }
  }
  println!("cargo:warning=using default libname {DEFAULT_GUROBI_LIBNAME:?}, set GUROBI_LIBNAME to override");
  String::from(DEFAULT_GUROBI_LIBNAME)
}

fn main() {
  println!("cargo:rerun-if-env-changed=GUROBI_HOME");
  println!("cargo:rerun-if-env-changed=GUROBI_LIBNAME");

  let gurobi_home = try_gurobi_home();
  match gurobi_home.as_ref() {
    Ok(path) => {
      println!("cargo:rustc-link-search=native={}", path.display())
    }
    Err(e) => println!("cargo:warning={:#}", e),
  }
  let libname = get_lib_name(gurobi_home.as_ref().ok());
  println!("cargo:rustc-link-lib=dylib={}", libname);
}
