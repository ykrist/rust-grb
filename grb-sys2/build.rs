use std::env;
use std::path::PathBuf;
use std::process::Command;

fn try_gurobi_home() -> Result<PathBuf, String> {
  let path = env::var("GUROBI_HOME")
    .map_err(|_| "failed to retrieve the value of GUROBI_HOME".to_string())?;
  Ok(PathBuf::from(path))
}

fn try_conda_env() -> Result<PathBuf, String> {
  let path = env::var("CONDA_PREFIX")
    .map_err(|_| "failed to retrieve the value of CONDA_PREFIX".to_string())?;
  Ok(PathBuf::from(path))
}


fn locate_gurobi() -> PathBuf {
  for f in &[try_gurobi_home, try_conda_env] {
    match f()  {
      Ok(path) => {
        if !path.exists() {
          eprintln!("path {:?} doesn't exist, ignoring", path.into_os_string());
          continue;
        }
        let path = path.canonicalize().unwrap();
        eprintln!("found Gurobi home: {:?}", &path);
        return path;
      }
      Err(e) => {
        eprintln!("{}", e);
      }
    }
  }
  panic!("Unable to find Gurobi installation, try setting GUROBI_HOME")
}

fn get_gurobi_cl(gurobi_home: &PathBuf) -> PathBuf {
  let mut gurobi_cl = gurobi_home.clone();
  gurobi_cl.push("bin");
  gurobi_cl.push("gurobi_cl");
  gurobi_cl
}

fn get_version_triple(gurobi_home: &PathBuf) -> (i32, i32, i32) {
  let gurobi_cl = get_gurobi_cl(gurobi_home);

  let output = Command::new(&gurobi_cl).arg("--version").output()
    .unwrap_or_else(|_| panic!(format!("failed to execute {:?}", &gurobi_cl)));
  let verno: Vec<_> = String::from_utf8_lossy(&output.stdout)
    .into_owned()
    .split_whitespace()
    .nth(3)
    .expect("failed to get version string")
    .split('.')
    .map(|s| s.parse().expect("failed to parse version tuple"))
    .collect();
  (verno[0], verno[1], verno[2])
}

fn main() {
  println!("cargo:rerun-if-env-changed=DOCS_RS");
  if let Ok(_) = std::env::var("DOCS_RS") {
    return;
  }

  let gurobi_home = locate_gurobi();
  let libpath: PathBuf = gurobi_home.join("lib");
  let (major, minor, _) = get_version_triple(&gurobi_home);
  let libname = format!("gurobi{}{}", major, minor);

  println!("cargo:rustc-link-search=native={}", libpath.display());
  println!("cargo:rustc-link-lib={}", libname);
}
