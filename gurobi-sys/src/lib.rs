#![allow(improper_ctypes)]

extern crate libc;

pub use libc::{c_int, c_double, c_schar};

#[repr(C)]
pub struct GRBenv;

#[repr(C)]
pub struct GRBmodel;

extern "C" {
  pub fn GRBloadenv(envP: *mut *mut GRBenv,
                    logfilename: *const c_schar)
                    -> c_int;

  pub fn GRBgeterrormsg(env: *mut GRBenv) -> *const c_schar;

  pub fn GRBfreeenv(env: *mut GRBenv);

  pub fn GRBnewmodel(env: *mut GRBenv,
                     modelP: *mut *mut GRBmodel,
                     Pname: *const c_schar,
                     numvars: c_int,
                     obj: *const c_double,
                     lb: *const c_double,
                     ub: *const c_double,
                     vtype: *const c_schar,
                     varnames: *const *const c_schar)
                     -> c_int;

  pub fn GRBfreemodel(model: *mut GRBmodel) -> c_int;

  pub fn GRBoptimize(model: *mut GRBmodel) -> c_int;

  pub fn GRBgetintattr(model: *mut GRBmodel,
                       attrname: *const c_schar,
                       valueP: *mut c_int)
                       -> c_int;

  pub fn GRBgetdblattr(model: *mut GRBmodel,
                       attrname: *const c_schar,
                       valueP: *mut c_double)
                       -> c_int;

  pub fn GRBgetdblattrarray(model: *mut GRBmodel,
                            attrname: *const c_schar,
                            first: c_int,
                            len: c_int,
                            values: *mut c_double)
                            -> c_int;

  pub fn GRBsetintattr(model: *mut GRBmodel,
                       attrname: *const c_schar,
                       value: c_int)
                       -> c_int;
  pub fn GRBwrite(model: *mut GRBmodel, filename: *const c_schar) -> c_int;

  pub fn GRBaddvar(model: *mut GRBmodel,
                   numnz: c_int,
                   vind: *const c_int,
                   vval: *const c_double,
                   obj: f64,
                   lb: f64,
                   ub: f64,
                   vtype: c_schar,
                   name: *const c_schar)
                   -> c_int;

  pub fn GRBupdatemodel(model: *mut GRBmodel) -> c_int;
  pub fn GRBcopymodel(model: *mut GRBmodel) -> *mut GRBmodel;

  pub fn GRBgetstrparam(env: *mut GRBenv,
                        paramname: *const c_schar,
                        value: *const c_schar)
                        -> c_int;

  pub fn GRBaddconstr(model: *mut GRBmodel,
                      numnz: c_int,
                      cind: *const c_int,
                      cval: *const c_double,
                      sense: c_schar,
                      rhs: c_double,
                      constrname: *const c_schar)
                      -> c_int;

  pub fn GRBaddqpterms(model: *mut GRBmodel,
                       numqnz: c_int,
                       qrow: *const c_int,
                       qcol: *const c_int,
                       qval: *const c_double)
                       -> c_int;

  pub fn GRBsetdblattrarray(model: *mut GRBmodel,
                            attrname: *const c_schar,
                            first: c_int,
                            len: c_int,
                            values: *const c_double)
                            -> c_int;

}
