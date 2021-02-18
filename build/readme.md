# The Build Script

This build script is responsible for generating the enums containing Gurobi Attributes and Parameters.
The inputs are two CSV files in this directory.

`attrs.csv` has the following format:
```
attr,dtype,otype
```
where `attr` is the Gurobi attribute name (case sensitive), `dtype` is the datatype which governs the marker trait used for blanket impls.
The allowed values for `dtype` are described below:

| `dtype`  | Description                                      |
| -------- | ------------------------------------------------ |
| `dbl`    | `f64`,  marker trait `DoubleAttr`                |
| `int`    | `i32`,  marker trait `IntAttr`                   |
| `chr`   | `c_char`, marker trait `CharAttr`                |
| `str`    | `String`,  marker trait `StrAttr`                |
| `custom` | Custom datatype, no marker traits will be added. |

The `otype` is the object type to which this attribute belongs (`Model`, `Var`, `Constr`, etc).
The allowed values for `otype` are listed below.

| `otype`   | Description                          |
| --------- | ------------------------------------ |
| `model`   | no marker trait                      |
| `var`     | marker trait `ObjAttr<Obj=Var>`      |
| `constr`  | marker trait `ObjAttr<Obj=Constr>`   |
| `qconstr` | marker trait `ObjAttr<Obj=QConstr>`  |
| `sos`     | marker trait `ObjAttr<Obj=SOS>`      |

This build script will group attributes by `otype` and `dtype`, and generate enums as needed.  For example,
for `otype = "constr"` and `dtype = "str"` the following code is generated (in  `src/attribute/attr_enums.rs`):
```rust
/// String Gurobi attributes for [`Constr`](crate::Constr) objects.
/// 
/// This enum contains the following Gurobi attributes:
///  - [`CTag`](https://www.gurobi.com/documentation/9.1/refman/ctag.html)
///  - [`ConstrName`](https://www.gurobi.com/documentation/9.1/refman/constrname.html)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, FromCStr, AsCStr)]
pub enum ConstrStrAttr {
    CTag,
    ConstrName,
}

impl StrAttr for ConstrStrAttr {
}

impl ObjAttr for ConstrStrAttr {
    type Obj = Constr;
}
```
Note the two marker traits.  The latter would not be implemented if `otype = "model"`


`params.csv` has the format similar format,
```
param,dtype
```
where `param` is the Gurobi parameter name (case sensitive) and `dtype` has the same meaning as above.
Note that there are currently no `char` parameters implemented in Gurobi.

The parameters always relate to an `Env`, so marker traits are not needed.  Below is example output in `src/parameter/param_enums.rs`
for `dtype = "str"`
```rust
/// String Gurobi parameters.
/// 
/// This enum contains the following Gurobi parameters:
///  - [`LogFile`](https://www.gurobi.com/documentation/9.1/refman/logfile.html)
///  - [`NodefileDir`](https://www.gurobi.com/documentation/9.1/refman/nodefiledir.html)
///  - [`ResultFile`](https://www.gurobi.com/documentation/9.1/refman/resultfile.html)
///  - [`WorkerPool`](https://www.gurobi.com/documentation/9.1/refman/workerpool.html)
///  - [`WorkerPassword`](https://www.gurobi.com/documentation/9.1/refman/workerpassword.html)
///  - [`Dummy`](https://www.gurobi.com/documentation/9.1/refman/dummy.html)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, FromCStr, AsCStr)]
pub enum StrParam {
    LogFile,
    NodefileDir,
    ResultFile,
    WorkerPool,
    WorkerPassword,
    Dummy,
}
```

Finally, in both cases, the enums are added to a module called `enum_exports`, and the variants of the enums are added to a module called `variant_exports`:
```rust
pub(super) mod enum_exports {
  pub use super::{ModelDoubleAttr, ModelIntAttr, ...};
}

pub mod variant_exports {
  pub use super::ModelDoubleAttr::*;
  pub use super::ModelIntAttr::*;
  ...
}
```
