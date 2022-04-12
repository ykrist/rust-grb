# grb ![GitHub tag (latest SemVer)](https://img.shields.io/github/v/tag/ykrist/rust-grb?sort=semver) ![](https://img.shields.io/crates/v/grb.svg) ![](https://img.shields.io/docsrs/grb)

This crate provides Rust bindings for Gurobi Optimizer.  It currently requires Gurobi 9.0 or higher.

This library started as fork of the [`gurobi`](https://github.com/ubnt-intrepid/rust-gurobi) which appears to be no longer maintained.  It has since undergone a number of fundamental API changes.

This crate supports Gurobi 9.5

## Installing and Linking

Before using this crate, you should install Gurobi and obtain a [license](http://www.gurobi.com/downloads/licenses/license-center).

### Building
In this section, it is assumed Gurobi is install at `/opt/gurobi/linux64`.

It is recommended you use the environment variables for you system's linker to ensure Gurobi can be found.
For example, on Linux systems this can be done by appending the path to the `lib` subfolder of the gurobi installation to `LIBRARY_PATH`.   For example, put

```base
export LIBRARY_PATH="LIBRARY_PATH:/opt/gurobi/linux64/lib"
```

in your `~/.profile` file.  You can also set this in a `PROJECT/.cargo/config.toml` file on per project basis (see the `[env]` [section](https://doc.rust-lang.org/cargo/reference/config.html)).

The other option is to set the environment variable `GUROBI_HOME` set to the installation path of Gurobi
(like eg `/opt/gurobi911/linux64`).  If you are using the Conda package from the Gurobi channel, you can set `GUROBI_HOME=${CONDA_PREFIX}` from within your Conda environment.

### Running
When running the compiled binaries or running tests, you may get
```bash
error while loading shared libraries: libgurobi95.so: cannot open shared object file: No such file or directory
```
In this case, you need to set the `LD_LIBRARY_PATH` (on Windows I believe this is called `PATH`) environment variable or embed the path to `libgurobi95.so` in the [rpath](https://en.wikipedia.org/wiki/Rpath) by supplying the appropriate linker flags in `RUSTFLAGS`.

For the example below, suppose Gurobi is in the path `/opt/gurobi/linux64/lib/libgurobi95.so`.  You set `LD_LIBRARY_PATH` in the same manner as the `LIBRARY_PATH` variable, in your `~/.profile`:

```base
export LD_LIBRARY_PATH="LD_LIBRARY_PATH:/opt/gurobi/linux64/lib"
```

## Documentation
Docs can be found on [docs.rs](https://docs.rs/grb/)

## License
This software is released under the [MIT license](LICENSE).
