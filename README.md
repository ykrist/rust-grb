# rust-gurobi

[![](http://meritbadge.herokuapp.com/gurobi)](https://crates.io/crates/gurobi)

An unofficial Rust API for Gurobi optimizer.

* [Documentation](https://ys-nuem.github.io/rust-gurobi/doc/gurobi/)


**Notices**

* This wrapper library is not officially supported by Gurobi.
* Too many works have not finished yet.


## Installation

Fix your `Cargo.toml` as follows:

```toml
[dependencies]
gurobi = "0.1.7"
```


## Example

```rust
extern crate gurobi;
use gurobi::*;

fn main() {
  let env = Env::new("logfile.log").unwrap();

  // create an empty model which associated with `env`:
  let mut model = env.new_model("model1").unwrap();

  // add decision variables.
  let x = model.add_var("x", Binary).unwrap();
  let y = model.add_var("y", Continuous(-10,0, 10.0)).unwrap();
  // ...

  // integrate all the variables into the model.
  model.update().unwrap();

  // add a linear constraint
  model.add_constr("c0", x - y + 2.0*z, Equal, 0.0).unwrap();
  // ...

  // optimize the model. 
  model.optimize().unwrap();

  let status = model.get(attr::Status).unwrap();

  let x = x.get(&model, attr::X).unwrap();
  let y = y.get(&model, attr::X).unwrap();
  // ...
}
```

## License

Copyright (c) 2016, Yusuke Sasaki

This software is released under the MIT license, see [LICENSE](LICENSE).
