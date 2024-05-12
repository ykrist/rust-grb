use std::ffi::CString;
use std::ptr::null_mut;
use std::rc::Rc;

use crate::error::{Error, Result};
use crate::parameter::{ParamGet, ParamSet};
use crate::util;
use grb_sys2 as ffi;
use grb_sys2::GRBenv;
use util::AsPtr;

/// Represents a User-Allocated Gurobi Env
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UserAllocEnv {
    ptr: *mut GRBenv,
}

impl AsPtr for UserAllocEnv {
    type Ptr = GRBenv;
    unsafe fn as_mut_ptr(&self) -> *mut GRBenv {
        self.ptr
    }
}

impl Drop for UserAllocEnv {
    fn drop(&mut self) {
        debug_assert!(!self.ptr.is_null());
        unsafe { ffi::GRBfreeenv(self.ptr) };
        self.ptr = null_mut();
    }
}

/// A Gurobi Environment object.
///
/// [`Model`s](crate::Model) objects created with [`Model::new`](crate::Model::new) will use the default `Env`.
/// This default `Env` is thread-local and lazily initialized.  Currently, it lasts until the current thread;
/// there is no way to de-allocate it from the current thread.
pub struct Env {
    /// The original user-allocated environment created by the user
    user_allocated: Rc<UserAllocEnv>,
    /// Is None if Env is user-allocated, otherwise is `Some(ptr)` where `ptr `
    /// is a Gurobi-allocated *GRBEnv
    gurobi_allocated: Option<*mut GRBenv>,
}

impl AsPtr for Env {
    type Ptr = GRBenv;
    unsafe fn as_mut_ptr(&self) -> *mut GRBenv {
        self.gurobi_allocated
            .unwrap_or_else(|| self.user_allocated.as_mut_ptr())
    }
}

/// Gurobi environment object (see the Gurobi [manual](https://www.gurobi.com/documentation/9.1/refman/environments.html))
/// A Gurobi environment which hasn't been started yet. Some Gurobi parameters,
/// such as [`Record`](https://www.gurobi.com/documentation/9.1/refman/record.html)
/// need to be set before the environment has been started.
///
/// Setting [`param::OutputFlag`](crate::param::OutputFlag) to 0 on an `EmptyEnv` is useful for stopping the license info from
/// being printed, since this happens on starting the environment.
///
/// # Examples
/// ```
/// use grb::*;
/// let mut env = Env::empty()?;
/// env.set(param::OutputFlag, 0)? // license will not be printed when env starts
///   .set(param::UpdateMode, 1)?
///   .set(param::LogFile, "".to_string())?;
/// let env : Env = env.start()?;
/// # Ok::<(), Error>(())
/// ```
pub struct EmptyEnv {
    env: Env,
}

impl EmptyEnv {
    /// Query a parameter value
    pub fn get<P: ParamGet<V>, V>(&self, param: P) -> Result<V> {
        self.env.get(param)
    }

    /// Set a parameter value
    pub fn set<P: ParamSet<V>, V>(&mut self, param: P, value: V) -> Result<&mut Self> {
        self.env.set(param, value)?;
        Ok(self)
    }

    /// Start the environment, returning the [`Env`] on success.
    pub fn start(self) -> Result<Env> {
        self.env
            .check_apicall(unsafe { ffi::GRBstartenv(self.env.as_mut_ptr()) })?;
        Ok(self.env)
    }
}

impl Env {
    thread_local!(pub(crate) static DEFAULT_ENV : Env = Env::new("gurobi.log").unwrap());

    pub(crate) fn is_shared(&self) -> bool {
        Rc::strong_count(&self.user_allocated) > 1 || Rc::weak_count(&self.user_allocated) > 0
    }
    /// Wrap user-allocated Gurobi env pointer
    /// # Safety
    /// - `ptr` must be non-null
    /// - `ptr` must have been obtained using `GRBEmptyEnv` or `GRBloadenv`
    /// - `ptr` must not have previously been used (elsewhere wrapped)
    unsafe fn new_user_allocated(ptr: *mut GRBenv) -> Env {
        debug_assert!(!ptr.is_null());
        Env {
            user_allocated: Rc::new(UserAllocEnv { ptr }),
            gurobi_allocated: None,
        }
    }

    /// Wrap Gurobi-allocated Gurobi env pointer
    /// # Safety
    /// - `ptr` must be non-null
    /// - `ptr` must have been obtained using `GRBgetenv`
    pub(crate) unsafe fn new_gurobi_allocated(original: &Env, ptr: *mut ffi::GRBenv) -> Env {
        debug_assert!(!ptr.is_null());
        Env {
            user_allocated: Rc::clone(&original.user_allocated),
            gurobi_allocated: Some(ptr),
        }
    }

    /// Create a new empty and un-started environment.
    pub fn empty() -> Result<EmptyEnv> {
        let mut env = null_mut();
        let err_code = unsafe { ffi::GRBemptyenv(&mut env) };
        if err_code != 0 {
            return Err(Error::FromAPI(get_error_msg(env), err_code));
        }
        let env = unsafe { Env::new_user_allocated(env) };
        Ok(EmptyEnv { env })
    }

    /// Create an environment with log file
    ///
    /// Setting `logfilename` to an empty string will not create a logfile.
    pub fn new(logfilename: &str) -> Result<Env> {
        let mut env = null_mut();
        let logfilename = CString::new(logfilename)?;
        let error = unsafe { ffi::GRBloadenv(&mut env, logfilename.as_ptr()) };
        if error != 0 {
            return Err(Error::FromAPI(get_error_msg(env), error));
        }
        Ok(unsafe { Env::new_user_allocated(env) })
    }

    /// Query the value of a parameter
    pub fn get<P: ParamGet<V>, V>(&self, param: P) -> Result<V> {
        param.get(self)
    }

    /// Set the value of a parameter
    pub fn set<P: ParamSet<V>, V>(&mut self, param: P, value: V) -> Result<()> {
        param.set(self, value)
    }

    /// Import a set of parameter values from a file
    pub fn read_params(&mut self, filename: &str) -> Result<()> {
        let filename = CString::new(filename)?;
        self.check_apicall(unsafe { ffi::GRBreadparams(self.as_mut_ptr(), filename.as_ptr()) })
    }

    /// Write the set of parameter values to a file
    pub fn write_params(&self, filename: &str) -> Result<()> {
        let filename = CString::new(filename)?;
        self.check_apicall(unsafe { ffi::GRBwriteparams(self.as_mut_ptr(), filename.as_ptr()) })
    }

    /// Insert a message into log file.
    ///
    /// When **message** cannot convert to raw C string, a panic is occurred.
    pub fn message(&self, message: &str) {
        let msg = CString::new(message).unwrap();
        unsafe { ffi::GRBmsg(self.as_mut_ptr(), msg.as_ptr()) };
    }

    pub(crate) fn error_from_api(&self, error: ffi::c_int) -> Error {
        Error::FromAPI(get_error_msg(unsafe { self.as_mut_ptr() }), error)
    }

    pub(crate) fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
        if error != 0 {
            return Err(self.error_from_api(error));
        }
        Ok(())
    }
}

fn get_error_msg(env: *mut ffi::GRBenv) -> String {
    unsafe { util::copy_c_str(ffi::GRBgeterrormsg(env)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{param, Model};

    #[test]
    fn param_get_set() {
        use super::*;
        let mut env = Env::new("").unwrap();
        env.set(param::IISMethod, 1).unwrap();
        assert_eq!(env.get(param::IISMethod).unwrap(), 1);
        env.set(param::IISMethod, 0).unwrap();
        assert_eq!(env.get(param::IISMethod).unwrap(), 0);
        assert!(env.set(param::IISMethod, 9999).is_err());
    }

    #[test]
    fn default_env_created_once() -> Result<()> {
        let m1 = Model::new("m1")?;
        let m2 = Model::new("m2")?;
        assert_eq!(&m1.get_env().user_allocated, &m2.get_env().user_allocated);
        Ok(())
    }
}
