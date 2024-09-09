// false positives due this being a lil hacky
/// This file exists here so that we can write tests for the build script
/// without having to publish a separate crate.
use anyhow::Context;
use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug)]
struct GurobiLibAffixes {
    prefix: &'static str,
    suffix: &'static str,
}

impl GurobiLibAffixes {
    fn match_path_for_libname(&self, filename: &OsStr) -> Option<String> {
        let filename = filename.to_str()?;
        let version = filename
            .strip_prefix(self.prefix)?
            .strip_suffix(self.suffix)?;
        if version.bytes().all(|b| b.is_ascii_digit()) {
            Some(format!("gurobi{version}"))
        } else {
            None
        }
    }
}

const DEFAULT_AFFIXES: GurobiLibAffixes = GurobiLibAffixes {
    prefix: "libgurobi",
    suffix: ".so",
};

const WINDOWS_AFFIXES: GurobiLibAffixes = GurobiLibAffixes {
    prefix: "gurobi",
    suffix: ".lib",
};

fn match_path_for_libname(filename: &OsStr) -> Option<String> {
    let affixes = if cfg!(windows) {
        WINDOWS_AFFIXES
    } else {
        DEFAULT_AFFIXES
    };
    affixes.match_path_for_libname(filename)
}

fn try_guess_libname(gurobi_libpath: &Path) -> anyhow::Result<String> {
    for f in std::fs::read_dir(gurobi_libpath)? {
        let f = f?.file_name();
        if let Some(libname) = match_path_for_libname(&f) {
            return Ok(libname);
        }
    }
    anyhow::bail!(
        "Unable to infer libname: no Gurobi library file found in {}.",
        gurobi_libpath.display()
    )
}

fn get_libname(gurobi_libpath: Option<&Path>) -> anyhow::Result<String> {
    if let Ok(libname) = env::var("GUROBI_LIBNAME") {
        return Ok(libname);
    }
    if let Some(path) = gurobi_libpath {
        return try_guess_libname(path);
    }
    anyhow::bail!("Unable to infer Gurobi libname, set the environment variable GUROBI_LIBNAME=...")
}

fn try_guess_libpath() -> anyhow::Result<PathBuf> {
    let path = env::var("GUROBI_HOME").context("unable to retrieve value of GUROBI_HOME")?;

    // You cannot unset environment variables in the config.toml so this is the next best thing.
    if path.is_empty() {
        anyhow::bail!("GUROBI_HOME is set to empty string")
    }

    let mut path: PathBuf = path.into();
    path.push("lib");
    path.canonicalize().with_context(|| {
        format!(
            "GUROBI_HOME points to {} which doesn't exist",
            path.display()
        )
    })
}

pub fn main() {
    if cfg!(feature = "build_script_tests") {
        return;
    }
    println!("cargo:rerun-if-env-changed=GUROBI_HOME");
    println!("cargo:rerun-if-env-changed=GUROBI_LIBNAME");

    let libpath = try_guess_libpath();
    match libpath.as_ref() {
        Ok(path) => {
            println!("cargo:rustc-link-search=native={}", path.display())
        }
        Err(e) => println!("cargo:warning={e:#}"),
    }
    match get_libname(libpath.as_deref().ok()) {
        Ok(libname) => println!("cargo:rustc-link-lib=dylib={libname}"),
        Err(err) => {
            println!("cargo:warning=Error: {err:#}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_affixes_match_path() {
        for (name, expected) in [
            ("libgurobi990.so", Some("gurobi990")),
            ("libgurobi0.so", Some("gurobi0")),
            ("libgurobi990_light.so", None),
            ("libGurobiJni95.so", None),
        ] {
            let result = DEFAULT_AFFIXES.match_path_for_libname(name.as_ref());
            assert_eq!(result.as_deref(), expected);
        }
    }

    #[test]
    fn test_windows_affixes_match_path() {
        for (name, expected) in [
            ("gurobi110.lib", Some("gurobi110")),
            ("gurobi950.lib", Some("gurobi950")),
            ("gurobi1.lib", Some("gurobi1")),
            ("gurobi1300655506.lib", Some("gurobi1300655506")),
            ("gurobi-javadoc.jar", None),
            ("gurobi.jar", None),
            ("gurobi.py", None),
            ("gurobi110.netstandard20.dll", None),
            ("gurobi110.netstandard20.xml", None),
            ("gurobi_c++md2017.lib", None),
            ("gurobi_c++mdd2017.lib", None),
            ("gurobi_c++mt2017.lib", None),
            ("gurobi_c++mtd2017.lib", None),
            ("rootcert.pem", None),
        ] {
            let result = WINDOWS_AFFIXES.match_path_for_libname(name.as_ref());
            assert_eq!(result.as_deref(), expected);
        }
    }
}
