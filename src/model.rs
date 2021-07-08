/*!

  A `Model` maps `BoolVariable`s to their respective truth values. A `Model` is really just a
  wrapper for a vector of `LiftedBool`s indexed by `BoolVariable`s (`u32`s).

*/

use crate::{
  LiftedBool,
  BoolVariable,
  Literal
};
use std::fmt::{Formatter, Display};
use std::ops::{Index, Not};
use std::borrow::Borrow;
use itertools::Itertools;

pub struct Model {
  assignments: Vec<LiftedBool>
}

impl Display for Model {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let stringified: String = self.assignments
                          .iter()
                          .enumerate()
                          .filter(| (i, lb) | lb != LiftedBool::Undefined)
                          .map(| (i, lb) |
                            if lb == LiftedBool::True {
                              format!("{}", i)
                            } else {
                              format!("-{}", i)
                            }
                          )
                          .collect()
                          .join(" ");
    write!(f, "{}", stringified)
  }
}

impl Index<BoolVariable> for Model{
  type Output = LiftedBool;

  fn index(&self, index: BoolVariable) -> &Self::Output {
    self.assignments.get(index).unwrap()
  }
}

impl Index<Literal> for Model{
  type Output = LiftedBool;

  fn index(&self, index: Literal) -> &Self::Output {
    let mut val = self[index.var()];
    if index.sign() {
      &val
    } else {
      &val.not()
    }
  }
}

impl Model {

  pub fn clear(&mut self){
    self.assignments.clear()
  }

  pub fn push(&mut self, value: LiftedBool) {
    self.assignments.push(value);
  }

}

pub fn value_of_bool_variable(var: BoolVariable, model: &Model) -> LiftedBool {
  model[var.into()]
}

pub fn value_of_literal(literal: Literal, model: &Model) -> LiftedBool {
  let result = model[literal.var().into()];
  match literal.sign() {
    true  => -result,
    false => result
  }
}




#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
