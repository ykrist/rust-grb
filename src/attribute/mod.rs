//! [Gurobi Attributes](https://www.gurobi.com/documentation/9.1/refman/attributes.html) for models,
//! constraints and variables.
//!
//! Setting or querying the wrong attribute for an object will result in an [`Error::FromAPI`](crate::Error::FromAPI).

#[allow(unused_imports)] // false positive - used in macros
use std::ptr::{null, null_mut};
use std::ffi::CString;
use std::iter::IntoIterator;
use std::fmt;
use std::convert::TryInto;

use grb_sys::{c_int, c_char};

use crate::model_object::*;
use crate::{Result, Model, VarType, ModelSense, Status, ConstrSense};
use crate::util::{AsPtr, copy_c_str};

mod attr_enums;
#[doc(inline)]
pub use attr_enums::enum_exports::*;
#[doc(inline)]
pub use attr_enums::variant_exports as attr;

mod private {
  use super::*;

  pub trait IntAttr {}
  pub trait CharAttr {}
  pub trait StrAttr {}
  pub trait DoubleAttr {}

  pub trait ObjAttr {
    type Obj: ModelObject;
  }

  pub trait AttrName: fmt::Debug {
    fn name(&self) -> CString {
      let s = format!("{:?}", &self);
      // We know the debug repr of these enum variants won't contain nul bytes or non-ascii chars.
      unsafe { CString::from_vec_unchecked(s.into_bytes()) }
    }
  }
}

use private::*;


/// A marker trait used to implement [`ObjAttrSet`] for both [`String`] and `&str`
pub trait StringLike: Into<Vec<u8>> {}

impl StringLike for String {}
impl<'a> StringLike for &'a str {}

impl<T: ObjAttr + fmt::Debug> AttrName for T {}

pub trait ObjAttrGet<O, V> {
  fn get(&self, model: &Model, idx: i32) -> Result<V>;
  fn get_batch<I: IntoIterator<Item=Result<i32>>>(&self, model: &Model, idx: I) -> Result<Vec<V>>;
}

pub trait ObjAttrSet<O, V> {
  fn set(&self, model: &Model, idx: i32, val: V) -> Result<()>;
  fn set_batch<I: IntoIterator<Item=(Result<i32>, V)>>(&self, model: &Model, idx_val_pairs: I) -> Result<()>;
}


macro_rules! impl_obj_get {
  ($t:ty, $default:expr, $get:path, $getbatch:path) => {
    fn get(&self, model: &Model, idx: i32) -> Result<$t> {
      let mut val = $default;
      unsafe {
        let m = model.as_mut_ptr();
        let code = $get(m, self.name().as_ptr(), idx, &mut val);
        model.check_apicall(code)?;
      }
      Ok(val)
    }

    fn get_batch<I: IntoIterator<Item=Result<i32>>>(&self, model: &Model, inds: I) -> Result<Vec<$t>> {
      let inds : Result<Vec<_>> = inds.into_iter().collect();
      let inds = inds?;
      let mut vals = vec![$default; inds.len()];

      unsafe {
        model.check_apicall($getbatch(
          model.as_mut_ptr(), self.name().as_ptr(), inds.len() as c_int, inds.as_ptr(), vals.as_mut_ptr()
        ))?;
      }

      Ok(vals)
    }

  };
}

macro_rules! impl_obj_set {
  ($t:ty, $default:expr, $set:path, $setbatch:path) => {
    fn set(&self, model: &Model, idx: i32, val: $t) -> Result<()> {
      unsafe {
        let m = model.as_mut_ptr();
        let code = $set(m, self.name().as_ptr(), idx, val);
        model.check_apicall(code)
      }
    }

    fn set_batch<I: IntoIterator<Item=(Result<i32>, $t)>>(&self, model: &Model, idx_val_pairs: I) -> Result<()> {
      let idx_val_pairs = idx_val_pairs.into_iter();
      let size_hint = idx_val_pairs.size_hint().0;
      let mut inds = Vec::with_capacity(size_hint);
      let mut vals = Vec::with_capacity(size_hint);

      for (i,v) in idx_val_pairs {
        inds.push(i?);
        vals.push(v);
      }

      unsafe {
        model.check_apicall($setbatch(
          model.as_mut_ptr(), self.name().as_ptr(), inds.len() as c_int, inds.as_ptr(), vals.as_ptr()
          ))?;
      }

      Ok(())
    }
  };
}

/// Generate getter methods for a custom-type attribute (eg Sense or VType which have the
/// ConstrSense and VarType enum types respectively)
macro_rules! impl_obj_get_custom {
  ($t:path, $default:expr, $get:path, $getbatch:path) => {
    fn get(&self, model: &Model, idx: i32) -> Result<$t> {
      let mut val = $default;
      unsafe {
        let m = model.as_mut_ptr();
        let code = $get(m, self.name().as_ptr(), idx, &mut val);
        model.check_apicall(code)?;
      }
      Ok(val.try_into().unwrap())
    }

    fn get_batch<I: IntoIterator<Item=Result<i32>>>(&self, model: &Model, inds: I) -> Result<Vec<$t>> {
      let inds : Result<Vec<_>> = inds.into_iter().collect();
      let inds = inds?;
      let mut vals = vec![$default; inds.len()];

      unsafe {
        model.check_apicall($getbatch(
          model.as_mut_ptr(), self.name().as_ptr(), inds.len() as c_int, inds.as_ptr(), vals.as_mut_ptr()
        ))?;
      }

      let vals = vals.into_iter().map(|ch| (ch as c_char).try_into().unwrap()).collect();
      Ok(vals)
    }
  };
}

impl<A> ObjAttrGet<A::Obj, i32> for A where
  A: IntAttr + AttrName + ObjAttr,
{
  impl_obj_get! { i32, i32::MIN, grb_sys::GRBgetintattrelement, grb_sys::GRBgetintattrlist }
}


impl<A> ObjAttrSet<A::Obj, i32> for A where
  A: IntAttr + AttrName + ObjAttr,
{
  impl_obj_set! { i32, i32::MIN, grb_sys::GRBsetintattrelement, grb_sys::GRBsetintattrlist }
}

impl<A> ObjAttrGet<A::Obj, f64> for A where
  A: DoubleAttr + AttrName + ObjAttr,
{
  impl_obj_get! { f64, f64::MIN, grb_sys::GRBgetdblattrelement, grb_sys::GRBgetdblattrlist }
}


impl<A> ObjAttrSet<A::Obj, f64> for A where
  A: DoubleAttr + AttrName + ObjAttr,
{
  impl_obj_set! { f64, f64::MIN, grb_sys::GRBsetdblattrelement, grb_sys::GRBsetdblattrlist }
}


impl<A> ObjAttrGet<A::Obj, c_char> for A where
  A: CharAttr + AttrName + ObjAttr,
{
  impl_obj_get! { c_char, 0i8, grb_sys::GRBgetcharattrelement, grb_sys::GRBgetcharattrlist }
}


impl<A> ObjAttrSet<A::Obj, c_char> for A where
  A: CharAttr + AttrName + ObjAttr,
{
  impl_obj_set! { c_char, 0i8, grb_sys::GRBsetcharattrelement, grb_sys::GRBsetcharattrlist }
}

impl AttrName for VarVTypeAttr {}
impl ObjAttrSet<Var, c_char> for VarVTypeAttr {
  impl_obj_set! { c_char, 0i8, grb_sys::GRBsetcharattrelement, grb_sys::GRBsetcharattrlist }
}



impl ObjAttrSet<Var, VarType> for VarVTypeAttr {
  fn set(&self, model: &Model, idx: i32, val: VarType) -> Result<()> {
    self.set(model, idx, val as c_char)
  }
  fn set_batch<I: IntoIterator<Item=(Result<i32>, VarType)>>(&self, model: &Model, idx_val_pairs: I) -> Result<()> {
    self.set_batch(model, idx_val_pairs.into_iter().map(|(idx, vt)| (idx, vt as c_char)))
  }
}

impl ObjAttrGet<Var, VarType> for VarVTypeAttr {
  impl_obj_get_custom!{ VarType, 0i8, grb_sys::GRBgetcharattrelement, grb_sys::GRBgetcharattrlist}
}

impl AttrName for ConstrSenseAttr {}
impl ObjAttrSet<Constr, c_char> for ConstrSenseAttr {
  impl_obj_set! { c_char, 0i8, grb_sys::GRBsetcharattrelement, grb_sys::GRBsetcharattrlist }
}

impl ObjAttrSet<Constr, ConstrSense> for ConstrSenseAttr {
  fn set(&self, model: &Model, idx: i32, val: ConstrSense) -> Result<()> {
    self.set(model, idx, val as c_char)
  }
  fn set_batch<I: IntoIterator<Item=(Result<i32>, ConstrSense)>>(&self, model: &Model, idx_val_pairs: I) -> Result<()> {
    self.set_batch(model, idx_val_pairs.into_iter().map(|(idx, vt)| (idx, vt as c_char)))
  }
}

impl ObjAttrGet<Constr, ConstrSense> for ConstrSenseAttr {
  impl_obj_get_custom!{ ConstrSense, 0i8, grb_sys::GRBgetcharattrelement, grb_sys::GRBgetcharattrlist}
}


/// From the Gurobi manual regarding string attributes:
///
/// Note that all interface routines that return string-valued attributes are returning pointers into internal
/// Gurobi data structures. The user should copy the contents of the pointer to a different data structure before
/// the next call to a Gurobi library routine. The user should also be careful to never modify the data pointed to
/// by the returned character pointer.
impl<A> ObjAttrGet<A::Obj, String> for A where
  A: StrAttr + AttrName + ObjAttr,
{
  fn get(&self, model: &Model, idx: i32) -> Result<String> {
    unsafe {
      let mut s: grb_sys::c_str = std::ptr::null();
      model.check_apicall(grb_sys::GRBgetstrattrelement(
        model.as_mut_ptr(), self.name().as_ptr(), idx, &mut s,
      ))?;
      Ok(copy_c_str(s))
    }
  }

  fn get_batch<I: IntoIterator<Item=Result<i32>>>(&self, model: &Model, idx: I) -> Result<Vec<String>> {
    let inds: Result<Vec<_>> = idx.into_iter().collect();
    let inds = inds?;

    unsafe {
      let mut cstrings: Vec<*const c_char> = vec![std::ptr::null(); inds.len()];
      model.check_apicall(grb_sys::GRBgetstrattrlist(
        model.as_mut_ptr(), self.name().as_ptr(), inds.len() as c_int,
        inds.as_ptr(), cstrings.as_mut_ptr(),
      ))?;

      let strings = cstrings.into_iter()
        .map(|s| copy_c_str(s))
        .collect();
      Ok(strings)
    }
  }
}


impl<'a, A, T> ObjAttrSet<A::Obj, T> for A where
  A: StrAttr + AttrName + ObjAttr,
  T: StringLike
{
  fn set(&self, model: &Model, idx: i32, val: T) -> Result<()> {
    let val = CString::new(val)?;
    unsafe {
      model.check_apicall(grb_sys::GRBsetstrattrelement(
        model.as_mut_ptr(), self.name().as_ptr(), idx, val.as_ptr(),
      ))?
    }
    Ok(())
  }

  fn set_batch<I: IntoIterator<Item=(Result<i32>, T)>>(&self, model: &Model, idx_val_pairs: I) -> Result<()> {
    let idx_val_pairs = idx_val_pairs.into_iter();
    let size_hint = idx_val_pairs.size_hint().0;
    let mut inds = Vec::with_capacity(size_hint);
    let mut cstrings = Vec::with_capacity(size_hint);
    let mut cstr_ptrs = Vec::with_capacity(size_hint);

    for (i, s) in idx_val_pairs {
      let cs = CString::new(s)?;
      inds.push(i?);
      cstr_ptrs.push(cs.as_ptr());
      cstrings.push(cs);
    }

    unsafe {
      model.check_apicall(grb_sys::GRBsetstrattrlist(
        model.as_mut_ptr(), self.name().as_ptr(), inds.len() as c_int,
        inds.as_ptr(), cstr_ptrs.as_ptr(),
      ))
    }
  }
}


pub trait ModelAttrGet<V> {
  fn get(&self, model: &Model) -> Result<V>;
}

pub trait ModelAttrSet<V> {
  fn set(&self, model: &Model, val: V) -> Result<()>;
}


macro_rules! impl_model_attr {
  ($target:path, $t:ty, $default:expr, $get:path, $set:path) => {
    impl AttrName for $target {}

    impl ModelAttrGet<$t> for $target {
      fn get(&self, model: &Model) -> Result<$t> {
        let mut val = $default;
        unsafe {
          model.check_apicall($get(
            model.as_mut_ptr(), self.name().as_ptr(), &mut val,
          ))?
        }
        Ok(val)
      }
    }

    impl ModelAttrSet<$t> for $target {
      fn set(&self, model: &Model, val: $t) -> Result<()> {
        unsafe {
          model.check_apicall($set(
            model.as_mut_ptr(), self.name().as_ptr(), val,
          ))
        }
      }
    }

  };
}

impl_model_attr! { ModelIntAttr, i32, i32::MIN, grb_sys::GRBgetintattr, grb_sys::GRBsetintattr }
impl_model_attr! { ModelDoubleAttr, f64, f64::NAN, grb_sys::GRBgetdblattr, grb_sys::GRBsetdblattr }

impl AttrName for ModelStrAttr {}

impl ModelAttrGet<String> for ModelStrAttr {
  fn get(&self, model: &Model) -> Result<String> {
    unsafe {
      let mut val: *const c_char = null_mut();
      model.check_apicall(grb_sys::GRBgetstrattr(
        model.as_mut_ptr(), self.name().as_ptr(), &mut val,
      ))?;
      Ok(copy_c_str(val))
    }
  }
}

impl<T: Into<Vec<u8>>> ModelAttrSet<T> for ModelStrAttr {
  fn set(&self, model: &Model, val: T) -> Result<()> {
    let val = CString::new(val)?;
    unsafe {
      model.check_apicall(grb_sys::GRBsetstrattr(
        model.as_mut_ptr(), self.name().as_ptr(), val.as_ptr(),
      ))
    }
  }
}

impl AttrName for ModelModelSenseAttr {}
impl ModelAttrGet<ModelSense> for ModelModelSenseAttr {
  fn get(&self, model: &Model) -> Result<ModelSense> {
    let mut val = i32::MIN;
    unsafe {
      model.check_apicall(grb_sys::GRBgetintattr(
        model.as_mut_ptr(), self.name().as_ptr(), &mut val,
      ))?
    }
    Ok(val.try_into().unwrap())
  }
}

impl ModelAttrSet<i32> for ModelModelSenseAttr {
  fn set(&self, model: &Model, val: i32) -> Result<()> {
    unsafe {
      model.check_apicall(grb_sys::GRBsetintattr(
        model.as_mut_ptr(), self.name().as_ptr(), val,
      ))
    }
  }
}


impl ModelAttrSet<ModelSense> for ModelModelSenseAttr {
  fn set(&self, model: &Model, val: ModelSense) -> Result<()> {
    self.set(model, val as i32)
  }
}

impl AttrName for ModelStatusAttr {}

impl ModelAttrGet<Status> for ModelStatusAttr {
  fn get(&self, model: &Model) -> Result<Status> {
    let mut val = i32::MIN;
    unsafe {
      model.check_apicall(grb_sys::GRBgetintattr(
        model.as_mut_ptr(), self.name().as_ptr(), &mut val,
      ))?
    }
    Ok(val.try_into().unwrap())
  }
}

