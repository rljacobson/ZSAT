mod solver;
mod approximate_set;
mod literal;
mod lifted_bool;
mod errors;
mod resource_limit;
mod model;
mod status;
mod symbol_table;
mod local_search;
mod check_satisfiability;
mod missing_types;  // todo: delete this.
mod config;
mod statistics;
mod logging;


use std::fmt::{Display, Formatter};
use std::ops::Not;


// Re-exports

pub use literal::{Literal, LiteralVector};
pub use lifted_bool::LiftedBool;
pub use errors::Error;
pub use model::Model;
pub use resource_limit::{
  ResourceLimit,
  ScopedResourceLimit,
  ScopedSuspendedResourceLimit,
};
pub use solver::Solver;



// Type defs

// todo: Should any of these be a newtype?
pub type BoolVariable       = u32;  // todo: Seems like this should be a usize, as it's used as an index.
pub type BoolVariableVector = Vec<BoolVariable>;
pub type VarApproximateSet  = approximate_set::OredIntegerSet<u64, BoolVariable>;
pub const NULL_BOOL_VAR: BoolVariable = u32::MAX >> 1;

pub type ClauseOffset               = usize;
pub type ExternalConstraintIndex    = usize;
pub type ExternalJustificationIndex = usize;
pub type Theory                     = i32; // todo: Why not u32?

pub type UIntSet = bit_set::BitSet;

// Newtypes

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Debug, Hash)]
pub struct DimacsLiteral(Literal);

impl Display for DimacsLiteral {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    unimplemented!()
  }
}






#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
