use crate::{Literal, LiteralVector};

#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct Constraint {
  pub(crate) id: usize,
  pub(crate) k: usize,
  pub(crate) slack: i64,
  pub(crate) literals: LiteralVector,
}

impl Constraint{
  pub(crate) fn new(k: usize, id: usize) -> Self{
    Self{
      id,
      k,
      ..Self::default()
    }
  }
  fn push(&mut self, literal: Literal) {
    self.literals.push(literal)
  }
  fn size(&self) -> usize {
    self.literals.len()
  }
  fn operator(&self, idx: usize) -> Literal {
    self.literals[idx]
  }
  pub(crate) fn iter(&self) -> std::slice::Iter<'_, Literal> {
    self.literals.iter()
  }

}
