/*!

  More or less standard stochastic local search. The important bits are in the `local_search` submodule
(`crate::local_search::local_search`). See that module for details.

*/


mod constraint;
mod variable_info;
mod config;
pub(crate) mod local_search;

use std::default::Default;


use crate::{
  BoolVariableVector,
  Literal,
  LiteralVector,
  log::log_assert,
};


// Re-exports
pub use config::LocalSearchConfig;
pub use local_search::{
  LocalSearchCore,
  LocalSearch
};




// region Module-private structs

/// Pseudo-boolean Coefficient
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
struct PbCoefficient {
  constraint_id: u32,
  coefficient  : u32,
}
type CoefficientVector = Vec<PbCoefficient>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
struct LocalSearchStatistics {
  count_of_flips   : usize,
  count_of_restarts: usize,
}
impl LocalSearchStatistics {
  pub fn reset(&mut self) {
    self.count_of_flips    = 0;
    self.count_of_restarts = 0;
  }
  pub fn new(&mut self) -> Self {
    Self::default()
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum LocalSearchMode {
  GSAT,
  WSAT
}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
