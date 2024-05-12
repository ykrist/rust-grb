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
        let raw_f = f?.file_name();
        let Some(f) = raw_f.to_str() else { continue };
        let Some(f) = f.strip_prefix("libgurobi") else {
            continue;
        };
        let Some(version) = f.strip_suffix(".so") else {
            continue;
        };
        if version.bytes().all(|b| (b'0'..=b'9').contains(&b)) {
            return Ok(format!("gurobi{version}"));
        }
    }
    anyhow::bail!("no libgurobi*.so matches found in {}", path.display())
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
