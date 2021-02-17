#![allow(clippy::missing_safety_doc)]
//! Gurobi parameters for [`Env`](crate::Env)  and [`Model`](crate::Model) objects.  See the
//! [manual](https://www.gurobi.com/documentation/9.1/refman/parameters.html) for a list
//! of parameters and their uses.
use grb_sys as ffi;
use std::ffi::{CStr, CString};

use crate::env::Env;
use crate::util::{copy_c_str, AsPtr};
use crate::Result;

mod param_enums; // generated code - see build/main.rs

#[doc(inline)]
pub use param_enums::enum_exports::*;
pub use param_enums::variant_exports as param;

use crate::constants::GRB_MAX_STRLEN;
use cstr_enum::AsCStr;

/// A queryable Gurobi parameter for a [`Model`](crate::Model) or [`Env`](crate::Env)
pub trait ParamGet<V> {
    /// This parameter's value type (string, double, int, char)
    /// Query a parameter from an environment
    fn get(&self, env: &Env) -> Result<V>;
}

/// A modifiable Gurobi parameter for a [`Model`](crate::Model) or [`Env`](crate::Env)
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
    impl_param_get! { i32, i32::MIN, grb_sys::GRBgetintparam }
}

impl ParamSet<i32> for IntParam {
    impl_param_set! { i32, grb_sys::GRBsetintparam }
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
            env.check_apicall(grb_sys::GRBgetstrparam(
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
            env.check_apicall(grb_sys::GRBsetstrparam(
                env.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                value.as_ptr(),
            ))
        }
    }
}

/// Support for querying and seting undocumented Gurobi parameters.
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
/// # Example
/// ```
/// use grb::prelude::*;
/// use grb::parameter::Undocumented;
///
/// let mut m = Model::new("model")?;
/// let undocumented_parameter = Undocumented::new("GURO_PAR_MINBPFORBID")?;
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
pub struct Undocumented {
    name: CString,
}

impl Undocumented {
    /// Declare a new `Undocumented` parameter.
    ///
    /// # Errors
    /// Will return an [`Error::NulError`](crate::Error) if the string given cannot be converted into a
    /// C-style string.
    pub fn new(string: impl Into<Vec<u8>>) -> Result<Undocumented> {
        Ok(Undocumented {
            name: CString::new(string)?,
        })
    }
}

// not strictly necessary, since we can use self.name directly
impl AsCStr for Undocumented {
    fn as_cstr(&self) -> &CStr {
        self.name.as_ref()
    }
}

impl ParamGet<i32> for &Undocumented {
    impl_param_get! { i32, i32::MIN, grb_sys::GRBgetintparam }
}

impl ParamSet<i32> for &Undocumented {
    impl_param_set! { i32, grb_sys::GRBsetintparam }
}

impl ParamGet<f64> for &Undocumented {
    impl_param_get! { f64, f64::NAN, ffi::GRBgetdblparam }
}

impl ParamSet<f64> for &Undocumented {
    impl_param_set! { f64, ffi::GRBsetdblparam }
}

impl ParamGet<String> for &Undocumented {
    fn get(&self, env: &Env) -> Result<String> {
        let mut buf = [0i8; GRB_MAX_STRLEN];
        unsafe {
            env.check_apicall(grb_sys::GRBgetstrparam(
                env.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                buf.as_mut_ptr(),
            ))?;
            Ok(copy_c_str(buf.as_ptr()))
        }
    }
}

impl ParamSet<String> for &Undocumented {
    fn set(&self, env: &mut Env, value: String) -> Result<()> {
        let value = CString::new(value)?;
        unsafe {
            env.check_apicall(grb_sys::GRBsetstrparam(
                env.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                value.as_ptr(),
            ))
        }
    }
}
