use fnv::FnvHashMap;
use std::fmt::Debug;
use std::hash::Hash;

use crate::Model;
use crate::{Error, Result};
use grb_sys2 as ffi;

mod private_traits {
    use super::*;

    pub trait ModelObjectPrivate: Sized + Hash + Eq + Copy {
        fn from_raw(id: u32, model_id: u32) -> Self;
        fn idx_manager_mut(model: &mut Model) -> &mut IdxManager<Self>;
        fn idx_manager(model: &Model) -> &IdxManager<Self>;
        unsafe fn gurobi_remove(m: *mut ffi::GRBmodel, inds: &[i32]) -> ffi::c_int;
        fn model_id(&self) -> u32;
    }
}

use private_traits::ModelObjectPrivate;

/// This trait encompasses all Gurobi model objects: [`Var`], [`Constr`], [`QConstr`] and [`SOS`].
/// Each `ModelObject` is associated with a particular model, and can only be used with that model.
/// Each `ModelObject` also has a unique, fixed 32-bit ID.  Gurobi itself uses an `i32` to index objects
/// (only positive indices are used), so the 32-bit limitation is already there.  Note that IDs are only
/// guaranteed to be unique if the concrete types of the `ModelObject` are the same and the objects
/// belong to the same model.  For example, if `v` is a `Var` and `c` is a `Constr`, then `v` and `c` may
/// have the same ID.  Additionally, if `s` is also a `Var`, but doesn't belong to the same `Model` as `v`,
/// `s` and `v` may have the same ID.
pub trait ModelObject: ModelObjectPrivate + Debug {
    /// Retrieve the object's ID.
    fn id(&self) -> u32;
}

macro_rules! create_model_obj_ty {
    ($t:ident, $model_attr:ident, $delfunc:path, $doc:literal) => {
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        #[doc = $doc]
        pub struct $t {
            pub(crate) id: u32,
            pub(crate) model_id: u32,
        }

        impl ModelObjectPrivate for $t {
            fn from_raw(id: u32, model_id: u32) -> $t {
                Self { id, model_id }
            }

            fn idx_manager_mut(model: &mut Model) -> &mut IdxManager<$t> {
                &mut model.$model_attr
            }

            fn idx_manager(model: &Model) -> &IdxManager<$t> {
                &model.$model_attr
            }

            unsafe fn gurobi_remove(m: *mut ffi::GRBmodel, inds: &[i32]) -> ffi::c_int {
                $delfunc(m, inds.len() as i32, inds.as_ptr())
            }

            fn model_id(&self) -> u32 {
                self.model_id
            }
        }

        impl ModelObject for $t {
            fn id(&self) -> u32 {
                self.id
            }
        }
    };
}

create_model_obj_ty!(Var, vars, ffi::GRBdelvars,
  "A Gurobi variable.

  To interact with the attributes of a variable, use [`Model::get_obj_attr`] and [`Model::set_obj_attr`]"
  );
create_model_obj_ty!(Constr, constrs, ffi::GRBdelconstrs,
  "A linear constraint added to a [`Model`]

  To interact with the attributes of a constraint, use [`Model::get_obj_attr`] and [`Model::set_obj_attr`]"
);
create_model_obj_ty!(QConstr, qconstrs, ffi::GRBdelqconstrs,
"A quadratic constraint added to a [`Model`]

  To interact with the attributes of a constraint, use [`Model::get_obj_attr`] and [`Model::set_obj_attr`]"
);
create_model_obj_ty!(SOS, sos, ffi::GRBdelsos,
"An SOS constraint added to a [`Model`]

 To interact with the attributes of a constraint, use [`Model::get_obj_attr`] and [`Model::set_obj_attr`]"
);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum IdxState {
    Present(i32),
    // has been processed with a call to update()
    Build(i32),
    // has an index and can be used for building, and setting but not querying attributes
    Pending,
    // hasn't got an index yet, cannot be used for building, setting, or querying attributes
    Removed(i32), // object has been removed, but can still be used to build and set attributes.
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum UpdateAction {
    Noop = 0,
    Fix = 1,
    // Sweep from the back of the ordering, mapping Build(idx) to Present(idx)
    Rebuild = 2, // Rebuild the whole lookup
}

/// A struct to keep track of the index state of model objects such as `Var`, `Constr`, `QConstr` and `SOS`
/// It also maintains the absolute order of variables, with respect to the order
/// If variables have been removed, it is necessary to update to rebuild the lookup (see the `update` method).
/// The `update_action` field is an optimisation to avoid having to do this for "appends" (only adding new variables)
#[derive(Debug)]
#[doc(hidden)]
pub struct IdxManager<T: Hash + Eq> {
    update_model: bool,
    update_action: UpdateAction,
    next_id: u32,
    model_id: u32,
    order: Vec<T>,
    lookup: FnvHashMap<T, IdxState>,
}

impl<T: ModelObject> IdxManager<T> {
    pub(crate) fn new_with_existing_obj(model_id: u32, nobj: usize) -> IdxManager<T> {
        let mut im = IdxManager::new(model_id);
        for id in 0..nobj {
            let v = T::from_raw(id as u32, model_id);
            im.order.push(v);
            im.lookup.insert(v, IdxState::Present(id as i32));
        }
        im.next_id = nobj as u32;
        im
    }

    pub(crate) fn new(model_id: u32) -> IdxManager<T> {
        let order = Vec::new();
        let lookup = FnvHashMap::default();
        IdxManager {
            order,
            lookup,
            model_id,
            next_id: 0,
            update_action: UpdateAction::Noop,
            update_model: false,
        }
    }

    fn mark_update_action(&mut self, a: UpdateAction) {
        if a > self.update_action {
            self.update_action = a;
        }
    }

    pub(crate) fn get_index(&self, o: &T) -> Result<i32> {
        if let Some(state) = self.lookup.get(o) {
            match *state {
                IdxState::Removed(_) => Err(Error::ModelObjectRemoved),
                IdxState::Pending | IdxState::Build(_) => Err(Error::ModelObjectPending),
                IdxState::Present(idx) => Ok(idx),
            }
        } else if o.model_id() == self.model_id {
            Err(Error::ModelObjectRemoved)
        } else {
            Err(Error::ModelObjectMismatch)
        }
    }

    pub(crate) fn get_index_build(&self, o: &T) -> Result<i32> {
        if let Some(state) = self.lookup.get(o) {
            match *state {
                IdxState::Pending => Err(Error::ModelObjectPending),
                IdxState::Present(idx) | IdxState::Build(idx) | IdxState::Removed(idx) => Ok(idx),
            }
        } else if o.model_id() == self.model_id {
            Err(Error::ModelObjectRemoved)
        } else {
            Err(Error::ModelObjectMismatch)
        }
    }

    pub(crate) fn model_update_needed(&self) -> bool {
        self.update_model
    }

    pub(crate) fn objects(&self) -> &[T] {
        assert!(!self.update_model);
        self.order.as_slice()
    }

    pub(crate) fn remove(&mut self, o: T, _update_lazy: bool) -> Result<()> {
        if o.model_id() != self.model_id {
            return Err(Error::ModelObjectMismatch);
        }

        let state = self.lookup.get_mut(&o).ok_or(Error::ModelObjectRemoved)?;
        match *state {
            IdxState::Build(_) | IdxState::Pending => return Err(Error::ModelObjectPending),
            IdxState::Present(idx) => *state = IdxState::Removed(idx),
            IdxState::Removed(_) => return Err(Error::ModelObjectRemoved),
        }
        self.update_model = true;
        self.mark_update_action(UpdateAction::Rebuild);
        debug_assert_eq!(self.lookup.len(), self.order.len());
        Ok(())
    }

    pub fn add_new(&mut self, update_lazy: bool) -> T {
        debug_assert_eq!(self.lookup.len(), self.order.len());
        let o = T::from_raw(self.next_id, self.model_id);
        self.next_id += 1;
        self.mark_update_action(UpdateAction::Fix);
        let state = if update_lazy {
            IdxState::Pending
        } else {
            IdxState::Build(self.lookup.len() as i32)
        };
        self.update_model = true;
        #[cfg(debug_assertions)]
        {
            // can't do vec![self.add_new(_); 100], since this just clones a bunch of shit
            if let Some(other) = self.order.last() {
                let s = self.lookup[other];
                if s != IdxState::Pending {
                    assert_ne!(s, state);
                }
            }
        }

        self.lookup.insert(o, state);
        self.order.push(o);
        o
    }

    // debug helper
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    fn print_vars(&self) {
        println!("----------------------------------------------------");
        for o in &self.order {
            print!("{:?} ", self.lookup[o]);
        }
        println!();
    }

    pub(crate) fn update(&mut self) {
        debug_assert_eq!(self.lookup.len(), self.order.len());
        use std::collections::hash_map::Entry;

        match self.update_action {
            UpdateAction::Noop => {}

            UpdateAction::Fix => {
                // O(k) where k is the number of elements that need to be updated
                let mut k = self.order.len() as i32 - 1;
                for var in self.order.iter().rev() {
                    let state = self.lookup.get_mut(var).unwrap();
                    match *state {
                        IdxState::Removed(_) => unreachable!(),
                        IdxState::Pending => {
                            *state = IdxState::Present(k);
                        }
                        IdxState::Build(idx) => {
                            debug_assert_eq!(idx, k);
                            *state = IdxState::Present(k);
                        }
                        IdxState::Present(_) => break,
                    };
                    k -= 1;
                }
            }

            UpdateAction::Rebuild => {
                // O(n) where n is the total number of elements.
                let mut k = 0i32;
                let order = &mut self.order;
                let lookup = &mut self.lookup;
                order.retain(|&o| match lookup.entry(o) {
                    Entry::Vacant(_) => unreachable!("bug, should always have an entry in lookup"),
                    Entry::Occupied(mut e) => {
                        let state = *e.get();
                        match state {
                            IdxState::Present(_) | IdxState::Build(_) | IdxState::Pending => {
                                e.insert(IdxState::Present(k));
                                k += 1;
                                true
                            }
                            IdxState::Removed(_) => {
                                e.remove();
                                false
                            }
                        }
                    }
                });
                debug_assert_eq!(k as usize, self.lookup.len());
            }
        }

        debug_assert_eq!(self.lookup.len(), self.lookup.len());
        self.update_model = false;
        self.update_action = UpdateAction::Noop;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum Action {
        SwitchUpdateMode,
        AddVar(u8),
        RemoveVar(u8),
    }

    fn action_strat(num: usize) -> impl Strategy<Value = Vec<Action>> {
        let s = prop_oneof![
            Just(Action::SwitchUpdateMode),
            any::<u8>().prop_map(Action::AddVar),
            any::<u8>().prop_map(Action::RemoveVar),
        ];
        proptest::collection::vec(s, ..num)
    }

    fn state_machine(actions: Vec<Action>) {
        let mut update_mode_lazy = true;
        let mut vars: Vec<Option<Var>> = vec![None; u8::MAX as usize + 1];
        let mut idx_manager = IdxManager::new(0);
        for a in actions {
            match a {
                Action::SwitchUpdateMode => {
                    update_mode_lazy = !update_mode_lazy;
                    if !update_mode_lazy {
                        idx_manager.update(); // purge old pending states
                    }
                }
                Action::AddVar(i) => {
                    let i = i as usize;
                    let v = vars[i];
                    match v {
                        Some(v) => {
                            if !update_mode_lazy {
                                idx_manager.get_index_build(&v).unwrap();
                            }
                        }
                        None => {
                            vars[i] = Some(idx_manager.add_new(update_mode_lazy));
                        }
                    }
                }
                Action::RemoveVar(i) => {
                    let i = i as usize;
                    let v = vars[i];
                    match v {
                        Some(v) => match idx_manager.remove(v, update_mode_lazy) {
                            Ok(()) => vars[i] = None,
                            Err(e) => assert_eq!(e, Error::ModelObjectPending),
                        },
                        None => {}
                    }
                }
            }
        }
    }

    proptest! {
      #[test]
      fn fuzz(actions in action_strat(100)) {
        state_machine(actions);
      }
    }

    #[test]
    fn regressions() {
        use Action::*;
        state_machine(vec![AddVar(4), SwitchUpdateMode, AddVar(4)]);
        state_machine(vec![
            AddVar(0),
            AddVar(1),
            AddVar(2),
            RemoveVar(1),
            SwitchUpdateMode,
            AddVar(1),
        ]);
    }
}
