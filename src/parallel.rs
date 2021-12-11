/*!



 */


use std::collections::HashSet;
use std::error::Error;
use std::sync::Mutex;

use crate::parameters::ParameterValue;
use crate::{Literal, LiteralVector, ResourceLimit, Solver};
use crate::clause::{Clause, ClauseVector};
// use crate::data_structures::{VectorIndexSet, VectorPool, VectorIndex};
use crate::log::log_at_level;
use crate::symbol_table::SymbolData;
use std::borrow::BorrowMut;
use std::rc::Rc;
use crate::resource_limit::ArcRwResourceLimit;

type VectorIndexSet = HashSet<usize>;
type VectorIndex = usize;

// todo: figure out what derives VectorPool needs.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
struct VectorPool {
  /// The inner `Vec<VectorIndex>` represents the clause.
  vectors: Vec<Vec<VectorIndex>>,
  owners: Vec<VectorIndex>
}

impl VectorPool {
/*
  /// Given an `index` of the head of a vector, advances `index` to the head of the next vector. If the head of the last
  /// vector is provided, `index` is "wrapped around" to `0`, which is the head of the first vector.
  pub fn next(&self, index: &mut VectorIndex) {
    log_assert!(index < self.size);

    let n = index + 2 + self.get_length(*index);
    if n >= self.size {
      *index = 0;
    }
    else {
      *index = n;
    }
  }
 */

  /// Clears `vectors` and `owners` and reserves `thread_count` space in each vector.
  pub fn reserve(&mut self, thread_count: usize) {
    self.vectors.clear();
    self.vectors.reserve(thread_count);
    self.owners.clear();
    self.owners.reserve(thread_count);
  }

  pub fn add_vector(&mut self, owner: VectorIndex, vector: &Vec<VectorIndex>) {
    self.vectors.push(vector.clone());
    self.owner.push(owner);
  }

  /// Returns a pointer to the vector data of the last vector?
  pub fn get_vector_for_owner(&mut self, owner: VectorIndex)
    -> Option<&Vec<VectorIndex>>
  {
    let found = self.owners.iter().position(&owner);

    match found {

      Some(index) => Some(&self.vectors[index]),

      None => None

    }
  }

}

// todo: Is this something that can be replaced with a standard utility struct?
#[derive(Default, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Parallel<'a, 'b> {
  units   : LiteralVector,
  unit_set: VectorIndexSet,
  literals: LiteralVector,
  mux     : Mutex<VectorPool>,
  pool    : VectorPool,

  // For exchange with local search:
  num_clauses   : usize,
  solver_copy   : Option<Box<Solver<'a>>>, // Scoped Pointer
  consumer_ready: bool,
  priorities    : Vec<f64>,

  resource_limit: ArcRwResourceLimit,       // Scoped Resource Limit
  limits : Vec<ArcRwResourceLimit>,
  solvers: Vec<Rc<Solver<'b>>> // Vector of solver pointers, might need to be Rcs
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
      solvers       : Vec::new() // Vector of solver pointers, might need to be Rcs
    }
  }

  /// GLUCOSE heuristic:
  /// https://www.ijcai.org/Proceedings/09/Papers/074.pdf
  /// Plingeling heuristic:
  /// https://epub.jku.at/obvulioa/content/titleinfo/5973528/full.pdf
  /// http://fmv.jku.at/papers/Biere-SAT-Competition-2013-Lingeling.pdf
  fn enable_add(c: &Clause) -> bool {
    return (c.size() <= 40 && c.glue() <= 8) || c.glue() <= 2;
  }

  pub fn init_solvers(&mut self, solver: &mut Solver, num_extra_solvers: usize){

    let num_threads = num_extra_solvers + 1;
    self.solvers.reserve(num_extra_solvers);
    self.limits.reserve(num_extra_solvers);
    let saved_phase = solver.parameters.borrow().get_value("phase");//[("phase", SymbolData::new("caching"))];

    for i in 0..num_extra_solvers {
      solver.parameters["random_seed"] = solver.rand();
      if i == 1 + num_threads/2 {
        solver.parameters["phase"] = ParameterValue::Symbol("random");
      }
      self.solvers[i] = Rc::new(Solver::from_params_limit(solver.parameters.clone(), &self.limits[i]));
      self.solvers[i].copy(solver, true);
      self.solvers[i].set_parallel(self, i);
      self.push_child(self.solvers[i].resource_limit());
    }
    solver.set_par(self, num_extra_solvers);
    solver.parameters["phase"] = saved_phase;
  }

  pub fn push_child(&mut self, rl: ArcRwResourceLimit){ self.resource_limit.push_child(rl); }

  pub fn reserve(&mut self, num_owners: usize, size: usize) { self.pool.reserve(num_owners, size); }

  pub fn get_solver(&self, i: usize) -> Rc<Solver> { return self.solvers[i].clone(); }

  pub fn cancel_solver(&self, i: usize) { self.limits[i].cancel(); }

  // exchange unit literals
  pub fn exchange(
    &mut self,
    s: &mut Solver,
    input: &LiteralVector,
    limit: &mut usize,
    output: &mut LiteralVector)
  {
    if s.get_config().num_threads == 1 || s.parallel_syncing_clauses {
      return;
    }

    let old_par_syncing_clauses_value = s.parallel_syncing_clauses;
    s.parallel_syncing_clauses = true;
    { // Scope of `lock_guard` for `self.mux`
      let _lock_guard = self.mux.lock().unwrap();

      if *limit < self.units.len() {
        // this might repeat some literals.
        // output.append(self.units.len() - limit, self.units.data() + limit);
        output.append(self.units[limit..]);
      }
      for lit in input() {
        if !self.unit_set.contains(&lit.index()) {
          self.unit_set.insert(lit.index());
          self.units.push_back(lit);
        }
      }
      *limit = self.units.len();
      // Restore previous sync clause value
      s.parallel_syncing_clauses = old_par_syncing_clauses_value;
    }
  }

  /// Add the clause to the shared clause pool.
  pub fn share_clause(&mut self, solver: &mut Solver, c: &Clause){
    if solver.get_config().num_threads == 1 || !self.enable_add(c) || solver.parallel_syncing_clauses {
      return;
    }

    let old_par_syncing_clauses = solver.parallel_syncing_clauses;
    solver.parallel_syncing_clauses = true;

    let n = c.size();
    let owner = solver.parallel_id;
    log_at_level(3, format!("{}: share {}\n", owner, c).as_str());
    let _lock = self.mux.lock();

    self.pool.begin_add_vector(owner.into(), n.into());
    for i in 0..n {
      self.add_vector_elem(c[i].index());
    }
    self.pool.end_add_vector();

    solver.parallel_syncing_clauses = old_par_syncing_clauses;
  }

  /// Add the two-literal clause to the shared clause pool.
  pub fn share_literals(&mut self, solver: &mut Solver, l1: Literal, l2: Literal){
    if solver.get_config().num_threads == 1 || solver.parallel_syncing_clauses{
      return;
    }

    let old_par_syncing_clauses = solver.parallel_syncing_clauses;
    solver.parallel_syncing_clauses = true;

    log_at_level(
      3,
      format!("{}: share {} {}\n", solver.parallel_id, l1, l2).as_str()
    );

    {
      let _lock = self.mux.lock();
      self.pool.begin_add_vector(solver.parallel_id.into(), 2);
      self.pool.add_vector_elem(l1.index());
      self.pool.add_vector_elem(l2.index());
      self.pool.end_add_vector();
    }

    solver.parallel_syncing_clauses = old_par_syncing_clauses;
  }

  /// Receive clauses from shared clause pool
  pub fn get_clauses(&mut self, s: &mut Solver) {
    if (s.m_par_syncing_clauses) { return; }

    let old_par_syncing_clauses = s.parallel_syncing_clauses;
    s.parallel_syncing_clauses  = true;

    let _lock = self.mux.lock();

    let mut n  =  0u32;;
    unsigned const* ptr;
    unsigned owner = s.m_par_id;
    loop {

      let (n: u32, ptr: *usize) = // the result of the match
        match self.pool.get_vector_for_owner(owner, n, ptr) {

          Err(_) => break,

          Ok(value) => value

        }

      m_lits.reset();
      bool usable_clause = true;
      for (unsigned i = 0; usable_clause && i < n; ++i) {
        literal lit(to_literal(ptr[i]));
        m_lits.push_back(lit);
        usable_clause = lit.var() <= s.m_par_num_vars && !s.was_eliminated(lit.var());
      }
      IF_VERBOSE(3, verbose_stream() << s.m_par_id << ": retrieve " << m_lits << "\n";);
      SASSERT(n >= 2);
      if (usable_clause) {
        s.mk_clause_core(m_lits.size(), m_lits.data(), sat::status::redundant());
      }
    }



    s.parallel_syncing_clauses = old_par_syncing_clauses;
  }

  /// Exchange from solver state to local search and back.
  pub fn from_solver(&self, s: &Solver) {}

  pub fn to_solver(&self, s: &Solver) -> bool {
    // if (self.priorities.empty()) {
    //   return false;
    // }
    // for (bool_var v = 0; v < m_priorities.size(); ++v) {
    //   s.update_activity(v, m_priorities[v]);
    // }
    // return true;
  }

  pub fn from_local_search(&self, s: &i_local_search) -> bool {}
  pub fn to_local_search(&self, s: &i_local_search) {}

  pub fn copy_solver(&self, s: &Solver) -> bool {}

}



#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
