
mod solver;
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
mod log;
mod data_structures;
mod watched;
mod clause;
mod parameters;
mod parallel;


// Re-exported items
pub use data_structures::{OredIntegerSet, Statistic, Statistics};
pub use errors::Error;
pub use lifted_bool::LiftedBool;
pub use literal::{Literal, LiteralVector};
pub use model::Model;
pub use resource_limit::{
  ResourceLimit,
  ScopedResourceLimit,
  ScopedSuspendedResourceLimit,
};
pub use solver::Solver;



/// This library tries to track a specific version of z3 and reports this version, e.g., on some fatal errors in
/// debug mode.
const Z3_FULL_VERSION: &str = "1.2.3";

// todo: Should any of these be a newtype?
// todo: Seems like `BoolVariable` should be a `usize`, as it's used as an index. It might be a `u32` for the sake of
//       spacial economy.

/// A bool variable $x_j$ has corresponding literals $x_j$ and $\overline{x}_j$. We represent
/// $x_j$ by $j$ and $\overline{x}_j$ by $\overline{j}$.
pub type BoolVariable                 = usize;                               // u32;
pub const NULL_BOOL_VAR: BoolVariable = BoolVariable::MAX >> 1;
pub type BoolVariableVector           = Vec<BoolVariable>;
pub type VariableApproximateSet       = OredIntegerSet<usize, BoolVariable>;
pub type ExtensionConstraintIndex     = usize;
pub type ExternalJustificationIndex   = usize;
pub type Theory                       = i32;                                 // todo: Why not u32?
pub type UIntSet                      = bit_set::BitSet;


/*
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Debug, Hash)]
pub struct DimacsLiteral(Literal);

impl Display for DimacsLiteral {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    unimplemented!()
  }
}
*/

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
