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
                    "unexpected value {ch:?} when converting to VarType"
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
                    "unexpected value {ch:?} when converting to ConstrSense"
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

/// Type of general constraint
#[non_exhaustive]
#[repr(i32)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum GenConstrType {
    /// The constraint $r = \max\{x_1,\ldots,x_k,c\}$ states that
    /// the resultant variable $r$ should be equal to the maximum of
    /// the operand variables $x_1,\ldots,x_k$ and the constant $c$.
    ///
    /// For example, a solution $(r=3, x_1=2, x_2=3, x_3=0)$ would be feasible for
    /// the constraint $r = \max\{x_1,x_2,x_3,1.7\}$
    /// because $3$ is indeed the maximum of $2$, $3$, $0$, and $1.7$.
    Max,
    /// The constraint $r = \min\{x_1,\ldots,x_k,c\}$ states that
    /// the resultant variable $r$ should be equal to the minimum of
    /// the operand variables $x_1,\ldots,x_k$ and the constant $c$.
    Min,
    /// The constraint $r = \mbox{abs}\{x\}$ states that the resultant variable $r$
    /// should be equal to the absolute value of the operand variable $x$.
    ///
    /// For example, a solution $(r=3, x=-3)$ would be feasible for
    /// the constraint $r = \mbox{abs}\{x\}$.
    Abs,
    /// The constraint $r = \mbox{and}\{x_1,\ldots,x_k\}$ states that
    /// the binary resultant variable $r$ should be $1$ if and only if
    /// all of the binary operand variables $x_1,\ldots,x_k$ are equal to $1$.
    ///
    /// For example, a solution $(r=1, x_1=1, x_2=1, x_3=1)$ would be feasible for
    /// the constraint $r = \mbox{and}\{x_1,x_2,x_3\}$.
    ///
    /// TODO: remove?
    /// Note that any involved variables that are not already binary are converted to binary.
    And,
    /// Similar to an AND constraint, the constraint $r = \mbox{or}\{x_1,\ldots,x_k\}$ states that
    /// the binary resultant variable $r$ should be $1$ if and only if
    /// at least one of the binary operand variables $x_1,\ldots,x_k$ is equal to $1$.
    ///
    /// TODO: remove?
    /// Note that any involved variables that are not already binary are converted to binary.
    Or,
    /// The constraint $r = \mbox{norm}\{x_1,\ldots,x_k\}$ states that
    /// the resultant variable $r$ should be equal to
    /// the vector norm of the operand variables $x_1,\ldots,x_k$.
    ///
    /// TODO: remove?
    /// A few options are available: the 0-norm, 1-norm, 2-norm, and infinity-norm.
    Norm,
    /// An indicator constraint $y = f \rightarrow a^Tx \leq b$ states that
    /// if the binary indicator variable $y$ is equal to $f$ in a given solution, where $f \in \{0,1\}$,
    /// then the linear constraint $a^Tx \leq b$ has to be satisfied.
    /// On the other hand, if $y \neq f$ (i.e., $y = 1-f$) then the linear constraint may be violated.
    ///
    /// Note that the sense of the linear constraint can also be $=$ or $\geq$;
    /// refer to this earlier section for a more detailed description of linear constraints.
    ///
    /// Note also that declaring an INDICATOR constraint implicitly declares the indicator variable to be of binary type.
    Indicator,
    /// A piecewise-linear constraint $y = f(x)$ states that
    /// the point $(x, y)$ must lie on the piecewise-linear function $f()$ defined by
    /// a set of points $(x_1, y_1), (x_2, y_2), ..., (x_n, y_n)$.
    ///
    /// TODO: remove?
    /// Refer to the description of piecewise-linear objectives for details of how piecewise-linear functions are defined.
    Pwl,
    /// $y = p_0 x^n + p_1 x^{n-1} + ... + p_n x + p_{n+1}$
    Polynomial,
    /// $y = exp(x)$ or $y = e^x$
    NaturalExp,
    /// $y = a^x$, where $a > 0$ is the base for the exponential function
    Exp,
    /// : $y = \log_e(x)$ or $y = \ln(x)$
    NaturalLog,
    /// $y = \log_a(x)$, where $a > 0$ is the base for the logarithmic function
    Log,
    /// $y = \frac{1}{1 + exp(-x)}$ or $y = \frac{1}{1 + e^{-x}}$
    Logistic,
    /// $y = x^a$, where $x \geq 0$ for any $a$ and $x > 0$ for $a < 0$
    Pow,
    /// $y = \sin(x)$
    Sin,
    /// $y = \cos(x)$
    Cos,
    /// $y = \tan(x)$
    Tan,
}

impl TryFrom<i32> for GenConstrType {
    type Error = String;
    fn try_from(val: i32) -> std::result::Result<Self, Self::Error> {
        match val {
            1..=18 => Ok(unsafe { std::mem::transmute(val) }),
            _ => Err("Invalid GenConstrType value, should be in [1,18]".to_string()),
        }
    }
}
