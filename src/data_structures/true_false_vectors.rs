/*!

Silly convenience container that keeps "true" things in one vector and "false" things in another.

 */

use std::ops::{Index, IndexMut};


pub struct TFVectors<T> {
  pub trues : T,
  pub falses: T
}

impl<T> TFVectors<T> {

  pub fn new() -> Self {
    Self::default()
  }

  fn get(&self, truth_value: bool) -> &T {
    if truth_value {
      &self.trues
    } else {
      &self.falses
    }
  }

  fn get_mut(&mut self, truth_value: bool) -> &mut T{
    if truth_value {
      &mut self.trues
    } else {
      &mut self.falses
    }
  }
}

impl<T: Default> Default for TFVectors<T> {
  fn default() -> Self {
    Self {
      trues : T::default(),
      falses: T::default()
    }
  }
}

impl<T: Default> TFVectors<T> {
  pub fn new() -> Self {
    Self::default()
  }
}

  impl<'a, T> Index<bool> for TFVectors<T>{
  type Output = T;

  fn index(&self, index: bool) -> &Self::Output {
    self.get(index)
  }
}

impl<'a, T> IndexMut<bool> for TFVectors<T> {
  fn index_mut(&mut self, index: bool) -> &mut Self::Output {
    self.get_mut(index)
  }
}

impl<'a, T> Index<usize> for TFVectors<T>{
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    self.get(index == 0)
  }
}

impl<'a, T> IndexMut<usize> for TFVectors<T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    self.get_mut(index == 0)
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
