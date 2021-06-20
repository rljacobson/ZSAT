/*!
An `OredIntegerSet` is an implementation of an `ApproximateSet` that uses bitwise Or to accumulate
integer members in a single index value. The fewer the elements it "contains", the more accurate it
is. This implementation is about as quick and dirty as possible without just using a `HashSet`.

Possible alternatives:
  * Bloom filter: https://crates.io/crates/bloom
  * Cuckoo filter: https://crates.io/crates/cuckoofilter
  * XOr filter: https://crates.io/crates/xorf
*/

use std::ops::{
  BitAndAssign,
  BitOrAssign,
  SubAssign
};

use num_traits::{PrimInt, Unsigned};

use super::ApproximateSet;

pub struct OredIntegerSet<IndexType, MemberType>
  where IndexType: PrimInt + Unsigned,
        MemberType: Into<IndexType>
{
  index: IndexType // The internal representation of the set.
}

impl<ValueType, MemberType> ApproximateSet<MemberType> for OredIntegerSet<ValueType, MemberType>
  where ValueType: PrimInt + Unsigned,
        MemberType: Into<ValueType>
{
  fn new() -> Self{
    Self{
      index: ValueType::zero()
    }
  }

  fn with_value(value: &MemberType) -> Self {
    let mut set = Self::new();
    set.insert(value);
    set
  }

  fn with_values(values: Vec<MemberType>) -> Self {
    let mut set = Self::new();
    for i in values {
      set.insert(i);
    }
    set
  }

  fn insert(&mut self, value: &MemberType) {
    self.index |= value.into();
  }

  fn may_contain(&self, value: &MemberType) -> bool {
    (self.index & value.into()) != ValueType::zero()
  }

  fn make_union(a: &Self, b: &Self) -> Self{
    Self{
      index: a.index | b.index
    }
  }

  fn make_intersection(a: &Self, b: &Self) -> Self{
    Self{
      index: a.index & b.index
    }
  }

  fn empty(&self) -> bool {
    self.index == ValueType::zero()
  }

  fn may_equal(&self, other: &Self) -> bool {
    self.index == other.index
  }

  fn equivalent(&self, other: &Self) -> bool {
    self.index == other.index
  }

  fn reset(&mut self) {
    self.index = ValueType::zero();
  }

}

impl<SetType, T> BitOrAssign for OredIntegerSet<SetType, T>
  where SetType: PrimInt + Unsigned,
        T: Into<SetType>{
  fn bitor_assign(&mut self, rhs: Self) {
    self.index |= rhs.index;
  }
}

impl<SetType, T> BitAndAssign for OredIntegerSet<SetType, T>
  where SetType: PrimInt + Unsigned,
        T: Into<SetType>{
  fn bitand_assign(&mut self, rhs: Self) {
    self.index &= rhs.index;
  }
}

impl<SetType, T> SubAssign for OredIntegerSet<SetType, T>
  where SetType: PrimInt + Unsigned,
        T: Into<SetType>{
  fn sub_assign(&mut self, rhs: Self) {
    self.index &= !rhs.index;
  }
}