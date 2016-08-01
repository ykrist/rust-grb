extern crate gurobi_sys as ffi;

pub use ffi::{IntAttr, DoubleAttr, CharAttr, StringAttr};
pub use ffi::IntAttr::*;
pub use ffi::DoubleAttr::*;
pub use ffi::CharAttr::*;
pub use ffi::StringAttr::*;

use error::Result;

/// provides function to query/set the value of attributes.
pub trait HasAttr<Attr> {
  type Output;

  fn get(&self, attr: Attr) -> Result<Self::Output>;

  fn set(&mut self, attr: Attr, value: Self::Output) -> Result<()>;

  fn get_element(&self, attr: Attr, element: i32) -> Result<Self::Output>;

  fn set_element(&mut self,
                 attr: Attr,
                 element: i32,
                 value: Self::Output)
                 -> Result<()>;

  fn get_array(&self,
               attr: Attr,
               first: usize,
               len: usize)
               -> Result<Vec<Self::Output>>;

  fn set_array(&mut self,
               attr: Attr,
               first: usize,
               values: &[Self::Output])
               -> Result<()>;

  fn get_list(&self, attr: Attr, ind: &[i32]) -> Result<Vec<Self::Output>>;
  fn set_list(&mut self,
              attr: Attr,
              ind: &[i32],
              value: &[Self::Output])
              -> Result<()>;
}
