
// todo: Is a flat error type hierarchy sufficient?

use thiserror::Error as DeriveError;
use user_error::UFE;

#[derive(Clone, Eq, PartialEq, Debug, Hash, DeriveError)]
pub enum Error {
  #[error("A Solver Error occurred.")]
  SolverError,

  #[error(transparent)]
  UnknownError{source: Box<dyn std::error::Error>}
}

// Spurious "trait bound `ZSATError: Error` is not satisfied" error. The trait bound is derived
// using `thiserror::Error`.
impl UFE for Error {}