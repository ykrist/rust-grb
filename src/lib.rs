// vim: set foldmethod=syntax :

extern crate gurobi_sys as ffi;

use std::ptr::{null, null_mut};
use std::ffi::{CStr, CString};

pub use VarType::*;
pub use ConstrSense::*;
pub use ModelSense::*;

#[derive(Debug)]
pub enum IntParam {
  SolutionLimit,
  Method,
  ScaleFlag,
  SimplexPricing,
  Quad,
  NormAdjust,
  Sifting,
  SiftMethod,
  SubMIPNodes,
  VarBranch,
  Cuts,
  CliqueCuts,
  CoverCuts,
  FlowCoverCuts,
  FlowPathCuts,
  GUBCoverCuts,
  ImpliedCuts,
  MIPSepCuts,
  MIRCuts,
  ModKCuts,
  ZeroHalfCuts,
  NetworkCuts,
  SubMIPCuts,
  CutAggPasses,
  CutPasses,
  GomoryPasses,
  NodeMethod,
  Presolve,
  Aggregate,
  IISMethod,
  PreCrush,
  PreDepRow,
  PrePasses,
  DisplayInterval,
  OutputFlag,
  Threads,
  BarIterLimit,
  Crossover,
  CrossoverBasis,
  BarCorrectors,
  BarOrder,
  PumpPasses,
  RINS,
  Symmetry,
  MIPFocus,
  NumericFocus,
  AggFill,
  PreDual,
  SolutionNumber,
  MinRelNodes,
  ZeroObjNodes,
  BranchDir,
  InfUnbdInfo,
  DualReductions,
  BarHomogeneous,
  PreQLinearize,
  MIQCPMethod,
  QCPDual,
  LogToConsole,
  PreSparsify,
  PreMIQCPForm,
  Seed,
  ConcurrentMIP,
  ConcurrentJobs,
  DistributedMIPJobs,
  LazyConstraints,
  TuneResults,
  TuneTrials,
  TuneOutput,
  TuneJobs,
  Disconnected,
  NoRelHeuristic,
  UpdateMode,
  WorkerPort,
  Record,
}

#[derive(Debug)]
pub enum DoubleParam {
  Cutoff,
  IterationLimit,
  NodeLimit,
  TimeLimit,
  FeasibilityTol,
  IntFeasTol,
  MarkowitzTol,
  MIPGap,
  MIPGapAbs,
  OptimalityTol,
  PerturbValue,
  Heuristics,
  ObjScale,
  NodefileStart,
  BarConvTol,
  BarQCPConvTol,
  PSDTol,
  ImproveStartGap,
  ImproveStartNodes,
  ImproveStartTime,
  FeasRelaxBigM,
  TuneTimeLimit,
  PreSOS1BigM,
  PreSOS2BigM,
}

#[derive(Debug)]
pub enum StringParam {
  LogFile,
  NodefileDir,
  ResultFile,
  WorkerPool,
  WorkerPassword,
  Dummy,
}

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
pub enum CharAttr {
  VType,
  Sense,
  QCSense,
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

#[derive(Debug)]
pub enum StringAttr {
ModelName,
  VarName,
  ConstrName,
  QCName,
}


///
#[derive(Debug)]
pub enum VarType {
  Binary,
  Continuous(f64, f64),
  Integer(i64, i64),
}

///
pub enum ConstrSense {
  Equal,
  Greater,
  Less,
}

///
pub enum ModelSense {
  Minimize,
  Maximize,
}

///
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

pub type Result<T> = std::result::Result<T, Error>;

fn get_error_msg_env(env: *mut ffi::GRBenv) -> String {
  unsafe { from_c_str(ffi::GRBgeterrormsg(env)) }
}

fn make_c_str(s: &str) -> Result<CString> {
  CString::new(s).map_err(|e| Error::NulError(e))
}

unsafe fn from_c_str(s: *const ffi::c_char) -> String {
  CStr::from_ptr(s).to_string_lossy().into_owned()
}

/// Gurobi environment object
pub struct Env {
  env: *mut ffi::GRBenv,
}

/// Gurobi Model
pub struct Model<'a> {
  model: *mut ffi::GRBmodel,
  env: &'a Env,
}

impl Env {
  /// create an environment with log file
  pub fn new(logfilename: &str) -> Result<Env> {
    let mut env = null_mut::<ffi::GRBenv>();
    let logfilename = try!(make_c_str(logfilename));
    let error = unsafe { ffi::GRBloadenv(&mut env, logfilename.as_ptr()) };
    if error != 0 {
      return Err(Error::FromAPI(get_error_msg_env(env), error));
    }
    Ok(Env { env: env })
  }

  /// create an empty model object associted with the environment.
  pub fn new_model(&self, modelname: &str, sense: ModelSense) -> Result<Model> {
    let modelname = try!(make_c_str(modelname));
    let mut model = null_mut::<ffi::GRBmodel>();
    let error = unsafe {
      ffi::GRBnewmodel(self.env,
                       &mut model,
                       modelname.as_ptr(),
                       0,
                       null(),
                       null(),
                       null(),
                       null(),
                       null())
    };
    if error != 0 {
      return Err(Error::FromAPI(self.get_error_msg(), error));
    }

    let sense = match sense {
      ModelSense::Minimize => -1,
      ModelSense::Maximize => 1,
    };
    let attrname = try!(make_c_str(format!("{:?}", IntAttr::ModelSense).as_str()));
    let error = unsafe { ffi::GRBsetintattr(model, attrname.as_ptr(), sense) };
    if error != 0 {
      return Err(Error::FromAPI(self.get_error_msg(), error));
    }

    Ok(Model {
      model: model,
      env: self,
    })
  }

  pub fn get_int_param(&self, param: IntParam) -> Result<i32> {
    let mut value = 0; 
    let paramname = try!(make_c_str(format!("{:?}",param).as_str()));
    let error = unsafe {
      ffi::GRBgetintparam(self.env, paramname.as_ptr(), &mut value)
    };
    if error != 0 {
      return Err(Error::FromAPI(self.get_error_msg(), error));
    }
    Ok(value)
  }

  pub fn get_double_param(&self, param: DoubleParam) -> Result<f64> {
    let mut value = 0.0; 
    let paramname = try!(make_c_str(format!("{:?}",param).as_str()));
    let error = unsafe {
      ffi::GRBgetdblparam(self.env, paramname.as_ptr(), &mut value)
    };
    if error != 0 {
      return Err(Error::FromAPI(self.get_error_msg(), error));
    }
    Ok(value)
  }

  pub fn get_str_param(&self, param: StringParam) -> Result<String> {
    let mut buf = Vec::with_capacity(1024);
    let paramname = try!(make_c_str(format!("{:?}",param).as_str()));
    let error = unsafe {
      ffi::GRBgetstrparam(self.env, paramname.as_ptr(), buf.as_mut_ptr())
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

  /// optimize the model.
  pub fn optimize(&mut self) -> Result<()> {
    try!(self.update());

    let error = unsafe { ffi::GRBoptimize(self.model) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  /// write information of the model to file.
  pub fn write(&self, filename: &str) -> Result<()> {
    let filename = try!(make_c_str(filename));
    let error = unsafe { ffi::GRBwrite(self.model, filename.as_ptr()) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// get an integral attribute from API.
  pub fn get_int(&self, attr: IntAttr) -> Result<i64> {
    let mut value: ffi::c_int = 0;
    let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    let error =
      unsafe { ffi::GRBgetintattr(self.model, attrname.as_ptr(), &mut value) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(value as i64)
  }

  /// get an real-valued attribute from API.
  pub fn get_double(&self, attr: DoubleAttr) -> Result<f64> {
    let mut value: ffi::c_double = 0.0;
    let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    let error =
      unsafe { ffi::GRBgetdblattr(self.model, attrname.as_ptr(), &mut value) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(value as f64)
  }

  /// get an array of intagral attributes from API.
  pub fn get_int_array(&self,
                       attr: IntAttr,
                       first: usize,
                       len: usize)
                       -> Result<Vec<i32>> {
    let mut values = Vec::with_capacity(len);
    values.resize(len, 0);
    let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    let error = unsafe {
      ffi::GRBgetintattrarray(self.model,
                              attrname.as_ptr(),
                              first as ffi::c_int,
                              len as ffi::c_int,
                              values.as_mut_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(values)
  }

  /// get an array of real-valued attributes from API.
  pub fn get_double_array(&self,
                          attr: DoubleAttr,
                          first: usize,
                          len: usize)
                          -> Result<Vec<f64>> {
    let mut values = Vec::with_capacity(len);
    values.resize(len, 0.0);
    let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    let error = unsafe {
      ffi::GRBgetdblattrarray(self.model,
                              attrname.as_ptr(),
                              first as ffi::c_int,
                              len as ffi::c_int,
                              values.as_mut_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(values)
  }

  /// set an integral attribute from API.
  pub fn set_int(&self, attr: IntAttr, value: i32) -> Result<()> {
    let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    let error =
      unsafe { ffi::GRBsetintattr(self.model, attrname.as_ptr(), value) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// set an integral attribute from API.
  pub fn set_double(&self, attr: DoubleAttr, value: f64) -> Result<()> {
    let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    let error =
      unsafe { ffi::GRBsetdblattr(self.model, attrname.as_ptr(), value) };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// set the values of integral attributes
  pub fn set_int_array(&mut self,
                          attr: IntAttr,
                          first: usize,
                          values: &[i32])
                          -> Result<()> {
    let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    let error = unsafe {
      ffi::GRBsetintattrarray(self.model,
                              attrname.as_ptr(),
                              first as ffi::c_int,
                              values.len() as ffi::c_int,
                              values.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// set the values of real-valued attributes
  pub fn set_double_array(&mut self,
                          attr: DoubleAttr,
                          first: usize,
                          values: &[f64])
                          -> Result<()> {
    let attrname = try!(make_c_str(format!("{:?}", attr).as_str()));
    let error = unsafe {
      ffi::GRBsetdblattrarray(self.model,
                              attrname.as_ptr(),
                              first as ffi::c_int,
                              values.len() as ffi::c_int,
                              values.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }
    Ok(())
  }

  /// add a decision variable to the model.
  pub fn add_var(&mut self,
                 name: &str,
                 vtype: VarType,
                 obj: f64)
                 -> Result<()> {
    // extract parameters
    use VarType::*;
    let (vtype, lb, ub) = match vtype {
      Binary => ('B' as ffi::c_char, 0.0, 1.0),
      Continuous(lb, ub) => ('C' as ffi::c_char, lb, ub),
      Integer(lb, ub) => {
        ('I' as ffi::c_char, lb as ffi::c_double, ub as ffi::c_double)
      }
    };
    let name = try!(make_c_str(name));
    let error = unsafe {
      ffi::GRBaddvar(self.model,
                     0,
                     null(),
                     null(),
                     obj,
                     lb,
                     ub,
                     vtype,
                     name.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  /// add quadratic terms of objective function.
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

  /// add a quadratic constraint to the model.
  pub fn add_qconstr(&mut self,
                     constrname: &str,
                     lind: &[ffi::c_int],
                     lval: &[ffi::c_double],
                     qrow: &[ffi::c_int],
                     qcol: &[ffi::c_int],
                     qval: &[ffi::c_double],
                     sense: ConstrSense,
                     rhs: ffi::c_double)
                     -> Result<()> {
    if lind.len() != lval.len() {
      return Err(Error::InconsitentDims);
    }
    if qrow.len() != qcol.len() {
      return Err(Error::InconsitentDims);
    }
    if qcol.len() != qval.len() {
      return Err(Error::InconsitentDims);
    }

    let sense = match sense {
      ConstrSense::Equal => '=' as ffi::c_char,
      ConstrSense::Less => '<' as ffi::c_char,
      ConstrSense::Greater => '>' as ffi::c_char,
    };
    let constrname = try!(make_c_str(constrname));

    let error = unsafe {
      ffi::GRBaddqconstr(self.model,
                         lind.len() as ffi::c_int,
                         lind.as_ptr(),
                         lval.as_ptr(),
                         qrow.len() as ffi::c_int,
                         qrow.as_ptr(),
                         qcol.as_ptr(),
                         qval.as_ptr(),
                         sense,
                         rhs,
                         constrname.as_ptr())
    };
    if error != 0 {
      return Err(self.error_from_api(error));
    }

    Ok(())
  }

  /// add a linear constraint to the model.
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
      ConstrSense::Equal => '=' as ffi::c_char,
      ConstrSense::Less => '<' as ffi::c_char,
      ConstrSense::Greater => '>' as ffi::c_char,
    };
    let constrname = try!(make_c_str(name));

    let error = unsafe {
      ffi::GRBaddconstr(self.model,
                        ind.len() as ffi::c_int,
                        ind.as_ptr(),
                        val.as_ptr(),
                        sense,
                        rhs,
                        constrname.as_ptr())
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


#[test]
fn test1() {
  let s1 = "mip1.log";
  let s2 = unsafe { from_c_str(make_c_str(s1).unwrap().as_ptr()) };
  assert!(s1 == s2);
}

#[test]
fn test2() {
  let s1 = "mip1.log";
  let env = Env::new(s1).unwrap();
  let logfilename = env.get_str_param("LogFile").unwrap();
  assert_eq!(s1, logfilename);
}
