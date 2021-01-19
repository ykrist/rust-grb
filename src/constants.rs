use gurobi_sys::c_int;

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