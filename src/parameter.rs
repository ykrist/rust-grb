#![allow(clippy::missing_safety_doc)]
//! Gurobi parameters for [`Env`]  and [`Model`](crate::Model) objects.  See the
//! [manual](https://www.gurobi.com/documentation/9.1/refman/parameters.html) for a list
//! of parameters and their uses.
use grb_sys2 as ffi;
use std::ffi::{CStr, CString};

use crate::env::Env;
use crate::util::{copy_c_str, AsPtr};
use crate::Result;

#[allow(missing_docs)]
mod param_enums {
    include!(concat!(env!("OUT_DIR"), "/param_enums.rs"));
    // generated code - see build/main.rs
}

#[doc(inline)]
pub use param_enums::enum_exports::*;
pub use param_enums::variant_exports as param;

use crate::constants::GRB_MAX_STRLEN;
use cstr_enum::AsCStr;

/// A queryable Gurobi parameter for a [`Model`](crate::Model) or [`Env`]
pub trait ParamGet<V> {
    /// This parameter's value type (string, double, int, char)
    /// Query a parameter from an environment
    fn get(&self, env: &Env) -> Result<V>;
}

/// A modifiable Gurobi parameter for a [`Model`](crate::Model) or [`Env`]
pub trait ParamSet<V> {
    /// Set a parameter on an environment
    fn set(&self, env: &mut Env, value: V) -> Result<()>;
}

macro_rules! impl_param_get {
    ($t:ty,  $default:expr, $get:path) => {
        #[inline]
        fn get(&self, env: &Env) -> Result<$t> {
            let mut val = $default;
            unsafe {
                env.check_apicall($get(env.as_mut_ptr(), self.as_cstr().as_ptr(), &mut val))?;
            }
            Ok(val)
        }
    };
}

macro_rules! impl_param_set {
    ($t:ty,  $set:path) => {
        #[inline]
        fn set(&self, env: &mut Env, value: $t) -> Result<()> {
            unsafe {
                env.check_apicall($set(env.as_mut_ptr(), self.as_cstr().as_ptr(), value))?;
            }
            Ok(())
        }
    };
}

impl ParamGet<i32> for IntParam {
    impl_param_get! { i32, i32::MIN, ffi::GRBgetintparam }
}

impl ParamSet<i32> for IntParam {
    impl_param_set! { i32, ffi::GRBsetintparam }
}

impl ParamGet<f64> for DoubleParam {
    impl_param_get! { f64, f64::NAN, ffi::GRBgetdblparam }
}

impl ParamSet<f64> for DoubleParam {
    impl_param_set! { f64, ffi::GRBsetdblparam }
}

impl ParamGet<String> for StrParam {
    fn get(&self, env: &Env) -> Result<String> {
        let mut buf = [0i8; GRB_MAX_STRLEN];
        unsafe {
            env.check_apicall(ffi::GRBgetstrparam(
                env.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                buf.as_mut_ptr(),
            ))?;
            Ok(copy_c_str(buf.as_ptr()))
        }
    }
}

impl ParamSet<String> for StrParam {
    fn set(&self, env: &mut Env, value: String) -> Result<()> {
        let value = CString::new(value)?;
        unsafe {
            env.check_apicall(ffi::GRBsetstrparam(
                env.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                value.as_ptr(),
            ))
        }
    }
}

/// Support for querying and setting dynamic/undocumented Gurobi parameters.
///
/// Use an instance of this type to set or query parameters using the [`Model::get_param`](crate::Model::get_param)
/// or [`Model::set_param`](crate::Model::set_param) methods.
///
/// Current (very short) list of undocumented parameters:
///
/// | Name | Type | Default | Description |
/// | --- | --- | --- | --- |
/// | `GURO_PAR_MINBPFORBID` | `i32` | `2000000000` |Minimum `BranchPriority` a variable must have to stop it being removed during presolve |
///
/// This is also useful for using new parameters which are not yet supported directly by this crate.
///
/// # Example
/// ```
/// use grb::prelude::*;
/// use grb::parameter::Parameter;
///
/// let mut m = Model::new("model")?;
/// let undocumented_parameter = Parameter::new("GURO_PAR_MINBPFORBID")?;
///
/// // requires return type to be annotated
/// let val : i32 = m.get_param(&undocumented_parameter)?;
/// assert_eq!(val, 2_000_000_000);
///
/// // Wrong return type results in a FromAPI error
/// let result : grb::Result<String> = m.get_param(&undocumented_parameter);
/// assert!(matches!(result, Err(grb::Error::FromAPI(_, _))));
///
/// m.set_param(&undocumented_parameter, 10)?;
/// # Ok::<(), grb::Error>(())
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct Parameter {
    name: CString,
}

#[doc(hidden)]
#[deprecated = "Renamed to `Parameter` type"]
pub type Undocumented = Parameter;

impl Parameter {
    /// Declare a new parameter.
    ///
    /// # Errors
    /// Will return an [`Error::NulError`](crate::Error) if the string given cannot be converted into a
    /// C-style string.
    pub fn new(string: impl Into<Vec<u8>>) -> Result<Parameter> {
        Ok(Parameter {
            name: CString::new(string)?,
        })
    }
}

// not strictly necessary, since we can use self.name directly
impl AsCStr for Parameter {
    fn as_cstr(&self) -> &CStr {
        self.name.as_ref()
    }
}

impl ParamGet<i32> for &Parameter {
    impl_param_get! { i32, i32::MIN, ffi::GRBgetintparam }
}

impl ParamSet<i32> for &Parameter {
    impl_param_set! { i32, ffi::GRBsetintparam }
}

impl ParamGet<f64> for &Parameter {
    impl_param_get! { f64, f64::NAN, ffi::GRBgetdblparam }
}

impl ParamSet<f64> for &Parameter {
    impl_param_set! { f64, ffi::GRBsetdblparam }
}

impl ParamGet<String> for &Parameter {
    fn get(&self, env: &Env) -> Result<String> {
        let mut buf = [0i8; GRB_MAX_STRLEN];
        unsafe {
            env.check_apicall(ffi::GRBgetstrparam(
                env.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                buf.as_mut_ptr(),
            ))?;
            Ok(copy_c_str(buf.as_ptr()))
        }
    }
}

impl ParamSet<String> for &Parameter {
    fn set(&self, env: &mut Env, value: String) -> Result<()> {
        let value = CString::new(value)?;
        unsafe {
            env.check_apicall(ffi::GRBsetstrparam(
                env.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                value.as_ptr(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parameter_names() -> crate::Result<()> {
        let params: Vec<_> =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/build/params.csv"))
                .unwrap()
                .lines()
                .skip(1)
                .map(|line| {
                    let mut line = line.split(",");
                    let param = line.next().unwrap();
                    let ty = line.next().unwrap();
                    assert_eq!(line.next(), None);
                    (param.to_string(), ty.to_string())
                })
                .collect();

        let model = crate::Model::new("test")?;
        for (param, ty) in params {
            let param = Parameter::new(param).unwrap();
            match ty.as_str() {
                "dbl" => {
                    let _v: f64 = model.get_param(&param)?;
                }
                "int" => {
                    let _v: i32 = model.get_param(&param)?;
                }
                "str" => {
                    let _v: String = model.get_param(&param)?;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}
