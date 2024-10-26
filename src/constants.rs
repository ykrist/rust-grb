use grb_sys2::{c_char, c_int};
use std::convert::TryFrom;

// Constants defined by Gurobi API
pub const GRB_MAX_STRLEN: usize = 512;
pub const GRB_UNDEFINED: f64 = 1e101;
/// A large constant used by Gurobi to represent numeric infinity.
pub const GRB_INFINITY: f64 = 1e100;

pub const ERROR_CALLBACK: c_int = 10011;

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
    #[allow(dead_code)]
    pub const MULTIOBJ: i32 = 8;
    pub const IIS: i32 = 9;

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
    pub const MIP_OPENSCENARIOS: i32 = 3007;
    pub const MIP_PHASE: i32 = 3008;

    pub const MIPSOL_SOL: i32 = 4001;
    pub const MIPSOL_OBJ: i32 = 4002;
    pub const MIPSOL_OBJBST: i32 = 4003;
    pub const MIPSOL_OBJBND: i32 = 4004;
    pub const MIPSOL_NODCNT: i32 = 4005;
    pub const MIPSOL_SOLCNT: i32 = 4006;
    pub const MIPSOL_OPENSCENARIOS: i32 = 4007;
    pub const MIPSOL_PHASE: i32 = 4008;

    pub const MIPNODE_STATUS: i32 = 5001;
    pub const MIPNODE_REL: i32 = 5002;
    pub const MIPNODE_OBJBST: i32 = 5003;
    pub const MIPNODE_OBJBND: i32 = 5004;
    pub const MIPNODE_NODCNT: i32 = 5005;
    pub const MIPNODE_SOLCNT: i32 = 5006;
    #[allow(dead_code)]
    pub const MIPNODE_BRVAR: i32 = 5007;
    pub const MIPNODE_OPENSCENARIOS: i32 = 5008;
    pub const MIPNODE_PHASE: i32 = 5009;

    pub const MSG_STRING: i32 = 6001;
    pub const RUNTIME: i32 = 6002;

    pub const BARRIER_ITRCNT: i32 = 7001;
    pub const BARRIER_PRIMOBJ: i32 = 7002;
    pub const BARRIER_DUALOBJ: i32 = 7003;
    pub const BARRIER_PRIMINF: i32 = 7004;
    pub const BARRIER_DUALINF: i32 = 7005;
    pub const BARRIER_COMPL: i32 = 7006;

    #[allow(dead_code)]
    pub const MULTIOBJ_OBJCNT: i32 = 8001;
    #[allow(dead_code)]
    pub const MULTIOBJ_SOLCNT: i32 = 8002;
    #[allow(dead_code)]
    pub const MULTIOBJ_SOL: i32 = 8003;

    pub const IIS_CONSTRMIN: i32 = 9001;
    pub const IIS_CONSTRMAX: i32 = 9002;
    pub const IIS_CONSTRGUESS: i32 = 9003;
    pub const IIS_BOUNDMIN: i32 = 9004;
    pub const IIS_BOUNDMAX: i32 = 9005;
    pub const IIS_BOUNDGUESS: i32 = 9006;
}

/// Gurobi variable types (see [manual](https://www.gurobi.com/documentation/9.1/refman/variables.html))
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum VarType {
    Binary = b'B',
    Continuous = b'C',
    Integer = b'I',
    SemiCont = b'S',
    SemiInt = b'N',
}

impl From<VarType> for c_char {
    fn from(val: VarType) -> Self {
        val as u8 as c_char
    }
}

impl TryFrom<c_char> for VarType {
    type Error = String;
    fn try_from(val: c_char) -> std::result::Result<VarType, String> {
        let ch = val as u8;
        let vt = match ch {
            b'B' => VarType::Binary,
            b'C' => VarType::Continuous,
            b'I' => VarType::Integer,
            b'S' => VarType::SemiCont,
            b'N' => VarType::SemiInt,
            _ => {
                return Err(format!(
                    "unexpected value {:?} when converting to VarType",
                    ch
                ))
            }
        };
        Ok(vt)
    }
}

/// Sense for new linear/quadratic constraint
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ConstrSense {
    /// An equality constraint
    Equal = b'=',
    /// A greater-than constraint (left-hand side greater than or equal to right-hand side)
    Greater = b'>',
    /// A less-than constraint (left-hand side less than or equal to right-hand side)
    Less = b'<',
}

impl TryFrom<c_char> for ConstrSense {
    type Error = String;
    fn try_from(val: c_char) -> std::result::Result<ConstrSense, String> {
        let ch = val as u8;
        let vt = match ch {
            b'=' => ConstrSense::Equal,
            b'>' => ConstrSense::Greater,
            b'<' => ConstrSense::Less,
            _ => {
                return Err(format!(
                    "unexpected value {:?} when converting to ConstrSense",
                    ch
                ))
            }
        };
        Ok(vt)
    }
}

/// Sense of objective function, aka direction of optimisation.
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum ModelSense {
    /// Minimise the objective function
    Minimize = 1,
    /// Maximise the objective function
    Maximize = -1,
}

impl TryFrom<i32> for ModelSense {
    type Error = String;
    fn try_from(val: i32) -> std::result::Result<ModelSense, String> {
        match val {
            -1 => Ok(ModelSense::Maximize),
            1 => Ok(ModelSense::Minimize),
            _ => Err("Invalid ModelSense value, should be -1 or 1".to_string()),
        }
    }
}

/// Type of [SOS constraint](https://www.gurobi.com/documentation/9.1/refman/constraints.html)
#[derive(Debug, Copy, Clone)]
#[repr(i32)]
pub enum SOSType {
    /// Type 1 SOS constraint
    Ty1 = 1,
    /// Type 2 SOS constraint
    Ty2 = 2,
}

/// Status of a model
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum Status {
    /// Model is loaded, but no solution information is available.
    Loaded = 1,
    /// Model was solved to optimality (subject to tolerances), and an optimal solution is available.
    Optimal,
    /// Model was proven to be infeasible.
    Infeasible,
    /// Model was proven to be either infeasible or unbounded. To obtain a more definitive conclusion,
    /// set the `DualReductions` parameter to 0 and reoptimize
    InfOrUnbd,
    /// Model was proven to be unbounded.
    ///
    /// *Important note:* an unbounded status indicates the presence of an unbounded ray that allows
    /// the objective to improve without limit. It says nothing about whether the model has a feasible
    /// solution. If you require information on feasibility, you should set the objective to zero and
    /// reoptimize.
    Unbounded,
    /// Optimal objective for model was proven to be worse than the value specified in the Cutoff parameter.
    /// No solution information is available.
    CutOff,
    /// Optimization terminated because the total number of simplex iterations performed exceeded the value
    /// specified in the `IterationLimit` parameter, or because the total number of barrier iterations
    /// exceeded the value specified in the `BarIterLimit` parameter.
    IterationLimit,
    /// Optimization terminated because the total number of branch-and-cut nodes explored exceeded
    /// the value specified in the `NodeLimit` parameter.
    NodeLimit,
    /// Optimization terminated because the time expended exceeded the value specified in the `TimeLimit` parameter.
    TimeLimit,
    /// Optimization terminated because the number of solutions found reached the value specified in
    /// the `SolutionLimit` parameter.
    SolutionLimit,
    /// Optimization was terminated by the user.
    Interrupted,
    /// Optimization was terminated due to unrecoverable numerical difficulties.
    Numeric,
    /// Unable to satisfy optimality tolerances; a sub-optimal solution is available.
    SubOptimal,
    /// An asynchronous optimization call was made, but the associated optimization run is not yet complete.
    InProgress,
    /// User specified an objective limit (a bound on either the best objective or the best bound), and that
    /// limit has been reached.
    UserObjLimit,
}

impl TryFrom<i32> for Status {
    type Error = String;
    fn try_from(val: i32) -> std::result::Result<Status, String> {
        match val {
            1..=15 => Ok(unsafe { std::mem::transmute::<i32, Status>(val) }),
            _ => Err("Invalid Status value, should be in [1,15]".to_string()),
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
