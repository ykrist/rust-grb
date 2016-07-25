// vim: set foldmethod=syntax :

extern crate gurobi_sys as ffi;

use std::ptr::null_mut;
use std::ffi::{CStr, CString};

///
#[derive(Debug)]
pub enum IntAttr {
  NumConstrs,
  NumVars,
  NumSOS,
  NumQConstrs,
  NumNZs,
  NumQNZs,
  NumQCNZs,
  NumIntVars,
  NumBinVars,
  NumPWLObjVars,
  ModelSense,
  IsMIP,
  IsQP,
  IsQCP,
  Status,
  SolCount,
  BarIterCount,
  VBasis,
  CBasis,
  PWLObjCvx,
  BranchPriority,
  VarPreStat,
  BoundVioIndex,
  BoundSVioIndex,
  ConstrVioIndex,
  ConstrSVioIndex,
  ConstrResidualIndex,
  ConstrSResidualIndex,
  DualVioIndex,
  DualSVioIndex,
  DualResidualIndex,
  DualSResidualIndex,
  ComplVioIndex,
  IntVioIndex,
  IISMinimal,
  IISLB,
  IISUB,
  IISConstr,
  IISSOS,
  IISQConstr,
  TuneResultCount,
  Lazy,
  VarHintPri,
}

#[derive(Debug)]
pub enum DoubleAttr {
  Runtime,
  ObjCon,
  LB,
  UB,
  Obj,
  Start,
  PreFixVal,
  RHS,
  QCRHS,
  MaxCoeff,
  MinCoeff,
  MaxBound,
  MinBound,
  MaxObjCoeff,
  MinObjCoeff,
  MaxRHS,
  MinRHS,
  ObjVal,
  ObjBound,
  ObjBoundC,
  MIPGap,
  IterCount,
  NodeCount,
  X,
  RC,
  Pi,
  QCPi,
  Slack,
  QCSlack,
  BoundVio,
  BoundSVio,
  BoundVioSum,
  BoundSVioSum,
  ConstrVio,
  ConstrSVio,
  ConstrVioSum,
  ConstrSVioSum,
  ConstrResidual,
  ConstrSResidual,
  ConstrResidualSum,
  ConstrSResidualSum,
  DualVio,
  DualSVio,
  DualVioSum,
  DualSVioSum,
  DualResidual,
  DualSResidual,
  DualResidualSum,
  DualSResidualSum,
  ComplVio,
  ComplVioSum,
  IntVio,
  IntVioSum,
  Kappa,
  KappaExact,
  SAObjLow,
  SAObjUp,
  SALBLow,
  SALBUp,
  SARHSLow,
  SAUBLow,
  SAUBUp,
  SARHSUp,
  Xn,
  FarkasProof,
  FarkasDual,
  UnbdRay,
  PStart,
  DStart,
  BarX,
  VarHintVal,
}

/// represents error information which called the API.
#[derive(Debug)]
pub enum Error {
  /// This function has yet implemented
  NotImplemented,
  /// An exception returned from Gurobi C API
  FromAPI(String, ffi::c_int),
  /// see https://doc.rust-lang.org/std/ffi/struct.NulError.html
  NulError(std::ffi::NulError),
  /// Inconsistent argument dimensions.
  InconsitentDims,
}

///
#[derive(Debug)]
pub enum VarType {
  Binary,
  Continuous,
  Integer,
}

pub enum ConstrSense {
  Equal,
  Greater,
  Less,
}
pub use ConstrSense::*;

pub enum ModelSense {
  Minimize,
  Maximize,
}
pub use ModelSense::*;

///
pub type Result<T> = std::result::Result<T, Error>;

fn get_error_msg_env(env: *mut ffi::GRBenv) -> String {
  unsafe { from_c_str(ffi::GRBgeterrormsg(env)) }
}

fn make_c_str(s: &str) -> Result<*const ffi::c_schar> {
  let cstr = try!(CString::new(s).map_err(|e| Error::NulError(e)));
  Ok(cstr.as_ptr())
}

unsafe fn from_c_str(s: *const ffi::c_schar) -> String {
  CStr::from_ptr(s).to_string_lossy().into_owned()
}

/// Gurobi environment object
pub struct Env {
  env: *mut ffi::GRBenv,
}

///
pub struct Model<'a> {
  model: *mut ffi::GRBmodel,
  env: &'a Env,
}

#[test]
fn test1() {
  let s1 = "mip1.log";
  let s2 = unsafe { from_c_str(make_c_str(s1).unwrap()) };
  assert!(s1 == s2);
}

#[test]
fn test2() {
  let s1 = "mip1.log";
  let env = Env::new(s1).unwrap();
  let logfilename = env.get_str_param("LogFile").unwrap();
  assert_eq!(s1, logfilename);
}

impl Env {
  /// create an empty environment with log file
  pub fn new(logfilename: &str) -> Result<Env> {
    let mut env = null_mut::<ffi::GRBenv>();
    let logfilename = try!(make_c_str(logfilename));
    let error = unsafe { ffi::GRBloadenv(&mut env, logfilename) };
    if error != 0 {
      return Err(Error::FromAPI(get_error_msg_env(env), error));
    }
    Ok(Env { env: env })
  }

  pub fn new_model(&self, modelname: &str, sense: ModelSense) -> Result<Model> {
    let mut model = null_mut::<ffi::GRBmodel>();
    let error = unsafe {
      ffi::GRBnewmodel(self.env,
                       &mut model,
                       try!(make_c_str(modelname)),
                       0,
                       null_mut(),
                       null_mut(),
                       null_mut(),
                       null_mut(),
                       null_mut())
    };
    if error != 0 {
      return Err(Error::FromAPI(self.get_error_msg(), error));
    }

    let sense = match sense {
      ModelSense::Minimize => -1,
      ModelSense::Maximize => 1,
    };
    let error = unsafe {
      ffi::GRBsetintattr(model, try!(make_c_str("ModelSense")), sense)
    };
    if error != 0 {
      return Err(Error::FromAPI(self.get_error_msg(), error));
    }

    Ok(Model {
      model: model,
      env: self,
    })
  }

  pub fn get_str_param(&self, paramname: &str) -> Result<String> {
    let mut buf = Vec::with_capacity(1024);
    let error = unsafe {
      ffi::GRBgetstrparam(self.env,
                          try!(make_c_str(paramname)),
                          buf.as_mut_ptr())
    };
    if error != 0 {
      return Err(Error::FromAPI(self.get_error_msg(), error));
    }
    Ok(unsafe { from_c_str(buf.as_ptr()) })
  }

  fn get_error_msg(&self) -> String {
    get_error_msg_env(self.env)
  }
}

impl Drop for Env {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreeenv(self.env) };
    self.env = null_mut();
  }
}

impl<'a> Model<'a> {
  ///
  pub fn optimize(&mut self) -> Result<()> {
    try!(self.update());

    let error = unsafe { ffi::GRBoptimize(self.model) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  ///
  pub fn write(&self, filename: &str) -> Result<()> {
    let error =
      unsafe { ffi::GRBwrite(self.model, try!(make_c_str(filename))) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  ///
  pub fn get_int(&self, attr: IntAttr) -> Result<i64> {
    let mut value: ffi::c_int = 0;
    let error = unsafe {
      ffi::GRBgetintattr(self.model,
                         try!(make_c_str(format!("{:?}", attr).as_str())),
                         &mut value)
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(value as i64)
  }

  pub fn get_double(&self, attr: DoubleAttr) -> Result<f64> {
    let mut value: ffi::c_double = 0.0;
    let error = unsafe {
      ffi::GRBgetdblattr(self.model,
                         try!(make_c_str(format!("{:?}", attr).as_str())),
                         &mut value)
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(value as f64)
  }

  pub fn get_double_array(&self,
                          attr: DoubleAttr,
                          first: usize,
                          len: usize)
                          -> Result<Vec<f64>> {
    let mut values = Vec::with_capacity(len);
    values.resize(len, 0.0);
    let error = unsafe {
      ffi::GRBgetdblattrarray(self.model,
                              try!(make_c_str(format!("{:?}", attr).as_str())),
                              first as ffi::c_int,
                              len as ffi::c_int,
                              values.as_mut_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(values)
  }

  pub fn add_qpterms(&mut self,
                     qrow: &[ffi::c_int],
                     qcol: &[ffi::c_int],
                     qval: &[ffi::c_double])
                     -> Result<()> {
    if qrow.len() != qcol.len() {
      return Err(Error::InconsitentDims);
    }
    if qcol.len() != qval.len() {
      return Err(Error::InconsitentDims);
    }

    let error = unsafe {
      ffi::GRBaddqpterms(self.model,
                         qrow.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  pub fn set_double_array(&mut self,
                          attr: DoubleAttr,
                          first: usize,
                          values: &[f64])
                          -> Result<()> {
    let error = unsafe {
      ffi::GRBsetdblattrarray(self.model,
                              try!(make_c_str(format!("{:?}", attr).as_str())),
                              first as ffi::c_int,
                              values.len() as ffi::c_int,
                              values.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  pub fn add_bvar(&mut self, name: &str, obj: f64) -> Result<()> {
    self.add_var(name, VarType::Binary, 0.0, 1.0, obj)
  }

  pub fn add_cvar(&mut self, name: &str, obj: f64) -> Result<()> {
    self.add_var(name, VarType::Continuous, 0.0, 1.0, obj)
  }

  pub fn add_ivar(&mut self, name: &str, obj: f64) -> Result<()> {
    self.add_var(name, VarType::Integer, 0.0, 1.0, obj)
  }

  pub fn add_constr(&mut self,
                    name: &str,
                    ind: &[ffi::c_int],
                    val: &[ffi::c_double],
                    sense: ConstrSense,
                    rhs: ffi::c_double)
                    -> Result<()> {
    if ind.len() != val.len() {
      return Err(Error::InconsitentDims);
    }

    let sense = match sense {
      ConstrSense::Equal => '=' as ffi::c_schar,
      ConstrSense::Less => '<' as ffi::c_schar,
      ConstrSense::Greater => '>' as ffi::c_schar,
    };

    let error = unsafe {
      ffi::GRBaddconstr(self.model,
                        ind.len() as ffi::c_int,
                        ind.as_ptr(),
                        val.as_ptr(),
                        sense,
                        rhs,
                        try!(make_c_str(name)))
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// apply all modification of the model to process.
  pub fn update(&mut self) -> Result<()> {
    let error = unsafe { ffi::GRBupdatemodel(self.model) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// create a copy of the model
  pub fn copy(&self) -> Result<Model> {
    let copied = unsafe { ffi::GRBcopymodel(self.model) };
    if copied.is_null() {
      return Err(Error::FromAPI("Failed to create a copy of the model"
                                  .to_owned(),
                                20002));
    }
    Ok(Model {
      env: self.env,
      model: copied,
    })
  }
}


// internal methods.
impl<'a> Model<'a> {
  fn add_var(&mut self,
             name: &str,
             vtype: VarType,
             lb: f64,
             ub: f64,
             obj: f64)
             -> Result<()> {
    use VarType::*;
    let vtype = match vtype {
      Binary => 'B' as ffi::c_schar,
      Continuous => 'C' as ffi::c_schar,
      Integer => 'I' as ffi::c_schar,
    };

    let error = unsafe {
      ffi::GRBaddvar(self.model,
                     0,
                     null_mut(),
                     null_mut(),
                     obj,
                     lb,
                     ub,
                     vtype,
                     try!(make_c_str(name)))
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  // make an instance of error object related to C API.
  fn error_from_api(&self, errcode: ffi::c_int) -> Error {
    Error::FromAPI(self.env.get_error_msg(), errcode)
  }
}

impl<'a> Drop for Model<'a> {
  fn drop(&mut self) {
    unsafe { ffi::GRBfreemodel(self.model) };
    self.model = null_mut();
  }
}
