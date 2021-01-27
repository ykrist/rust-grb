use gurobi_sys::{c_char, c_int};

// Constants defined by Gurobi API
pub const GRB_MAX_STRLEN : usize = 512;
pub const GRB_UNDEFINED : f64 = 1e101;
pub const GRB_INFINITY: f64 = 1e100;

pub const ERROR_INVALID_ARGUMENT: c_int = 10003;
pub const ERROR_UNKNOWN_ATTR: c_int = 10004;
pub const ERROR_DATA_NOT_AVAILABLE: c_int = 10005;

pub mod callback {
  // Location where the callback called.
  pub const POLLING: i32 = 0;
  pub const PRESOLVE: i32 = 1;
  pub const SIMPLEX: i32 = 2;
  pub const MIP: i32 = 3;
  pub const MIPSOL: i32 = 4;
  pub const MIPNODE: i32 = 5;
  pub const MESSAGE: i32 = 6;
  pub const BARRIER: i32 = 7;

  pub const PRE_COLDEL: i32 = 1000;
  pub const PRE_ROWDEL: i32 = 1001;
  pub const PRE_SENCHG: i32 = 1002;
  pub const PRE_BNDCHG: i32 = 1003;
  pub const PRE_COECHG: i32 = 1004;

  pub const SPX_ITRCNT: i32 = 2000;
  pub const SPX_OBJVAL: i32 = 2001;
  pub const SPX_PRIMINF: i32 = 2002;
  pub const SPX_DUALINF: i32 = 2003;
  pub const SPX_ISPERT: i32 = 2004;

  pub const MIP_OBJBST: i32 = 3000;
  pub const MIP_OBJBND: i32 = 3001;
  pub const MIP_NODCNT: i32 = 3002;
  pub const MIP_SOLCNT: i32 = 3003;
  pub const MIP_CUTCNT: i32 = 3004;
  pub const MIP_NODLFT: i32 = 3005;
  pub const MIP_ITRCNT: i32 = 3006;
  #[allow(dead_code)]
  pub const MIP_OBJBNDC: i32 = 3007;

  pub const MIPSOL_SOL: i32 = 4001;
  pub const MIPSOL_OBJ: i32 = 4002;
  pub const MIPSOL_OBJBST: i32 = 4003;
  pub const MIPSOL_OBJBND: i32 = 4004;
  pub const MIPSOL_NODCNT: i32 = 4005;
  pub const MIPSOL_SOLCNT: i32 = 4006;
  #[allow(dead_code)]
  pub const MIPSOL_OBJBNDC: i32 = 4007;

  pub const MIPNODE_STATUS: i32 = 5001;
  pub const MIPNODE_REL: i32 = 5002;
  pub const MIPNODE_OBJBST: i32 = 5003;
  pub const MIPNODE_OBJBND: i32 = 5004;
  pub const MIPNODE_NODCNT: i32 = 5005;
  pub const MIPNODE_SOLCNT: i32 = 5006;
  #[allow(dead_code)]
  pub const MIPNODE_BRVAR: i32 = 5007;
  #[allow(dead_code)]
  pub const MIPNODE_OBJBNDC: i32 = 5008;

  pub const MSG_STRING: i32 = 6001;
  pub const RUNTIME: i32 = 6002;

  pub const BARRIER_ITRCNT: i32 = 7001;
  pub const BARRIER_PRIMOBJ: i32 = 7002;
  pub const BARRIER_DUALOBJ: i32 = 7003;
  pub const BARRIER_PRIMINF: i32 = 7004;
  pub const BARRIER_DUALINF: i32 = 7005;
  pub const BARRIER_COMPL: i32 = 7006;
}


/// Type for new variable
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum VarType {
  Binary = b'B',
  Continuous = b'C',
  Integer = b'I',
  SemiCont = b'S',
  SemiInt = b'N',
}

impl Into<c_char> for VarType {
  fn into(self) -> c_char { self as u8 as c_char}
}

impl Into<VarType> for c_char {
  fn into(self) -> VarType {
    let ch = self as u8 as char;
    match ch {
      'B' => VarType::Binary,
      'C' => VarType::Continuous,
      'I' => VarType::Integer,
      'S' => VarType::SemiCont,
      'N' => VarType::SemiInt,
      ch => panic!("unexpected value `{}` when converting to VarType", ch),
    }
  }
}



/// Sense for new linear/quadratic constraint
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ConstrSense {
  Equal = b'=',
  Greater = b'>',
  Less = b'<',
}

impl Into<c_char> for ConstrSense {
  fn into(self) -> c_char {  self as u8 as c_char  }
}


/// Sense of new objective function
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum ModelSense {
  Minimize = 1,
  Maximize = -1,
}

impl Into<i32> for ModelSense {
  fn into(self) -> i32 { self as i32 }
}


/// Type of new SOS constraint
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum SOSType {
  SOSType1 = 1,
  SOSType2 = 2,
}

impl Into<i32> for SOSType {
  fn into(self) -> i32 { self as i32 }
}



/// Status of a model
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum Status {
  Loaded = 1,
  Optimal,
  Infeasible,
  InfOrUnbd,
  Unbounded,
  CutOff,
  IterationLimit,
  NodeLimit,
  TimeLimit,
  SolutionLimit,
  Interrupted,
  Numeric,
  SubOptimal,
  InProgress,
}

impl From<i32> for Status {
  fn from(val: i32) -> Status {
    match val {
      1..=14 => unsafe { std::mem::transmute(val) },
      _ => panic!("cannot convert to Status: {}", val)
    }
  }
}

/// Type of cost function at feasibility relaxation
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum RelaxType {
  /// The weighted magnitude of bounds and constraint violations
  /// ($penalty(s\_i) = w\_i s\_i$)
  Linear = 0,

  /// The weighted square of magnitude of bounds and constraint violations
  /// ($penalty(s\_i) = w\_i s\_i\^2$)
  Quadratic = 1,

  /// The weighted count of bounds and constraint violations
  /// ($penalty(s\_i) = w\_i \cdot [s\_i > 0]$)
  Cardinality = 2,
}

impl Into<i32> for RelaxType {
  fn into(self) -> i32 { self as i32 }
}


