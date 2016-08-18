
///
pub trait Shape: Copy {
  fn size(&self) -> usize;
  fn names(&self, name: &str) -> Vec<String>;
}

impl Shape for () {
  fn size(&self) -> usize { 1 }
  fn names(&self, name: &str) -> Vec<String> { vec![name.to_string()] }
}

impl Shape for (usize) {
  fn size(&self) -> usize { *self }
  fn names(&self, name: &str) -> Vec<String> { (0..(*self)).map(|i| format!("{}[{}]", name, i)).collect() }
}

impl Shape for (usize, usize) {
  fn size(&self) -> usize { self.0 * self.1 }
  fn names(&self, name: &str) -> Vec<String> {
    (0..(self.0)).zip((0..(self.1))).map(|(i, j)| format!("{}[{}][{}]", name, i, j)).collect()
  }
}

impl Shape for (usize, usize, usize) {
  fn size(&self) -> usize { self.0 * self.1 * self.2 }
  fn names(&self, name: &str) -> Vec<String> {
    (0..(self.0))
      .zip((0..(self.1)))
      .zip((0..(self.2)))
      .map(|((i, j), k)| format!("{}[{}][{}][{}]", name, i, j, k))
      .collect()
  }
}


/// represents a tensor object which contains an array of values with type T.
pub trait Tensor<T, S: Shape> {
  fn shape(&self) -> Option<S>;
  fn body(&self) -> &Vec<T>;
}


pub struct TensorVal<T, S: Shape>(Vec<T>, S);

impl<T, S: Shape> TensorVal<T, S> {
  pub fn new(body: Vec<T>, shape: S) -> TensorVal<T, S> { TensorVal(body, shape) }
}

impl<T, S: Shape> Tensor<T, S> for TensorVal<T, S> {
  fn shape(&self) -> Option<S> { Some(self.1) }
  fn body(&self) -> &Vec<T> { &self.0 }
}
