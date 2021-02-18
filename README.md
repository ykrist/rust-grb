# grb ![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/ykrist/rust-grb?sort=semver) ![](https://img.shields.io/crates/v/grb.svg) ![](https://img.shields.io/docsrs/grb)

This crate provides Rust bindings for Gurobi Optimizer.  It currently requires Gurobi 9.0 or higher.

This library started as fork of the [`gurobi`](https://github.com/ubnt-intrepid/rust-gurobi) which appears to be no longer maintained.
It has since undergone a number of fundamental API changes. 

## Installing and Linking

* Before using this crate, you should install Gurobi and obtain a [license](http://www.gurobi.com/downloads/licenses/license-center).

* **Linking**: Make sure that the environment variable `GUROBI_HOME` is set to the installation path of Gurobi
  (like `C:\gurobi652\win64` or `/opt/gurobi652/linux64`).  If you are using the Conda package
  from the Gurobi channel, the build script will fall back to `GUROBI_HOME=${CONDA_PREFIX}`, so you
  should not set `GUROBI_HOME`.

## Documentation
Docs can be found on [docs.rs](https://docs.rs/grb/)

## License
This software is released under the [MIT license](LICENSE).
