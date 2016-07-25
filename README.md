# rust-gurobi

An unofficial Rust API for Gurobi optimizer.

**Notices**

* This wrapper library is not officially supported by Gurobi.
* Too many works have not finished yet.

## Usage

Fix your `Cargo.toml` as follows:

```toml
[dependencies]
gurobi = { git = "https://github.com/ys-nuem/rust-gurobi.git" }
```

## Example

```rust
extern crate gurobi;

fn main() {
  let env = gurobi::Env::new("logfile.log").unwrap();

  // create an empty model which associated with `env`:
  let mut model = env.new_model("model1", gurobi::Maximize).unwrap();

  // add decision variables.
  model.add_bvar("x", 0.0).unwrap();
  model.add_cbar("y", 0.0, -10,0, 10.0).unwrap();
  // ...

  // integrate all the variables into the model.
  model.update().unwrap();

  // add a linear constraint
  model.add_constr("c0", &[0, 1, 2], &[1.0, -1.0, 2.0], gurobi::Equal, 0.0).unwrap();
  // ...
 
  model.optimize().unwrap(); 
}
```

## License

Copyright (c) 2016, Yusuke Sasaki

This software is released under the MIT license, see [LICENSE](LICENSE).
