#![allow(improper_ctypes)]

pub use std::os::raw::c_int;
pub use std::os::raw::c_double;
pub use std::os::raw::c_char;

#[repr(C)]
pub struct GRBenv;

#[repr(C)]
pub struct GRBmodel;

extern "C" {
  pub fn GRBloadenv(envP: *mut *mut GRBenv,
                    logfilename: *const c_char)
                    -> c_int;

  pub fn GRBfreeenv(env: *mut GRBenv);
  
  pub fn GRBgeterrormsg(env: *mut GRBenv) -> *const c_char;

  pub fn GRBgetintparam(env: *mut GRBenv,
                        paramname: *const c_char,
                        value: *mut c_int)
                        -> c_int;

  pub fn GRBgetdblparam(env: *mut GRBenv,
                        paramname: *const c_char,
                        value: *mut c_double)
                        -> c_int;

  pub fn GRBgetstrparam(env: *mut GRBenv,
                        paramname: *const c_char,
                        value: *mut c_char)
                        -> c_int;

  pub fn GRBnewmodel(env: *mut GRBenv,
                     modelP: *mut *mut GRBmodel,
                     Pname: *const c_char,
                     numvars: c_int,
                     obj: *const c_double,
                     lb: *const c_double,
                     ub: *const c_double,
                     vtype: *const c_char,
                     varnames: *const *const c_char)
                     -> c_int;

  pub fn GRBfreemodel(model: *mut GRBmodel) -> c_int;

  pub fn GRBupdatemodel(model: *mut GRBmodel) -> c_int;

  pub fn GRBcopymodel(model: *mut GRBmodel) -> *mut GRBmodel;

  pub fn GRBoptimize(model: *mut GRBmodel) -> c_int;

  pub fn GRBwrite(model: *mut GRBmodel, filename: *const c_char) -> c_int;

  pub fn GRBaddvar(model: *mut GRBmodel,
                   numnz: c_int,
                   vind: *const c_int,
                   vval: *const c_double,
                   obj: f64,
                   lb: f64,
                   ub: f64,
                   vtype: c_char,
                   name: *const c_char)
                   -> c_int;

  pub fn GRBaddconstr(model: *mut GRBmodel,
                      numnz: c_int,
                      cind: *const c_int,
                      cval: *const c_double,
                      sense: c_char,
                      rhs: c_double,
                      constrname: *const c_char)
                      -> c_int;

  pub fn GRBaddqconstr(model: *mut GRBmodel,
                       numlnz: c_int,
                       lind: *const c_int,
                       lval: *const c_double,
                       numqnz: c_int,
                       qrow: *const c_int,
                       qcol: *const c_int,
                       qval: *const c_double,
                       sense: c_char,
                       rhs: c_double,
                       QCname: *const c_char)
                       -> c_int;

  pub fn GRBaddqpterms(model: *mut GRBmodel,
                       numqnz: c_int,
                       qrow: *const c_int,
                       qcol: *const c_int,
                       qval: *const c_double)
                       -> c_int;

  pub fn GRBgetintattr(model: *mut GRBmodel,
                       attrname: *const c_char,
                       valueP: *mut c_int)
                       -> c_int;

  pub fn GRBgetdblattr(model: *mut GRBmodel,
                       attrname: *const c_char,
                       valueP: *mut c_double)
                       -> c_int;

  pub fn GRBgetintattrarray(model: *mut GRBmodel,
                            attrname: *const c_char,
                            first: c_int,
                            len: c_int,
                            values: *mut c_int)
                            -> c_int;

  pub fn GRBgetdblattrarray(model: *mut GRBmodel,
                            attrname: *const c_char,
                            first: c_int,
                            len: c_int,
                            values: *mut c_double)
                            -> c_int;

  pub fn GRBsetintattr(model: *mut GRBmodel,
                       attrname: *const c_char,
                       value: c_int)
                       -> c_int;

  pub fn GRBsetdblattr(model: *mut GRBmodel,
                       attrname: *const c_char,
                       value: c_double)
                       -> c_int;

  pub fn GRBsetintattrarray(model: *mut GRBmodel,
                            attrname: *const c_char,
                            first: c_int,
                            len: c_int,
                            values: *const c_int)
                            -> c_int;

  pub fn GRBsetdblattrarray(model: *mut GRBmodel,
                            attrname: *const c_char,
                            first: c_int,
                            len: c_int,
                            values: *const c_double)
                            -> c_int;
}
