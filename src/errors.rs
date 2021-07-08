
// todo: Is a flat error type hierarchy sufficient?

use thiserror::Error as DeriveError;
use user_error::UFE;

#[derive(Clone, Eq, PartialEq, Debug, Hash, DeriveError)]
pub enum Error {
  #[error("A Solver Error occurred.")]
  Solver,
  #[error("A SAT Parameter Error occurred.")]
  SATParameter,

  #[error("Local search is incomplete with extensions beyond PB.")]
  IncompleteExtension,

  #[error("Module has no parameters file or file not found.")]
  DeserializeParameters,

  // todo: Is this a real error or is it an Unknown error?
  #[error("A Default Error occurred.")]
  Default,

  #[error(transparent)]
  Unknown {source: Box<dyn std::error::Error>}
}

// Spurious "trait bound `ZSATError: Error` is not satisfied" error. The trait bound is derived
// using `thiserror::Error`.
impl UFE for Error { /* User Facing Error - nothing to implement. */}
