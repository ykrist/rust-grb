//! [Gurobi Attributes](https://www.gurobi.com/documentation/9.1/refman/attributes.html) for models,
//! constraints and variables.
//!
//! Setting or querying the wrong attribute for an object will result in an [`Error::FromAPI`](crate::Error::FromAPI).

use std::convert::TryInto;
use std::ffi::CString;
use std::iter::IntoIterator;
#[allow(unused_imports)] // false positive - used in macros
use std::ptr::{null, null_mut};

use cstr_enum::AsCStr;
use grb_sys2::{c_char, c_int};
use grb_sys2 as ffi;

use crate::model_object::*;
use crate::util::{copy_c_str, AsPtr};
use crate::{ConstrSense, Model, ModelSense, Result, Status, VarType};

#[allow(missing_docs)]
mod attr_enums {
    include!(concat!(env!("OUT_DIR"), "/attr_enums.rs"));
    // generated code - see build/main.rs
}
#[doc(inline)]
pub use attr_enums::enum_exports::*;
#[doc(inline)]
pub use attr_enums::variant_exports as attr;

mod private {
    use super::*;
    pub trait Attr {}

    pub trait IntAttr {}
    pub trait CharAttr {}
    pub trait StrAttr {}
    pub trait DoubleAttr {}

    pub trait ObjAttr {
        type Obj: ModelObject;
    }
}

use private::*;

/// A marker trait for internal blanket implementations.
pub trait StringLike: Into<Vec<u8>> {}

impl StringLike for String {}
impl<'a> StringLike for &'a str {}

/// A queryable [`ModelObject`] attribute (eg [`Var`] or [`Constr`])
pub trait ObjAttrGet<O, V> {
    /// Get the value for this attribute
    fn get(&self, model: &Model, idx: i32) -> Result<V>;
    /// Get multiple values for this attribute at once
    fn get_batch<I: IntoIterator<Item = Result<i32>>>(
        &self,
        model: &Model,
        idx: I,
    ) -> Result<Vec<V>>;
}

/// A modifiable [`ModelObject`] attribute (eg [`Var`] or [`Constr`])
pub trait ObjAttrSet<O, V> {
    /// Set the value for this attribute
    fn set(&self, model: &Model, idx: i32, val: V) -> Result<()>;
    /// Set multiple values for this attribute at once
    fn set_batch<I: IntoIterator<Item = (Result<i32>, V)>>(
        &self,
        model: &Model,
        idx_val_pairs: I,
    ) -> Result<()>;
}

macro_rules! impl_obj_get {
    ($t:ty, $default:expr, $get:path, $getbatch:path) => {
        fn get(&self, model: &Model, idx: i32) -> Result<$t> {
            let mut val = $default;
            unsafe {
                let m = model.as_mut_ptr();
                let code = $get(m, self.as_cstr().as_ptr(), idx, &mut val);
                model.check_apicall(code)?;
            }
            Ok(val)
        }

        fn get_batch<I: IntoIterator<Item = Result<i32>>>(
            &self,
            model: &Model,
            inds: I,
        ) -> Result<Vec<$t>> {
            let inds: Result<Vec<_>> = inds.into_iter().collect();
            let inds = inds?;
            let mut vals = vec![$default; inds.len()];

            unsafe {
                model.check_apicall($getbatch(
                    model.as_mut_ptr(),
                    self.as_cstr().as_ptr(),
                    inds.len() as c_int,
                    inds.as_ptr(),
                    vals.as_mut_ptr(),
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
                let code = $set(m, self.as_cstr().as_ptr(), idx, val);
                model.check_apicall(code)
            }
        }

        fn set_batch<I: IntoIterator<Item = (Result<i32>, $t)>>(
            &self,
            model: &Model,
            idx_val_pairs: I,
        ) -> Result<()> {
            let idx_val_pairs = idx_val_pairs.into_iter();
            let size_hint = idx_val_pairs.size_hint().0;
            let mut inds = Vec::with_capacity(size_hint);
            let mut vals = Vec::with_capacity(size_hint);

            for (i, v) in idx_val_pairs {
                inds.push(i?);
                vals.push(v);
            }

            unsafe {
                model.check_apicall($setbatch(
                    model.as_mut_ptr(),
                    self.as_cstr().as_ptr(),
                    inds.len() as c_int,
                    inds.as_ptr(),
                    vals.as_ptr(),
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
                let code = $get(m, self.as_cstr().as_ptr(), idx, &mut val);
                model.check_apicall(code)?;
            }
            Ok(val.try_into().unwrap())
        }

        fn get_batch<I: IntoIterator<Item = Result<i32>>>(
            &self,
            model: &Model,
            inds: I,
        ) -> Result<Vec<$t>> {
            let inds: Result<Vec<_>> = inds.into_iter().collect();
            let inds = inds?;
            let mut vals = vec![$default; inds.len()];

            unsafe {
                model.check_apicall($getbatch(
                    model.as_mut_ptr(),
                    self.as_cstr().as_ptr(),
                    inds.len() as c_int,
                    inds.as_ptr(),
                    vals.as_mut_ptr(),
                ))?;
            }

            let vals = vals
                .into_iter()
                .map(|ch| (ch as c_char).try_into().unwrap())
                .collect();
            Ok(vals)
        }
    };
}

impl<A> ObjAttrGet<A::Obj, i32> for A
where
    A: IntAttr + ObjAttr + AsCStr,
{
    impl_obj_get! { i32, i32::MIN, ffi::GRBgetintattrelement, ffi::GRBgetintattrlist }
}

impl<A> ObjAttrSet<A::Obj, i32> for A
where
    A: IntAttr + ObjAttr + AsCStr,
{
    impl_obj_set! { i32, i32::MIN, ffi::GRBsetintattrelement, ffi::GRBsetintattrlist }
}

impl<A> ObjAttrGet<A::Obj, f64> for A
where
    A: DoubleAttr + ObjAttr + AsCStr,
{
    impl_obj_get! { f64, f64::MIN, ffi::GRBgetdblattrelement, ffi::GRBgetdblattrlist }
}

impl<A> ObjAttrSet<A::Obj, f64> for A
where
    A: DoubleAttr + ObjAttr + AsCStr,
{
    impl_obj_set! { f64, f64::MIN, ffi::GRBsetdblattrelement, ffi::GRBsetdblattrlist }
}

impl<A> ObjAttrGet<A::Obj, c_char> for A
where
    A: CharAttr + ObjAttr + AsCStr,
{
    impl_obj_get! { c_char, 0i8, ffi::GRBgetcharattrelement, ffi::GRBgetcharattrlist }
}

impl<A> ObjAttrSet<A::Obj, c_char> for A
where
    A: CharAttr + ObjAttr + AsCStr,
{
    impl_obj_set! { c_char, 0i8, ffi::GRBsetcharattrelement, ffi::GRBsetcharattrlist }
}

impl ObjAttrSet<Var, c_char> for VarVTypeAttr {
    impl_obj_set! { c_char, 0i8, ffi::GRBsetcharattrelement, ffi::GRBsetcharattrlist }
}

impl ObjAttrSet<Var, VarType> for VarVTypeAttr {
    fn set(&self, model: &Model, idx: i32, val: VarType) -> Result<()> {
        self.set(model, idx, val as c_char)
    }
    fn set_batch<I: IntoIterator<Item = (Result<i32>, VarType)>>(
        &self,
        model: &Model,
        idx_val_pairs: I,
    ) -> Result<()> {
        self.set_batch(
            model,
            idx_val_pairs
                .into_iter()
                .map(|(idx, vt)| (idx, vt as c_char)),
        )
    }
}

impl ObjAttrGet<Var, VarType> for VarVTypeAttr {
    impl_obj_get_custom! { VarType, 0i8, ffi::GRBgetcharattrelement, ffi::GRBgetcharattrlist}
}

impl ObjAttrSet<Constr, c_char> for ConstrSenseAttr {
    impl_obj_set! { c_char, 0i8, ffi::GRBsetcharattrelement, ffi::GRBsetcharattrlist }
}

impl ObjAttrSet<Constr, ConstrSense> for ConstrSenseAttr {
    fn set(&self, model: &Model, idx: i32, val: ConstrSense) -> Result<()> {
        self.set(model, idx, val as c_char)
    }
    fn set_batch<I: IntoIterator<Item = (Result<i32>, ConstrSense)>>(
        &self,
        model: &Model,
        idx_val_pairs: I,
    ) -> Result<()> {
        self.set_batch(
            model,
            idx_val_pairs
                .into_iter()
                .map(|(idx, vt)| (idx, vt as c_char)),
        )
    }
}

impl ObjAttrGet<Constr, ConstrSense> for ConstrSenseAttr {
    impl_obj_get_custom! { ConstrSense, 0i8, ffi::GRBgetcharattrelement, ffi::GRBgetcharattrlist}
}

/// From the Gurobi manual regarding string attributes:
///
/// Note that all interface routines that return string-valued attributes are returning pointers into internal
/// Gurobi data structures. The user should copy the contents of the pointer to a different data structure before
/// the next call to a Gurobi library routine. The user should also be careful to never modify the data pointed to
/// by the returned character pointer.
impl<A> ObjAttrGet<A::Obj, String> for A
where
    A: StrAttr + AsCStr + ObjAttr,
{
    fn get(&self, model: &Model, idx: i32) -> Result<String> {
        unsafe {
            let mut s: ffi::c_str = std::ptr::null();
            model.check_apicall(ffi::GRBgetstrattrelement(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                idx,
                &mut s,
            ))?;
            if s.is_null() { return Ok(String::new()) }
            Ok(copy_c_str(s))
        }
    }

    fn get_batch<I: IntoIterator<Item = Result<i32>>>(
        &self,
        model: &Model,
        idx: I,
    ) -> Result<Vec<String>> {
        let inds: Result<Vec<_>> = idx.into_iter().collect();
        let inds = inds?;

        unsafe {
            let mut cstrings: Vec<*const c_char> = vec![std::ptr::null(); inds.len()];
            model.check_apicall(ffi::GRBgetstrattrlist(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                inds.len() as c_int,
                inds.as_ptr(),
                cstrings.as_mut_ptr(),
            ))?;

            let strings = cstrings.into_iter().map(|s| copy_c_str(s)).collect();
            Ok(strings)
        }
    }
}

impl<'a, A, T> ObjAttrSet<A::Obj, T> for A
where
    A: StrAttr + AsCStr + ObjAttr,
    T: StringLike,
{
    fn set(&self, model: &Model, idx: i32, val: T) -> Result<()> {
        let val = CString::new(val)?;
        unsafe {
            model.check_apicall(ffi::GRBsetstrattrelement(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                idx,
                val.as_ptr(),
            ))?
        }
        Ok(())
    }

    fn set_batch<I: IntoIterator<Item = (Result<i32>, T)>>(
        &self,
        model: &Model,
        idx_val_pairs: I,
    ) -> Result<()> {
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
            model.check_apicall(ffi::GRBsetstrattrlist(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                inds.len() as c_int,
                inds.as_ptr(),
                cstr_ptrs.as_ptr(),
            ))
        }
    }
}

/// A queryable [`Model`] attribute
pub trait ModelAttrGet<V> {
    /// Query the value for this attribute
    fn get(&self, model: &Model) -> Result<V>;
}

/// A modifiable [`Model`] attribute
pub trait ModelAttrSet<V> {
    /// Set a new value for this attribute
    fn set(&self, model: &Model, val: V) -> Result<()>;
}

macro_rules! impl_model_attr {
    ($target:path, $t:ty, $default:expr, $get:path, $set:path) => {
        impl ModelAttrGet<$t> for $target {
            fn get(&self, model: &Model) -> Result<$t> {
                let mut val = $default;
                unsafe {
                    model.check_apicall($get(
                        model.as_mut_ptr(),
                        self.as_cstr().as_ptr(),
                        &mut val,
                    ))?
                }
                Ok(val)
            }
        }

        impl ModelAttrSet<$t> for $target {
            fn set(&self, model: &Model, val: $t) -> Result<()> {
                unsafe {
                    model.check_apicall($set(model.as_mut_ptr(), self.as_cstr().as_ptr(), val))
                }
            }
        }
    };
}

impl_model_attr! { ModelIntAttr, i32, i32::MIN, ffi::GRBgetintattr, ffi::GRBsetintattr }
impl_model_attr! { ModelDoubleAttr, f64, f64::NAN, ffi::GRBgetdblattr, ffi::GRBsetdblattr }

impl ModelAttrGet<String> for ModelStrAttr {
    fn get(&self, model: &Model) -> Result<String> {
        unsafe {
            let mut val: *const c_char = null_mut();
            model.check_apicall(ffi::GRBgetstrattr(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                &mut val,
            ))?;
            if val.is_null() { return Ok(String::new()) }
            Ok(copy_c_str(val))
        }
    }
}

impl<T: Into<Vec<u8>>> ModelAttrSet<T> for ModelStrAttr {
    fn set(&self, model: &Model, val: T) -> Result<()> {
        let val = CString::new(val)?;
        unsafe {
            model.check_apicall(ffi::GRBsetstrattr(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                val.as_ptr(),
            ))
        }
    }
}

impl ModelAttrGet<ModelSense> for ModelModelSenseAttr {
    fn get(&self, model: &Model) -> Result<ModelSense> {
        let mut val = i32::MIN;
        unsafe {
            model.check_apicall(ffi::GRBgetintattr(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                &mut val,
            ))?
        }
        Ok(val.try_into().unwrap())
    }
}

impl ModelAttrSet<i32> for ModelModelSenseAttr {
    fn set(&self, model: &Model, val: i32) -> Result<()> {
        unsafe {
            model.check_apicall(ffi::GRBsetintattr(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                val,
            ))
        }
    }
}

impl ModelAttrSet<ModelSense> for ModelModelSenseAttr {
    fn set(&self, model: &Model, val: ModelSense) -> Result<()> {
        self.set(model, val as i32)
    }
}

impl ModelAttrGet<Status> for ModelStatusAttr {
    fn get(&self, model: &Model) -> Result<Status> {
        let mut val = i32::MIN;
        unsafe {
            model.check_apicall(ffi::GRBgetintattr(
                model.as_mut_ptr(),
                self.as_cstr().as_ptr(),
                &mut val,
            ))?
        }
        Ok(val.try_into().unwrap())
    }
}


#[cfg(test)]
mod tests {
  use crate as grb;
  use super::*;
  use std::ffi::CStr;
  use std::marker::PhantomData;
  use crate::SOSType;

  #[derive(Debug, Clone)]
  struct Attribute<T>(CString, PhantomData<T>);

  impl<T> Attribute<T> {
    pub fn new(s: String) -> Self {
      let mut s = s;
      // s.push_str("_fofooooo");
      Attribute(CString::new(s).unwrap(), PhantomData)
    }
  }

  impl<T: ModelObject> Attribute<T> {
    pub fn get<V>(self, model: &Model, obj: &T) -> Option<crate::Error>
    where Self: ObjAttrGet<T, V>
    {
      model.get_obj_attr::<_, _, V>(self, obj).err()
    }
  }

  impl Attribute<Model> {
    pub fn get_model<V>(self, model: &Model) -> Option<crate::Error>
    where Self: ModelAttrGet<V>
    {
      model.get_attr::<_, V>(self).err()
    }
  }

  impl<T> AsCStr for Attribute<T> {
    fn as_cstr(&self) -> &CStr { &self.0 }
  }

  impl<T> IntAttr for Attribute<T> {}
  impl<T> DoubleAttr for Attribute<T> {}
  impl<T> StrAttr for Attribute<T> {}
  impl<T> CharAttr for Attribute<T> {}

  impl ObjAttr for Attribute<Var> {
    type Obj = Var;
  }

  impl ObjAttr for Attribute<Constr> {
    type Obj = Constr;
  }
  impl ObjAttr for Attribute<QConstr> {
    type Obj = QConstr;
  }
  impl ObjAttr for Attribute<SOS> {
    type Obj = SOS;
  }

  impl_model_attr! { Attribute<Model>, i32, i32::MIN, ffi::GRBgetintattr, ffi::GRBsetintattr }
  impl_model_attr! { Attribute<Model>, f64, f64::NAN, ffi::GRBgetdblattr, ffi::GRBsetdblattr }

  impl ModelAttrGet<String> for Attribute<Model> {
    fn get(&self, model: &Model) -> Result<String> {
      unsafe {
        let mut val: *const c_char = null_mut();
        model.check_apicall(ffi::GRBgetstrattr(
          model.as_mut_ptr(),
          self.as_cstr().as_ptr(),
          &mut val,
        ))?;
        if val.is_null() { return Ok(String::new()) }
        Ok(copy_c_str(val))
      }
    }
  }


  struct Helper<O> {
    obj: Attribute<O>,
  }

  macro_rules! helper {
    (
      $model:ident, $attrname:ident, $a:expr, $b:expr;
      $($ty_str:literal, $obj_str:literal, $obj:ident);+$(;)?
    ) => {
      match ($a, $b) {
        $(
          ($ty_str, $obj_str) => $model.get_obj_attr::<_, _, helper!(@VAL_TY $ty_str)>(Attribute::<helper!(@OBJ_TY $obj_str)>::new($attrname), &$obj).err(),
        )*
        _ => None,
      }
    };

    (@VAL_TY "dbl") => { f64 };
    (@VAL_TY "int") => { i32 };
    (@VAL_TY "str") => { String };

    (@OBJ_TY "var") => { Var };
    (@OBJ_TY "qcons") => { QConstr };
    (@OBJ_TY "cons") => { Constr };
    (@OBJ_TY "sos") => { SOS };
  }

  #[test]
  fn attribute_names() -> crate::Result<()> {
    let params : Vec<_> = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/build/attrs.csv")).unwrap()
      .lines()
      .skip(1)
      .map(|line| {
        let mut line = line.split(",");
        let param = line.next().unwrap();
        let ty = line.next().unwrap();
        let obj = line.next().unwrap();
        assert_eq!(line.next(), None);
        (param.to_string(), ty.to_string(), obj.to_string())
      })
      .collect();

    let mut model = crate::Model::new("test")?;
    let var = crate::add_ctsvar!(model)?;

    let x = crate::add_binvar!(model)?;
    let y = crate::add_binvar!(model)?;
    let constraint = model.add_constr("", crate::c!(var >= 1))?;
    let qconstraint = model.add_qconstr("", crate::c!(var*var >= 1))?;

    let sos = model.add_sos(vec![(x, 1.0), (y, 1.0)], SOSType::Ty1)?;
    model.optimize()?;

    for (a, ty, obj) in params {
      let ty = ty.as_str();
      let obj = obj.as_str();

      // Oh boy, this is ugly
      eprintln!("{}", &a);
      let err = match (ty, obj) {
        ("dbl", "var") => Attribute::new(a).get::<f64>(&model, &var),
        ("int", "var") => Attribute::new(a).get::<i32>(&model, &var),
        ("str", "var") => Attribute::new(a).get::<String>(&model, &var),

        ("dbl", "constr") => Attribute::new(a).get::<f64>(&model, &constraint),
        ("int", "constr") => Attribute::new(a).get::<i32>(&model, &constraint),
        ("str", "constr") => Attribute::new(a).get::<String>(&model, &constraint),

        ("dbl", "qconstr") => Attribute::new(a).get::<f64>(&model, &qconstraint),
        ("int", "qconstr") => Attribute::new(a).get::<i32>(&model, &qconstraint),
        ("str", "qconstr") => Attribute::new(a).get::<String>(&model, &qconstraint),
        ("chr", "qconstr") => Attribute::new(a).get::<c_char>(&model, &qconstraint),

        ("dbl", "sos") => Attribute::new(a).get::<f64>(&model, &sos),
        ("int", "sos") => Attribute::new(a).get::<i32>(&model, &sos),
        ("str", "sos") => Attribute::new(a).get::<String>(&model, &sos),

        ("dbl", "model") => Attribute::new(a).get_model::<f64>(&model),
        ("int", "model") => Attribute::new(a).get_model::<i32>(&model),
        ("str", "model") => Attribute::new(a).get_model::<String>(&model),

        ("custom", _) => None,

        _ => panic!("missing test for: {} {}", ty, obj),
      };

      if let Some(err) = err {
        match err {
          // Unable to retrieve attribute
          crate::Error::FromAPI(_, 10005) => {},
          // It isn't a multi-objective model
          crate::Error::FromAPI(_, 10008) => {},
          err => return Err(err)
        }
      }

    }

    Ok(())
  }

}
