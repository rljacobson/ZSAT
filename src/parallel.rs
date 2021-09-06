/*!



 */


use std::collections::HashSet;
use std::error::Error;
use std::sync::Mutex;

use crate::{Literal, LiteralVector, ResourceLimit, Solver};
use crate::clause::{Clause, ClauseVector};
use crate::data_structures::{VectorIndexSet, VectorPool};
use crate::log::log_at_level;
use crate::symbol_table::SymbolData;


// todo: Is this something that can be replaced with a standard utility struct?
pub struct Parallel<'a, 'b> {
  units   : LiteralVector,
  unit_set: VectorIndexSet,
  literals: LiteralVector,
  pool    : VectorPool,
  mux     : Mutex<VectorPool>, // What is the mutex guarding? `self`?

  // For exchange with local search:
  num_clauses   : usize,
  solver_copy   : Option<Box<Solver<'a>>>, // Scoped Pointer
  consumer_ready: bool,
  priorities    : Vec<f64>,

  resource_limit: ResourceLimit,       // Scoped Resource Limit
  limits : Vec<ResourceLimit>,
  solvers: Vec<Box<Solver<'b>>> // Vector of solver pointers, might need to be Rcs
}

impl<'a, 'b> Parallel<'a, 'b> {

  // Todo: Make this take a resource limit, not a solver
  pub fn new(&self, solver: &Solver) -> Self {
    Parallel {
      units   : LiteralVector::new(),
      unit_set: VectorIndexSet::new(),
      literals: LiteralVector::new(),
      pool    : VectorPool::default(),
      mux     : Mutex::new(VectorPool::default()), // What is the mutex guarding? `self`? `pool`?

      // For exchange with local search:
      num_clauses   : 0,
      solver_copy   : None, // Scoped Pointer
      consumer_ready: false,
      priorities    : Vec::new(),

      resource_limit: solver.resource_limit.clone(),
      limits        : Vec::new(),
      self.solvers       : Vec::new() // Vector of solver pointers, might need to be Rcs
    }
  }

  fn enable_add(c: &Clause) -> bool {
    /// GLUCOSE heuristic:
    /// https://www.ijcai.org/Proceedings/09/Papers/074.pdf
    /// Plingeling heuristic:
    /// https://epub.jku.at/obvulioa/content/titleinfo/5973528/full.pdf
    /// http://fmv.jku.at/papers/Biere-SAT-Competition-2013-Lingeling.pdf
    return (c.size() <= 40 && c.glue() <= 8) || c.glue() <= 2;
  }

  pub fn init_solvers(&mut self, s: &Solver, num_extra_solvers: usize){

    let num_threads = num_extra_solvers + 1;
    self.solvers.reserve(num_extra_solvers);
    self.limits.reserve(num_extra_solvers);
    let saved_phase: SymbolData = s.params.get_sym("phase", SymbolData::new("caching"));

    for i in 0..num_extra_solvers {
      s.params.set_uint("random_seed", s.rand());
      if i == 1 + num_threads/2 {
        s.params.set_sym("phase", symbol("random"));
      }
      self.solvers[i] = Box::new(Solver::from_params_limit(s.params.clone(), &self.limits[i]));
      self.solvers[i].copy(s, true);
      self.solvers[i].set_par(this, i);
      push_child(self.solvers[i].resource_limit());
    }
    s.set_par(self, num_extra_solvers);
    s.params.set_sym("phase", saved_phase);
  }

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
