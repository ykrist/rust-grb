use std::env;
use std::fmt::{self, Display};
use std::path::PathBuf;

const LIB_NAME: &'static str = "gurobi95";

#[derive(Debug)]
enum Error {
  DoesNotExist(PathBuf),
  GurobiHomeNotGiven,
}

impl Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::DoesNotExist(p) => f.write_fmt(format_args!(
        "GUROBI_HOME is set to {:?} but {:?} does not exist (or is not a directory)",
        p.parent().unwrap(),
        p
      )),
      Error::GurobiHomeNotGiven => f.write_str("GUROBI_HOME not set"),
    }
  }
}

impl std::error::Error for Error {}

fn try_gurobi_home() -> Result<PathBuf, Error> {
  let path = env::var("GUROBI_HOME")
    .map_err(|_| Error::GurobiHomeNotGiven)?;

  // You cannot unset environment variables in the config.toml so this is the next best thing.
  if path.is_empty() {
    return Err(Error::GurobiHomeNotGiven)
  }

  let mut path : PathBuf = path.into();

  if cfg!(target_os = "windows") {
    path.push("bin");
  } else {
    path.push("lib");
  }

  path.canonicalize().map_err(|_| Error::DoesNotExist(path))
}

fn main() {
  match try_gurobi_home() {
    Ok(path) => println!("cargo:rustc-link-search=native={}", path.display()),
    Err(Error::GurobiHomeNotGiven) => eprintln!("GUROBI_HOME env var not set"),
    Err(e) => println!("cargo:warning={}", e)
  }
  println!("cargo:rustc-link-lib=dylib={}", LIB_NAME);
}
