/*!
Provides a trait `ApproximateSet` for a set data structure that determines membership only
probabilistically. The `ApproximateSet` holds values of type `T`.
*/

pub trait ApproximateSet<T> {
  fn new() -> Self;
  fn with_value(value: &T) -> Self;
  fn with_values(values: Vec<T>) -> Self;
  fn insert(&mut self, value: &T);
  fn may_contain(&self, value: &T) -> bool;
  fn must_not_contain(&self, value: &T) -> bool {
    !self.may_contain(value)
  }
  fn make_union(a: &Self, b: &Self) -> Self;
  fn make_intersection(a: &Self, b: &Self) -> Self;
  /// Tests if `self` is empty.
  fn empty(&self) -> bool;
  fn must_not_subset(&self, other: &Self) -> bool {
    !self.may_subset(other)
  }
  fn must_not_equal(&self, other: &Self) -> bool {
    !self.may_equal(other)
  }
  fn may_equal(&self, other: &Self) -> bool;
  /// Determines if `self` and `other` are the same approximate set.
  fn equivalent(&self, other: &Self) -> bool;
  fn may_subset(lhs: &Self, rhs: &Self) -> bool {
    let union = Self::make_union(lhs, rhs);
    rhs.equivalent(&union)
  }
  /// Sets `self` to thw empty set in-place.
  fn reset(&mut self);
  /// Tests whether the intersection of `self` and `other` is empty.
  fn empty_intersection(&self, other: &Self) -> bool {
    Self::make_intersection(self, other).empty()
  }
}
