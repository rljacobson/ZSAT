/*!



 */


use std::collections::HashSet;
use std::error::Error;
use std::sync::Mutex;

use crate::{Literal, LiteralVector, ResourceLimit, Solver};
use crate::clause::{Clause, ClauseVector};
use crate::data_structures::{VectorIndexSet, VectorPool};
use crate::log::log_at_level;



// todo: Is this something that can be replaced with a standard utility struct?
pub struct Parallel<'a, 'b> {
  units   : LiteralVector,
  unit_set: VectorIndexSet,
  literals: LiteralVector,
  pool    : VectorPool,
  mux     : Mutex<VectorPool>, // What is the mutex guarding? `self`?

  // For exchange with local search:
  num_clauses   : usize,
  solver_copy   : Box<Solver<'a>>, // Scoped Pointer
  consumer_ready: bool,
  priorities    : Vec<f64>,

  rlimit : ResourceLimit,       // Scoped Resource Limit
  limits : Vec<ResourceLimit>,
  solvers: Vec<Box<Solver<'b>>> // Vector of solver pointers, might need to be Rcs
}

impl<'a, 'b> Parallel<'a, 'b> {

  pub fn new(&self, solver: &Solver) -> Self {}

  fn enable_add(c: &Clause) -> bool {}

  pub fn init_solvers(&self, s: &solver, num_extra_solvers: usize);

  pub fn push_child(&self, rl: &reslimit);

  // reserve space
  pub fn reserve(&mut self, num_owners: usize, sz: usize) { self.pool.reserve(sz: num_owners); }

  pub fn get_solver(&self, i: usize) -> Solver { return *self.solvers[i]; }

  pub fn cancel_solver(&self, i: usize) { self.limits[i].cancel(); }

  // exchange unit literals
  pub fn exchange(&self, s: &solver, &const in: literal_vector, limit: &usize, out: &literal_vector);

  // Add the clause to the shared clause pool.
  pub fn share_clause(&self, s: &solver, &const c: clause){}
  // Add the two-literal clause to the shared clause pool.
  pub fn share_literals(&self, s: &solver, l1: literal, l2: literal){}

  // receive clauses from shared clause pool
  pub fn get_clauses(&self, s: &solver) {}

  // exchange from solver state to local search and back.
  pub fn from_solver(&self, s: &solver) {}
  pub fn to_solver(&self, s: &solver) -> bool {}

  pub fn from_local_search(&self, s: &i_local_search) -> bool {}
  pub fn to_local_search(&self, s: &i_local_search) {}

  pub fn copy_solver(&self, s: &solver) -> bool {}

}



#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
