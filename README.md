# grb ![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/ykrist/rust-grb?sort=semver) ![](https://img.shields.io/crates/v/grb.svg) ![](https://img.shields.io/docsrs/grb)

This crate provides Rust bindings for Gurobi Optimizer.  It currently requires Gurobi 9.0 or higher.

This library started as fork of the [`gurobi`](https://github.com/ubnt-intrepid/rust-gurobi) which appears to be no longer maintained.
It has since undergone a number of fundamental API changes. 

## Installing and Linking

* Before using this crate, you should install Gurobi and obtain a [license](http://www.gurobi.com/downloads/licenses/license-center).

### Building
Make sure that the environment variable `GUROBI_HOME` is set to the installation path of Gurobi
(like `C:\gurobi911\win64` or `/opt/gurobi911/linux64`).  If you are using the Conda package
from the Gurobi channel, the build script will fall back to `GUROBI_HOME=${CONDA_PREFIX}`, so you
should not set `GUROBI_HOME`.

### Running
When running the compiled binaries or running tests, you may get 
```bash
error while loading shared libraries: libgurobi91.so: cannot open shared object file: No such file or directory
```
In this case, you need to set the `LD_LIBRARY_PATH` environment variable or embed the path to `libgurobi.so` in the
[rpath](https://en.wikipedia.org/wiki/Rpath) by supplying the appropriate linker flags in `RUSTFLAGS`.

For the examples below, suppose Gurobi is in the path `/opt/gurobi/linux64/libgurobi91.so`

## Method #1: `LD_LIBRARY_PATH`
```bash
cargo build
export LD_LIBRARY_PATH="/opt/gurobi/linux64/:${LD_LIBRARY_PATH}"
target/debug/my_program # or: cargo test
```
`LD_LIBRARY_PATH` will need to be set every time the binary is run in a new shell session.  If you use conda environments, 
this is the recommended approach (see [here](https://conda.io/projects/conda/en/latest/user-guide/tasks/manage-environments.html#setting-environment-variables)).


## Method #2: rpath
```bash
export RUSTFLAGS="-C link-args=-Wl,-rpath=/opt/gurobi/linux64/"
cargo build
target/debug/my_program  # or: cargo test
```
This has the advantage that you don't need to set anything when you want to run the binary in a new shell 
session. On the other hand, the path to Gurobi is baked into `my_program`, so it is no longer portable.


## Documentation
Docs can be found on [docs.rs](https://docs.rs/grb/)

## License
This software is released under the [MIT license](LICENSE).
