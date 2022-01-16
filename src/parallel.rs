/*!

  A "vector" is a `Vec` containing the indices of a clause, `clause.map(|c| c.index()).collect()`. Vectors are often accompanied by an owner, which is the `parallel_id` of the solver it belongs to.

 */


use std::{
  collections::HashSet,
  sync::Mutex, rc::Rc
};

use crate::{
  parameters::ParameterValue,
  Literal,
  LiteralVector,
  Solver,
  clause::Clause,
  log_assert,
  log::log_at_level,
  resource_limit::ArcRwResourceLimit, status::Status
};

type VectorIndexSet = HashSet<usize>;
type VectorIndex    = usize;

// todo: figure out what derives VectorPool needs.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
struct VectorPool {
  /// The inner `Vec<VectorIndex>` represents the clause.
  vectors: Vec<Vec<VectorIndex>>,
  owners : Vec<VectorIndex>
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
    self.vectors.push(vector);
    self.owners.push(owner);
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
  units    : LiteralVector,
  unit_set : VectorIndexSet,
  literals : LiteralVector,
  pool_lock: Mutex<VectorPool>, // TODO: Should this be an RwLock?

  // For exchange with local search:
  num_clauses   : usize,
  solver_copy   : Option<Box<Solver<'a>>>, // Scoped Pointer
  consumer_ready: bool,
  priorities    : Vec<f64>,

  resource_limit: ArcRwResourceLimit,       // Scoped Resource Limit
  limits : Vec<ArcRwResourceLimit>,
  solvers: Vec<Rc<Solver<'b>>> // Vector of solver pointers, might need to be Arcs
}

impl<'a, 'b> Parallel<'a, 'b> {

  // Todo: Make this take a resource limit, not a solver
  pub fn new(&self, solver: &Solver) -> Self {
    Parallel {
      units    : LiteralVector::new(),
      unit_set : VectorIndexSet::new(),
      literals : LiteralVector::new(),
      pool_lock: Mutex::new(VectorPool::default()),

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
    let saved_phase =
      solver.parameters
            .borrow()
            .get_value("phase")
            .unwrap_or(ParameterValue::Symbol("caching"));

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
    // todo: This reference to self is going to need to be adjusted to prevent aliasing.
    solver.set_parallel(self, num_extra_solvers);
    solver.parameters["phase"] = saved_phase;
  }

  pub fn push_child(&mut self, rl: ArcRwResourceLimit){ self.resource_limit.push_child(rl); }

  pub fn reserve(&mut self, num_owners: usize) {
    let mut pool = self.pool_lock.lock().unwrap();
    pool.reserve(num_owners);
  }

  pub fn get_solver(&self, i: usize) -> Rc<Solver> { return self.solvers[i].clone(); }

  pub fn cancel_solver(&self, i: usize) { self.limits[i].cancel(); }

  /// Exchange unit literals. This is only used in `Solver::pop_reinit()`.
  // TODO: What does this do? Get rid of the output variables. It also acquires a lock on a `self`-level mutex, but the
  //       code below is using the pool lock, which isn't right.
  pub fn exchange(
    &mut self,
    solver: &mut Solver,
    input : &LiteralVector,
    limit : &mut usize,
    output: &mut LiteralVector)
  {
    if solver.get_config().num_threads == 1 || solver.parallel_syncing_clauses {
      return;
    }

    let old_par_syncing_clauses_value = solver.parallel_syncing_clauses;
    solver.parallel_syncing_clauses = true;
    { // Scope of `pool`
      let pool = self.pool_lock.lock().unwrap();

      if *limit < self.units.len() {
        // this might repeat some literals.
        // output.append(self.units.len() - limit, self.units.data() + limit);
        output.append(self.units[limit..]);
      }
      for lit in input {
        if !self.unit_set.contains(&lit.index()) {
          self.unit_set.insert(lit.index());
          self.units.push_back(lit);
        }
      }
      *limit = self.units.len();
      // Restore previous sync clause value
      solver.parallel_syncing_clauses = old_par_syncing_clauses_value;
    }
  }

  /// Add the clause to the shared clause pool.
  pub fn share_clause(&mut self, solver: &mut Solver, clause: &Clause){
    if solver.get_config().num_threads == 1 || !self.enable_add(clause) || solver.parallel_syncing_clauses {
      return;
    }

    let old_par_syncing_clauses = solver.parallel_syncing_clauses;
    solver.parallel_syncing_clauses = true;

    let n = clause.size();
    let owner = solver.parallel_id;
    log_at_level(3, format!("{}: share {}\n", owner, clause).as_str());
    let mut pool = self.pool_lock.lock().unwrap();

    pool.add_vector(owner, &clause.iter().map(|v| v.index()).collect());

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
  pub fn get_clauses(&mut self, solver: &mut Solver) {
    if solver.parallel_syncing_clauses {
      return;
    }

    // todo: Why save the previous state? We only reach here if it's false.
    let old_par_syncing_clauses = solver.parallel_syncing_clauses;
    solver.parallel_syncing_clauses  = true;

    // Blocks until lock is available.
    let pool = self.pool_lock.lock().unwrap();
    let mut n  =  0u32;
    // unsigned const* ptr;
    let owner = solver.parallel_id;
    loop {

      let vector = // the result of the match
        match pool.get_vector_for_owner(owner) {

          Some(value) => value,

          None => break,

        };

      self.literals.clear();
      let usable_clause = true;
      for i in 0..vector.len() {
        let literal = vector[i];
        self.literals.push(literal);
        usable_clause = (literal.var() <= solver.parallel_variable_count) && !solver.eliminated[literal.var()];
        if !usable_clause {
          break;
        }
      }
      log_at_level(3, format!("{}: retrieve {}", solver.parallel_id, self.literals));
      log_assert!(n >= 2);
      if usable_clause {
        solver.mk_clause_core(&self.literals, Status::redundant());
      }
    }



    solver.parallel_syncing_clauses = old_par_syncing_clauses;
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
