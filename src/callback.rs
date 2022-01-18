//! Interface to Gurobi's callback API
//!
//! Gurobi allows for user callbacks to be called at different points during a solve.
//! At each of these points, the user may query or modify the model in different ways.
//!
//! This module provides a context handle type for each place at which a callback may be called.
//! In the Gurobi [manual](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html),
//! these are represented by the `where` values. The handle types are bundled in the
//! [`Where`] enum, so to obtain an instance of a particular handle type
//! in a callback, use pattern matching. For example:
//! ```
//! # use grb::prelude::*;
//! # use grb::callback::CbResult;
//! fn callback(w: Where) -> CbResult {
//!   match w {
//!     Where::PreSolve(ctx) => {/* type of ctx = PreSolveCtx  */ },
//!     Where::MIPSol(ctx) => {/* type of ctx = MIPCtx  */ },
//!     _ => {},
//!   }
//!   Ok(())
//! }
//! ```
//!
//! For details on each handle type and its available methods, see the `*Ctx` structs in this module.
//!
//! Callbacks can be defined using the [`Callback`] trait on an object, or using a closure.
//!
//! # Examples
//! ## Using closures
//! Because of Rust's lifetime requirements on closures, if you are using large lookup structures within your
//! callbacks, you should wrap them in a [`std::rc::Rc`]`<`[`std::cell::RefCell`]`<_>>`.  This can be a little
//! tedious, so if you need to use a stateful callback, so implementing the `Callback` trait is preferred.
//! ```
//! use grb::prelude::*;
//! use std::{rc::Rc, cell::RefCell};
//!
//! #[derive(Default)]
//! struct MyCallbackStats {
//!   ncalls : usize,
//!   big_data: [u8; 32],
//! }
//!
//! let mut m = Model::new("model")?;
//! let x = add_ctsvar!(m, obj: 2)?;
//! let y = add_intvar!(m, bounds: 0..100)?;
//! m.add_constr("c0", c!(x <= y - 0.5 ))?;
//!
//! // Need to put `stats` behind a Rc<RefCell<_>> because of closure lifetimes.
//! let stats = Rc::new(RefCell::new(MyCallbackStats::default()));
//!
//! let mut callback = {
//!   // Note that `MyCallbackStats` doesn't implement Clone: `Rc<_>` makes a cheap pointer copy
//!   let stats = stats.clone();
//!   // `move` moves the `stats` clone we just made into the closure
//!   move |w : Where| {
//!     // This should never panic - `callback` runs single-threaded
//!     let stats: &mut MyCallbackStats = &mut *stats.borrow_mut();
//!     if let Where::Polling(_) = w {
//!       println!("in polling: callback has been called {} times", stats.ncalls);
//!     }
//!     stats.ncalls += 1;
//!     Ok(())
//!  }
//! };
//!
//! m.optimize_with_callback(&mut callback)?;
//!
//! # Ok::<(), grb::Error>(())
//! ```
//!
//! ## Using the `Callback` trait
//! ```
//! use grb::prelude::*;
//! use grb::callback::CbResult;
//!
//! #[derive(Default)]
//! struct MyCallbackStats {
//!   ncalls : usize,
//!   big_data: [u8; 32],
//! }
//!
//! impl Callback for MyCallbackStats {
//!   fn callback(&mut self, w: Where) -> CbResult {
//!     if let Where::Polling(_) = w {
//!       println!("in polling: callback has been called {} times", self.ncalls);
//!     }
//!     self.ncalls += 1;
//!     Ok(())
//!   }
//! }
//! let mut m = Model::new("model")?;
//! let x = add_ctsvar!(m, obj: 2)?;
//! let y = add_intvar!(m, bounds: 0..100)?;
//! m.add_constr("c0", c!(x <= y - 0.5 ))?;
//!
//! let mut stats = MyCallbackStats::default();
//! m.optimize_with_callback(&mut stats)?;
//!
//! # Ok::<(), grb::Error>(())
//! ```

use grb_sys2 as ffi;
use std::borrow::Borrow;
use std::convert::TryInto;
use std::iter::{IntoIterator, Iterator};
use std::os::raw;
use std::ptr::null;

use crate::constants::{callback::*, ERROR_CALLBACK, GRB_UNDEFINED};
use crate::constr::IneqExpr;
use crate::util::{self, AsPtr};
use crate::{model::Model, Error, Result, Status, Var, INFINITY}; // used for setting a partial solution in a callback

/// The return type for callbacks, an alias of [`anyhow::Result`].
///
/// All callbacks, whether they are implemented as closures, functions or objects
/// should return this type.  The [`anyhow::Error`] type can be constructed from any
/// [`std::error::Error`], so you can use the `?` operator on any error inside a callback.
pub type CbResult = anyhow::Result<()>;

/// A trait that allows structs to be used as a callback object
///
/// # Examples
/// This example shows how to store every integer solution found during a MIP solve
/// ```
/// use grb::prelude::*;
/// use grb::callback::CbResult;
///
/// struct CallbackData {
///   vars: Vec<Var>,
///   solutions: Vec<Vec<f64>>,
/// }
///
/// impl Callback for CallbackData {
///   fn callback(&mut self, w: Where) -> CbResult {
///     match w {
///       Where::MIPSol(ctx) => {
///         self.solutions.push(ctx.get_solution(&self.vars)?)
///       }
///       _ => {}
///     }
///     Ok(())
///   }
/// }
/// ```
/// This example shows how to cache lazy cuts for later use (perhaps adding them as hard constraints with
/// [`Model::add_constrs`] once optimisation has finished)
/// ```
/// use grb::prelude::*;
/// use grb::constr::IneqExpr;
/// use grb::callback::CbResult;
///
/// struct LazyCutSep {
///   vars: Vec<Var>,
///   past_cuts : Vec<IneqExpr>,
/// }
///
/// impl LazyCutSep {
///   fn separate_cuts(&self, solution: &[f64]) -> Vec<IneqExpr> {
///     /* ... */
///     # Vec::new()
///   }
/// }
///
/// impl Callback for LazyCutSep {
///   fn callback(&mut self, w: Where) -> CbResult {
///     if let Where::MIPSol(ctx) = w {
///       let solution = ctx.get_solution(&self.vars)?;
///       let cuts = self.separate_cuts(&solution);
///       self.past_cuts.extend_from_slice(&cuts);
///       for c in cuts {
///         ctx.add_lazy(c)?;
///       }
///     }
///     Ok(())
///   }
/// }
/// ```
///
pub trait Callback {
    /// The main callback method.  The pattern-matching the [`Where`] will give a
    /// context object (see module-level docs) which can be used to interact with Gurobi.
    fn callback(&mut self, w: Where) -> CbResult;
}

impl<F: FnMut(Where) -> CbResult> Callback for F {
    fn callback(&mut self, w: Where) -> CbResult {
        self(w)
    }
}

/// The C function given to the Gurobi API with `GRBsetcallbackfunc`
pub(crate) extern "C" fn callback_wrapper(
    _model: *mut ffi::GRBmodel,
    cbdata: *mut ffi::c_void,
    where_: ffi::c_int,
    usrdata: *mut ffi::c_void,
) -> ffi::c_int {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let u = unsafe { &mut *(usrdata as *mut UserCallbackData) };
    let (cb_obj, model, nvars) = (&mut u.cb_obj, u.model, u.nvars);
    let where_ = Where::new(CbCtx::new(cbdata, where_, model, nvars));

    let callback_result = catch_unwind(AssertUnwindSafe(|| {
        let w = match where_ {
            Ok(w) => w,
            #[allow(unused_variables)]
            Err(e @ Error::NotYetSupported(_)) => {
                // eprintln!("{}", e);
                return Ok(());
            }
            Err(_) => unreachable!(),
        };
        cb_obj.callback(w)
    }));

    match callback_result {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            eprintln!("Callback returned error:\n{:#?}", e);
            ERROR_CALLBACK
        }
        Err(_) => {
            eprintln!("Callback panicked! You should return an error instead.");
            ERROR_CALLBACK
        }
    }
}

/// The `usrdata` struct passed to [`callback_wrapper`]
pub(crate) struct UserCallbackData<'a> {
    pub(crate) model: &'a Model,
    pub(crate) nvars: usize,
    pub(crate) cb_obj: &'a mut dyn Callback,
}

macro_rules! impl_getter {
    ($name:ident, i32, $wher:path, $what:path, $help:literal) => {
        #[doc = $help]
        pub fn $name(&self) -> Result<i32> {
            self.0.get_int($wher, $what)
        }
    };

    ($name:ident, i32 => $t:path, $wher:path, $what:path, $help:literal, $map:expr) => {
        #[doc = $help]
        pub fn $name(&self) -> Result<$t> {
            self.0.get_int($wher, $what).map($map)
        }
    };

    ($name:ident, f64, $wher:path, $what:path, $help:literal) => {
        #[doc = $help]
        pub fn $name(&self) -> Result<f64> {
            self.0.get_double($wher, $what)
        }
    };
}

macro_rules! impl_set_solution {
    () => {
        /// Provide a new feasible solution for a MIP model.  Not all variables need to be given.
        /// The return value on success has the following meaning, depending from on the callback context:
        ///
        /// | Context | Possible Values | Meaning |
        /// | --- | --- | --- |
        /// | [`MIPNodeCtx`] | `None` | Suggested solution was not feasible. |
        /// |              | `Some(val)` | Suggested solution was feasible, has objective value of `val`. |
        /// | [`MIPSolCtx`] | `None` | No information |
        /// | [`MIPCtx`] | `None` | No information |
        ///
        /// On success, if the solution was feasible the method returns the computed objective value,
        /// otherwise returns `None`.
        pub fn set_solution<I, V, T>(&self, solution: I) -> Result<Option<f64>>
        where
            V: Borrow<Var>,
            T: Borrow<f64>,
            I: IntoIterator<Item = (V, T)>,
        {
            self.0.set_solution(solution)
        }
    };
}

// TODO: add WORK; rename
macro_rules! impl_runtime {
    () => {
        /// Retrieve the elapsed solver runtime in seconds.
        pub fn runtime(&self) -> Result<f64> {
            self.0.get_runtime()
        }
    };
}

macro_rules! impl_common {
    () => {
        /// Signal Gurobi to terminate the optimisation.  Will not take effect immediately
        pub fn terminate(&self) {
            self.0.terminate()
        }

        /// Generate a request to proceed to the next phase of the computation. Note that the request is only accepted in
        ///  a few phases of the algorithm, and it won't be acted upon immediately.
        ///
        /// In the current Gurobi version, this callback allows you to proceed from the NoRel heuristic to the standard MIP
        ///  search. You can determine the current algorithm phase `ctx.proceed()` (in [`MIPCtx`], [`MIPNodeCtx`] and [`MIPSolCtx`]).
        pub fn proceed(&mut self) {
            self.0.proceed()
        }
    };
}

macro_rules! impl_add_lazy {
    () => {
        /// Add a new lazy constraint to the model
        ///
        /// *Important*: Requires that the `LazyConstraints` parameter is set to 1
        pub fn add_lazy(&self, constr: IneqExpr) -> Result<()> {
            self.0.add_lazy(constr)
        }
    };
}

/// Callback context object during [`POLLING`](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html).
pub struct PollingCtx<'a>(CbCtx<'a>);
impl<'a> PollingCtx<'a> {
    impl_common! {}
}

/// Callback context object during [`PRESOLVE`](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html).
pub struct PreSolveCtx<'a>(CbCtx<'a>);
impl<'a> PreSolveCtx<'a> {
    impl_common! {}
    impl_runtime! {}
    impl_getter! { col_del, i32, PRESOLVE, PRE_COLDEL, "Number of columns removed so far." }
    impl_getter! { row_del, i32, PRESOLVE, PRE_ROWDEL, "Number of rows removed so far." }
    impl_getter! { sense_chg, i32, PRESOLVE, PRE_SENCHG, "Number of constraint senses changed so far." }
    impl_getter! { bnd_chg, i32, PRESOLVE, PRE_BNDCHG, "Number of variable bounds changed so far." }
    impl_getter! { coeff_chg, i32, PRESOLVE, PRE_COECHG, "Number of coefficients changed so far." }
}

/// Callback context object during [`SIMPLEX`](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html).
pub struct SimplexCtx<'a>(CbCtx<'a>);
impl<'a> SimplexCtx<'a> {
    impl_common! {}
    impl_runtime! {}
    impl_getter! { iter_cnt, f64, SIMPLEX, SPX_ITRCNT, "Current simplex iteration count." }
    impl_getter! { obj_val, f64, SIMPLEX, SPX_OBJVAL, "Current simplex objective value." }
    impl_getter! { prim_inf, f64, SIMPLEX, SPX_PRIMINF, "Current primal infeasibility." }
    impl_getter! { dual_inf, f64, SIMPLEX, SPX_DUALINF, "Current primal infeasibility." }
    impl_getter! { is_perturbed, i32, SIMPLEX, SPX_ISPERT, "Is problem currently perturbed?" }
}

/// Callback context object during [`MIP`](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html).
pub struct MIPCtx<'a>(CbCtx<'a>);
impl<'a> MIPCtx<'a> {
    impl_common! {}
    impl_set_solution! {}
    impl_runtime! {}
    impl_getter! { open_scenarios, i32, MIP, MIP_OPENSCENARIOS, "Number of scenarios that are still open in a multi-scenario model." }
    impl_getter! { obj_best, f64, MIP, MIP_OBJBST, "Current best objective." }
    impl_getter! { obj_bnd, f64, MIP, MIP_OBJBND, "Current best objective bound." }
    impl_getter! { node_cnt, f64, MIP, MIP_NODCNT, "Current explored node count." }
    impl_getter! { sol_cnt, i32, MIP, MIP_SOLCNT, "Current count of feasible solutions found." }
    impl_getter! { cut_cnt, i32, MIP, MIP_CUTCNT, "Current count of cutting planes applied." }
    impl_getter! { node_left, f64, MIP, MIP_NODLFT, "Current unexplored node count." }
    impl_getter! { iter_cnt, f64, MIP, MIP_ITRCNT, "Current simplex iteration count." }

    /// Current algorithmic phase in the MIP solution
    pub fn phase(&self) -> Result<MipPhase> {
        MipPhase::from_raw(self.0.get_int(MIP, MIP_PHASE)?)
    }
}

/// Callback context object during [`MIPSOL`](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html).
pub struct MIPSolCtx<'a>(CbCtx<'a>);
impl<'a> MIPSolCtx<'a> {
    /// This method is a no-op. It was added to this type by mistake but is kept for backwards-compatibility.
    #[doc(hidden)]
    #[deprecated(note = "This method does nothing, use `MIPNodeCtx::add_cut` instead.")]
    pub fn add_cut(&self, _constr: IneqExpr) -> Result<()> {
        eprintln!("MIPSolCtx::add_cut is a no-op, use MIPNodeCtx::add_cut instead.");
        Ok(())
    }

    /// Retrieve the new (integer) solution values for the given variables.  This will query the solution for ALL
    /// variables, and return the subset provided, so you should avoid calling this method multiple times per callback.
    pub fn get_solution<I, V>(&self, vars: I) -> Result<Vec<f64>>
    where
        V: Borrow<Var>,
        I: IntoIterator<Item = V>,
    {
        self.0.get_mip_solution(vars)
    }

    impl_common! {}
    impl_set_solution! {}
    impl_runtime! {}
    impl_add_lazy! {}
    impl_getter! { open_scenarios, i32, MIPSOL, MIPSOL_OPENSCENARIOS, "Number of scenarios that are still open in a multi-scenario model." }
    impl_getter! { obj, f64, MIPSOL, MIPSOL_OBJ, "Objective value for the new solution." }
    impl_getter! { obj_best, f64, MIPSOL, MIPSOL_OBJBST, "Current best objective." }
    impl_getter! { obj_bnd, f64, MIPSOL, MIPSOL_OBJBND, "Current best objective bound." }
    impl_getter! { node_cnt, f64, MIPSOL, MIPSOL_NODCNT, "Current explored node count." }
    impl_getter! { sol_cnt, i32, MIPSOL, MIPSOL_SOLCNT, "Current count of feasible solutions found." }

    /// Current algorithmic phase in the MIP solution
    pub fn phase(&self) -> Result<MipPhase> {
        MipPhase::from_raw(self.0.get_int(MIPSOL, MIPSOL_PHASE)?)
    }
}

/// Callback context object during [`MIPNODE`](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html).
pub struct MIPNodeCtx<'a>(CbCtx<'a>);
impl<'a> MIPNodeCtx<'a> {
    /// Add a new (linear) cutting plane to the MIP model.
    pub fn add_cut(&self, constr: IneqExpr) -> Result<()> {
        self.0.add_cut(constr)
    }

    /// Optimization status of current MIP node.
    pub fn status(&self) -> Result<Status> {
        self.0
            .get_int(MIPNODE, MIPNODE_STATUS)
            .map(|s| s.try_into().unwrap())
    }

    /// Get the optimal solution to this MIP node relaxation.  This will query the solution for ALL variables, and
    /// return the subset provided, so you should avoid calling this method multiple times per callback.
    pub fn get_solution<I, V>(&self, vars: I) -> Result<Vec<f64>>
    where
        V: Borrow<Var>,
        I: IntoIterator<Item = V>,
    {
        self.0.get_node_rel(vars)
    }

    /// Current algorithmic phase in the MIP solution
    pub fn phase(&self) -> Result<MipPhase> {
        MipPhase::from_raw(self.0.get_int(MIPNODE, MIPNODE_PHASE)?)
    }

    impl_set_solution! {}
    impl_common! {}
    impl_runtime! {}
    impl_add_lazy! {}
    impl_getter! { open_scenarios, i32, MIPNODE, MIPNODE_OPENSCENARIOS, "Number of scenarios that are still open in a multi-scenario model." }
    impl_getter! { obj_best, f64, MIPNODE, MIPNODE_OBJBST, "Current best objective." }
    impl_getter! { obj_bnd, f64, MIPNODE, MIPNODE_OBJBND, "Current best objective bound." }
    impl_getter! { node_cnt, f64, MIPNODE, MIPNODE_NODCNT, "Current explored node count." }
    impl_getter! { sol_cnt, i32, MIPNODE, MIPNODE_SOLCNT, "Current count of feasible solutions found." }
}

/// Callback context object during [`MESSAGE`](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html).
pub struct MessageCtx<'a>(CbCtx<'a>);
impl<'a> MessageCtx<'a> {
    /// The message about to be logged
    pub fn message(&self) -> Result<String> {
        self.0
            .get_string(MESSAGE, MSG_STRING)
            .map(|s| s.trim().to_owned())
    }

    impl_common! {}
}

/// Callback context object during [`BARRIER`](https://www.gurobi.com/documentation/9.1/refman/cb_codes.html).
pub struct BarrierCtx<'a>(CbCtx<'a>);
impl<'a> BarrierCtx<'a> {
    impl_common! {}
    impl_runtime! {}
    impl_getter! { iter_cnt, i32, BARRIER, BARRIER_ITRCNT, "Current simplex iteration count." }
    impl_getter! { prim_obj, f64, BARRIER, BARRIER_PRIMOBJ, "Primal objective value for current barrier iterate." }
    impl_getter! { dual_obj, f64, BARRIER, BARRIER_DUALOBJ, "Dual objective value for current barrier iterate." }
    impl_getter! { prim_inf, f64, BARRIER, BARRIER_PRIMINF, "Primal infeasibility for current barrier iterate." }
    impl_getter! { dual_inf, f64, BARRIER, BARRIER_DUALINF, "Dual infeasibility for current barrier iterate." }
    impl_getter! { compl_viol, f64, BARRIER, BARRIER_COMPL, "Complementarity violation for current barrier iterate." }
}

fn negative_int_to_none(val: i32) -> Option<u32> {
    if val < 0 {
        None
    } else {
        Some(val as u32)
    }
}

/// Callback context object during [`IIS`](https://www.gurobi.com/documentation/9.5/refman/cb_codes.html).
pub struct IISCtx<'a>(CbCtx<'a>);
impl<'a> IISCtx<'a> {
    impl_common! {}
    impl_runtime! {}
    impl_getter! { constr_min, i32, IIS, IIS_CONSTRMIN, "Minimum number of constraints in the IIS."}
    impl_getter! { constr_max, i32, IIS, IIS_CONSTRMAX, "Maximum number of constraints in the IIS."}
    impl_getter! { constr_guess, i32 => Option<u32>, IIS, IIS_CONSTRGUESS,
        "Estimated number of constraints in the IIS.",
        negative_int_to_none
    }
    impl_getter! { bound_min, i32, IIS, IIS_BOUNDMIN, "Minimum number of variable bounds in the IIS."}
    impl_getter! { bound_max, i32, IIS, IIS_BOUNDMAX, "Maximum number of variable bounds in the IIS."}
    impl_getter! { bound_guess, i32 => Option<u32>, IIS, IIS_BOUNDGUESS,
        "Estimated number of variable bounds in the IIS.",
        negative_int_to_none
    }
}

/// TODO: (medium) add MultiObj ctx
/// The argument given to callbacks.
#[allow(missing_docs)]
#[non_exhaustive]
pub enum Where<'a> {
    Polling(PollingCtx<'a>),
    PreSolve(PreSolveCtx<'a>),
    Simplex(SimplexCtx<'a>),
    MIP(MIPCtx<'a>),
    MIPSol(MIPSolCtx<'a>),
    MIPNode(MIPNodeCtx<'a>),
    Message(MessageCtx<'a>),
    Barrier(BarrierCtx<'a>),
    IIS(IISCtx<'a>),
}

//
impl Where<'_> {
    fn new<'a>(ctx: CbCtx<'a>) -> Result<Where<'a>> {
        let w = match ctx.where_raw {
            POLLING => Where::Polling(PollingCtx(ctx)),
            PRESOLVE => Where::PreSolve(PreSolveCtx(ctx)),
            SIMPLEX => Where::Simplex(SimplexCtx(ctx)),
            MIP => Where::MIP(MIPCtx(ctx)),
            MIPNODE => Where::MIPNode(MIPNodeCtx(ctx)),
            MIPSOL => Where::MIPSol(MIPSolCtx(ctx)),
            MESSAGE => Where::Message(MessageCtx(ctx)),
            BARRIER => Where::Barrier(BarrierCtx(ctx)),
            IIS => Where::IIS(IISCtx(ctx)),
            _ => {
                return Err(Error::NotYetSupported(format!("WHERE = {}", ctx.where_raw)));
            }
        };
        Ok(w)
    }
}

/// Possible values for `ctx.phase()`
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum MipPhase {
    /// Currently in the `NoRel` heuristic
    NoRel,
    /// Standard MIP Search
    Search,
    /// Performing MIP improvement
    Improve,
}

impl MipPhase {
    fn from_raw(val: i32) -> Result<Self> {
        match val {
            0 => Ok(MipPhase::NoRel),
            1 => Ok(MipPhase::Search),
            2 => Ok(MipPhase::Improve),
            _ => Err(Error::NotYetSupported("unknown phase".to_string())),
        }
    }
}

/// The context object for Gurobi callback.
struct CbCtx<'a> {
    where_raw: i32,
    cbdata: *mut ffi::c_void,
    model: &'a Model,
    nvars: usize,
}

impl<'a> CbCtx<'a> {
    pub(crate) fn new(
        cbdata: *mut ffi::c_void,
        where_raw: i32,
        model: &'a Model,
        nvars: usize,
    ) -> Self {
        CbCtx {
            cbdata,
            where_raw,
            model,
            nvars,
        }
    }

    pub fn proceed(&mut self) {
        unsafe {
            let model = self.model.as_mut_ptr();
            ffi::GRBcbproceed(model);
        }
    }

    /// Retrieve node relaxation solution values at the current node.
    pub fn get_node_rel<I, V>(&self, vars: I) -> Result<Vec<f64>>
    where
        V: Borrow<Var>,
        I: IntoIterator<Item = V>,
    {
        // memo: only MIPNode && status == Optimal
        // note that this MUST be after a call to model.update(), so the indices in model.vars are Added and the unwrap() is ok
        let vals = self.get_double_array_vars(MIPNODE, MIPNODE_REL)?;
        vars.into_iter()
            .map(|v| Ok(vals[self.model.get_index(v.borrow())? as usize]))
            .collect()
    }

    /// Retrieve values from the current solution vector.
    pub fn get_mip_solution<I, V>(&self, vars: I) -> Result<Vec<f64>>
    where
        V: Borrow<Var>,
        I: IntoIterator<Item = V>,
    {
        let vals = self.get_double_array_vars(MIPSOL, MIPSOL_SOL)?;
        vars.into_iter()
            .map(|v| Ok(vals[self.model.get_index(v.borrow())? as usize]))
            .collect()
    }

    /// Provide a new feasible solution for a MIP model.  Not all variables need to be given.
    pub fn set_solution<I, V, T>(&self, solution: I) -> Result<Option<f64>>
    where
        V: Borrow<Var>,
        T: Borrow<f64>,
        I: IntoIterator<Item = (V, T)>,
    {
        let mut soln = vec![GRB_UNDEFINED; self.model.get_attr(crate::attr::NumVars)? as usize];
        for (i, val) in solution {
            soln[self.model.get_index_build(i.borrow())? as usize] = *val.borrow();
        }
        let mut obj = INFINITY as raw::c_double;
        self.check_apicall(unsafe {
            ffi::GRBcbsolution(self.cbdata, soln.as_ptr(), &mut obj as *mut raw::c_double)
        })?;

        let obj = if obj == INFINITY || obj == GRB_UNDEFINED {
            None
        } else {
            Some(obj)
        };

        Ok(obj)
    }

    /// Retrieve the elapsed solver runtime in seconds.
    pub fn get_runtime(&self) -> Result<f64> {
        self.get_double(self.where_raw, RUNTIME)
    }

    /// Add a new cutting plane to the MIP model.
    pub fn add_cut(&self, constr: IneqExpr) -> Result<()> {
        // note the user can still provide a LinExpr containing vars from a different model, so unwrap() won't work
        let (lhs, sense, rhs) = constr.into_normalised_linear()?;
        let (inds, coeff) = self.model.get_coeffs_indices_build(&lhs)?;

        self.check_apicall(unsafe {
            ffi::GRBcbcut(
                self.cbdata,
                coeff.len() as ffi::c_int,
                inds.as_ptr(),
                coeff.as_ptr(),
                sense as ffi::c_char,
                rhs,
            )
        })
    }

    /// Add a new lazy constraint to the MIP model.
    pub fn add_lazy(&self, constr: IneqExpr) -> Result<()> {
        let (lhs, sense, rhs) = constr.into_normalised_linear()?;
        let (inds, coeff) = self.model.get_coeffs_indices_build(&lhs)?;
        self.check_apicall(unsafe {
            ffi::GRBcblazy(
                self.cbdata,
                coeff.len() as ffi::c_int,
                inds.as_ptr(),
                coeff.as_ptr(),
                sense as ffi::c_char,
                rhs,
            )
        })
    }

    pub fn terminate(&self) {
        self.model.terminate()
    }

    fn get_int(&self, where_: i32, what: i32) -> Result<i32> {
        let mut buf = 0i32;
        self.check_apicall(unsafe {
            ffi::GRBcbget(
                self.cbdata,
                where_,
                what,
                &mut buf as *mut i32 as *mut raw::c_void,
            )
        })
        .and(Ok(buf))
    }

    fn get_double(&self, where_: i32, what: i32) -> Result<f64> {
        let mut buf = 0.0f64;
        self.check_apicall(unsafe {
            ffi::GRBcbget(
                self.cbdata,
                where_,
                what,
                &mut buf as *mut f64 as *mut raw::c_void,
            )
        })
        .and(Ok(buf))
    }

    fn get_double_array_vars(&self, where_: i32, what: i32) -> Result<Vec<f64>> {
        let mut buf = vec![0.0; self.nvars];
        self.check_apicall(unsafe {
            ffi::GRBcbget(
                self.cbdata,
                where_,
                what,
                buf.as_mut_ptr() as *mut raw::c_void,
            )
        })
        .and(Ok(buf))
    }

    fn get_string(&self, where_: i32, what: i32) -> Result<String> {
        let mut buf = null();
        self.check_apicall(unsafe {
            ffi::GRBcbget(
                self.cbdata,
                where_,
                what,
                &mut buf as *mut *const i8 as *mut raw::c_void,
            )
        })
        .and(Ok(unsafe { util::copy_c_str(buf) }))
    }

    fn check_apicall(&self, error: ffi::c_int) -> Result<()> {
        if error != 0 {
            return Err(Error::FromAPI("Callback error".to_owned(), 40000));
        }
        Ok(())
    }
}
