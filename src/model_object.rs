use std::hash::Hash;
use fnv::FnvHashMap;

use gurobi_sys as ffi;
use crate::Model;
use crate::{Error, Result};

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
use std::fmt::Debug;

pub trait ModelObject: ModelObjectPrivate + Debug {
  fn id(&self) -> u32;
}


macro_rules! create_model_obj_ty {
    ($t:ident, $model_attr:ident, $delfunc:path) => {
      #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
      pub struct $t {
        pub(crate) id : u32,
        pub(crate) model_id: u32,
      }

      impl ModelObjectPrivate for $t {
        fn from_raw(id: u32, model_id: u32) -> $t {
          Self{ id, model_id }
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

        fn model_id(&self) -> u32 { self.model_id }
      }

      impl ModelObject for $t {
        fn id(&self) -> u32 { self.id }
      }

    };
}

create_model_obj_ty!(Var, vars, ffi::GRBdelvars);
create_model_obj_ty!(Constr, constrs, ffi::GRBdelconstrs);
create_model_obj_ty!(QConstr, qconstrs, ffi::GRBdelqconstrs);
create_model_obj_ty!(SOS, sos, ffi::GRBdelsos);

#[derive(Debug, Clone, Copy)]
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
pub struct IdxManager<T: Hash + Eq> {
  update_model: bool,
  update_action: UpdateAction,
  next_id: u32,
  model_id: u32,
  order: Vec<T>,
  lookup: FnvHashMap<T, IdxState>,
}


impl<T: ModelObject> IdxManager<T> {
  pub(crate) fn new(model_id: u32) -> IdxManager<T> {
    let order = Vec::new();
    let lookup = FnvHashMap::default();
    IdxManager { order, lookup, model_id, next_id: 0, update_action: UpdateAction::Noop, update_model: false }
  }

  fn mark_update_action(&mut self, a: UpdateAction) {
    if a > self.update_action {
      self.update_action = a
    }
  }

  pub(crate) fn get_index(&self, o: &T) -> Result<i32> {
    if let Some(state) = self.lookup.get(o) {
      match *state {
        IdxState::Removed(_) => Err(Error::ModelObjectRemoved),
        IdxState::Pending | IdxState::Build(_) => Err(Error::ModelObjectPending),
        IdxState::Present(idx) => Ok(idx)
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
        IdxState::Present(idx) | IdxState::Build(idx) | IdxState::Removed(idx) => Ok(idx)
      }
    } else if o.model_id() == self.model_id {
      Err(Error::ModelObjectRemoved)
    } else {
      Err(Error::ModelObjectMismatch)
    }
  }

  pub(crate) fn model_update_needed(&self) -> bool { self.update_model }

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
      IdxState::Present(idx) => { *state = IdxState::Removed(idx) }
      IdxState::Removed(_) => return Err(Error::ModelObjectRemoved),
    }
    self.update_model = true;
    self.mark_update_action(UpdateAction::Rebuild);
    Ok(())
  }

  pub fn add_new(&mut self, update_lazy: bool) -> T {
    let o = T::from_raw(self.next_id, self.model_id);
    self.next_id += 1;
    self.mark_update_action(UpdateAction::Fix);
    let state = if update_lazy {
      IdxState::Pending
    } else {
      IdxState::Build(self.lookup.len() as i32)
    };
    self.update_model = true;
    self.lookup.insert(o, state);
    self.order.push(o);
    o
  }

  pub(crate) fn update(&mut self) {
    use std::collections::hash_map::Entry;

    match self.update_action {
      UpdateAction::Noop => {}

      UpdateAction::Fix => { // O(k) where k is the number of elements that need to be updated
        let mut k = self.order.len() as i32 - 1;
        for var in self.order.iter().rev() {
          let state = self.lookup.get_mut(var).unwrap();
          match *state {
            IdxState::Removed(_) => unreachable!(),
            IdxState::Pending => {
              *state = IdxState::Present(k)
            }
            IdxState::Build(idx) => {
              debug_assert_eq!(idx, k);
              *state = IdxState::Present(k)
            }
            IdxState::Present(_) => break
          }
          k -= 1;
        }
      }

      UpdateAction::Rebuild => { // O(n) where n is the total number of elements.
        let mut k = 0i32;
        let order = &mut self.order;
        let lookup = &mut self.lookup;
        order.retain(|&o|
          match lookup.entry(o) {
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
          }
        );
        debug_assert_eq!(k as usize, self.lookup.len());
      }
    }

    debug_assert_eq!(self.lookup.len(), self.lookup.len());
    self.update_model = false;
    self.update_action = UpdateAction::Noop;
  }

  pub fn is_empty(&self) -> bool { self.lookup.is_empty() }

  pub fn len(&self) -> usize {
    assert!(self.update_action != UpdateAction::Rebuild);
    self.lookup.len()
  }
}
