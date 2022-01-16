/*!
Defines the `SolverCore` trait and its canonical implementation `Solver`.
*/

use std::{
  collections::{
    HashSet,
    HashMap,
  },
  rc::Rc,
};

use crate::{
  BoolVariableVector,
  clause::{
    ClauseWrapperVector,
    ClauseVector, Clause,
  },
  config::Config,
  data_structures::{
    ExponentialMovingAverage,
    RandomGenerator,
    Statistic,
    Statistics,
  },
  data_structures::{
    ApproximateSet,
    OredIntegerSet,
  },
  lifted_bool::LiftedBoolVector,
  literal::{
    Literal,
    LiteralSet,
    LiteralVector,
  },
  local_search::LocalSearchCore,
  missing_types::{
    AsymmBranch,
    BinarySPR,
    ClauseAllocator,
    Cleaner,
    Cuber,
    CutSimplifier,
    DRAT,
    Extension,
    Justification,
    ModelConverter,
    MUS,
    Parallel,
    ParamsRef,
    Probing,
    SCC,
    ScopedLimitTrail,
    SearchState,
    Simplifier,
    Stopwatch,
    VariableQueue,
  },
  model::Model,
  parameters::ParametersRef,
  ResourceLimit,
  status::Status,
  watched::WatchList, LiftedBool, log::trace,
};
use crate::missing_types::MinimalUnsatisfiableSet;
use crate::resource_limit::ArcRwResourceLimit;

const ENABLE_TERNARY: bool = true;

type LevelApproximateSet = OredIntegerSet<u32, u32>;
type IndexSet = HashSet<u32>;

struct BinaryClause(Literal, Literal);

pub trait SolverCore {
  fn new(resource_limit: ArcRwResourceLimit) -> Self;
  fn add_clause(n: u32, literals: LiteralVector, status: Status);
  fn check(literals: Vec<u32>);
  fn at_base_level(&self)       -> bool;
  fn get_core(&self)            -> &LiteralVector;
  fn get_model(&self)           -> &Model;
  fn get_reason_unknown(&self)  -> &char;
  fn is_inconsistent(&self)     -> bool;
  fn number_of_clauses(&self)   -> u32;
  fn number_of_variables(&self) -> u32;
  fn pop_to_base_level(&mut self);
}

/// Statistics collected about the (concrete) SAT solver.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct SolverStatistics {
  pub mk_var                : u32,
  pub mk_bin_clause         : u32,
  pub mk_ter_clause         : u32,
  pub mk_clause             : u32,
  pub conflict              : u32,
  pub propagate             : u32,
  pub bin_propagate         : u32,
  pub ter_propagate         : u32,
  pub decision              : u32,
  pub restart               : u32,
  pub gc_clause             : u32,
  pub del_clause            : u32,
  pub minimized_lits        : u32,
  pub dyn_sub_res           : u32,
  pub non_learned_generation: u32,
  pub blocked_corr_sets     : u32,
  pub elim_var_res          : u32,
  pub elim_var_bdd          : u32,
  pub units                 : u32,
  pub backtracks            : u32,
  pub backjumps             : u32,
}

impl SolverStatistics {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn collect_statistics(&self, statistics: &mut Statistics) {
    statistics["sat mk clause 2ary"]          = Statistic::from(self.mk_bin_clause);
    statistics["sat mk clause 3ary"]          = Statistic::from(self.mk_ter_clause);
    statistics["sat mk clause nary"]          = Statistic::from(self.mk_clause);
    statistics["sat mk var"]                  = Statistic::from(self.mk_var);
    statistics["sat gc clause"]               = Statistic::from(self.gc_clause);
    statistics["sat del clause"]              = Statistic::from(self.del_clause);
    statistics["sat conflicts"]               = Statistic::from(self.conflict);
    statistics["sat decisions"]               = Statistic::from(self.decision);
    statistics["sat propagations 2ary"]       = Statistic::from(self.bin_propagate);
    statistics["sat propagations 3ary"]       = Statistic::from(self.ter_propagate);
    statistics["sat propagations nary"]       = Statistic::from(self.propagate);
    statistics["sat restarts"]                = Statistic::from(self.restart);
    statistics["sat minimized lits"]          = Statistic::from(self.minimized_lits);
    statistics["sat subs resolution dyn"]     = Statistic::from(self.dyn_sub_res);
    statistics["sat blocked correction sets"] = Statistic::from(self.blocked_corr_sets);
    statistics["sat units"]                   = Statistic::from(self.units);
    statistics["sat elim bool vars res"]      = Statistic::from(self.elim_var_res);
    statistics["sat elim bool vars bdd"]      = Statistic::from(self.elim_var_bdd);
    statistics["sat backjumps"]               = Statistic::from(self.backjumps);
    statistics["sat backtracks"]              = Statistic::from(self.backtracks);
  }


}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Hash)]
struct Scope {
  pub trail_lim            : u32,
  pub clauses_to_reinit_lim: u32,
  pub inconsistent         : bool
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Solver<'s> {

  // Data members that should be in SolverCore.
  // todo: Consider putting getters & setters in SolverCore. Problem is, that would make it
  //       public. At least here we can specify finer-grained access.
  pub resource_limit: ArcRwResourceLimit,

  // todo: What should be `RC`'s in this struct? Should the `Rc`s be `Arc`s? `COW`s?
  checkpoint_enabled: bool,
  config            : Config<'s>,
  statistics        : SolverStatistics,
  pub ext           : Option<Box<Extension>>,
  cut_simplifier    : Option<Box<CutSimplifier>>,
  parallel          : Option<Box<Parallel>>,
  pub drat          : DRAT, // DRAT for generating proofs
  cls_allocator     : ClauseAllocator,
  cls_allocator_idx : bool,
  rand              : RandomGenerator,
  cleaner           : Cleaner,
  model             : Model,
  mc                : ModelConverter,
  model_is_current  : bool,
  simplifier        : Simplifier,
  scc               : SCC,
  asymm_branch      : AsymmBranch,
  probing           : Probing,
  is_probing        : bool,              // defaults to false
  mus               : MinimalUnsatisfiableSet,               // MUS for minimal core extraction
  binspr            : BinarySPR,
  inconsistent      : bool,
  searching         : bool,

  // A conflict is usually a single justification. That is, a justification for false. If `not_l` is not
  // `Literal::NULL`, then `conflict` is a justification for `l`, and the conflict is union of `no_l` and `conflict`.
  conflict        : Justification,
  not_l           : Literal,
  pub clauses     : ClauseVector,
  learned         : ClauseVector,
  num_frozen      : u32,
  active_vars     : Vec<u32>,
  free_vars       : Vec<u32>,
  vars_to_reinit  : Vec<u32>,
  pub watches     : Vec<WatchList>,
  assignment      : LiftedBoolVector,
  justification   : Vec<Justification>,
  decision        : Vec<bool>,
  mark            : Vec<bool>,
  lit_mark        : Vec<bool>,
  pub eliminated  : Vec<bool>,
  external        : Vec<bool>,
  var_scope       : Vec<u32>,
  touched         : Vec<u32>,
  touch_index     : u32,
  replay_assign   : LiteralVector,

  // branch variable selection:
  activity        : Vec<u32>,
  activity_inc    : u32,
  last_conflict   : Vec<u64>,
  last_propagation: Vec<u64>,
  participated    : Vec<u64>,
  canceled        : Vec<u64>,
  reasoned        : Vec<u64>,
  action          : i32,
  step_size       : f64,

  // phase
  pub phase             : Vec<bool>,
  pub best_phase        : Vec<bool>,
  pub best_phase_size   : u32,
  prev_phase            : Vec<bool>,
  assigned_since_gc     : Vec<char>,
  search_state          : SearchState,
  search_unsat_conflicts: u32,
  search_sat_conflicts  : u32,
  search_next_toggle    : u32,
  phase_counter         : u32,
  rephase_lim           : u32,
  rephase_inc           : u32,
  reorder_lim           : u32,
  reorder_inc           : u32,
  case_split_queue      : VariableQueue,
  qhead                 : u32,
  scope_level           : u32,
  search_level          : u32,
  fast_glue_avg         : ExponentialMovingAverage,
  slow_glue_avg         : ExponentialMovingAverage,
  fast_glue_backup      : ExponentialMovingAverage,
  slow_glue_backup      : ExponentialMovingAverage,
  trail_avg             : ExponentialMovingAverage,
  pub trail             : LiteralVector,
  clauses_to_reinit     : ClauseWrapperVector,
  reason_unknown        : String,
  visited               : Vec<u32>,
  visited_ts            : u32,

  scopes            : Vec<Scope>,
  vars_lim          : ScopedLimitTrail,
  stopwatch         : Stopwatch,
  pub(crate) parameters : ParametersRef<'s>,
  clone             : Rc<Solver<'s>>,     // for debugging purposes
  assumptions       : LiteralVector,      // additional assumptions during check
  assumption_set    : LiteralSet,         // set of enabled assumptions
  ext_assumption_set: LiteralSet,         // set of enabled assumptions
  core              : LiteralVector,      // unsat core

  pub(crate) parallel_id      : u32,
      parallel_limit_in       : u32,
      parallel_limit_out      : u32,
  pub parallel_variable_count : u32,
  pub parallel_syncing_clauses: bool,

  cuber         : Box<Cuber>,
  local_search  : Option<Box<dyn LocalSearchCore>>,
  aux_statistics: Statistics,

  // -----------------------
  //
  // Search
  //
  // -----------------------

  m_conflicts_since_init    : u32,  // { 0 };
  m_restarts                : u32,  // { 0 };
  m_restart_next_out        : u32,  // { 0 };
  m_conflicts_since_restart : u32,  // { 0 };
  m_force_conflict_analysis : bool, // { false };
  m_simplifications         : u32,  // { 0 };
  m_restart_threshold       : u32,  // { 0 };
  m_luby_idx                : u32,  // { 0 };
  m_conflicts_since_gc      : u32,  // { 0 };
  m_gc_threshold            : u32,  // { 0 };
  m_defrag_threshold        : u32,  // { 0 };
  m_num_checkpoints         : u32,  // { 0 };
  m_min_d_tk                : f64,  // { 0 } ;
  m_next_simplify           : u32,  // { 0 };
  m_simplify_enabled        : bool, // { true };
  m_restart_enabled         : bool, // { true };

  m_min_core          : LiteralVector,
  m_min_core_valid    : bool,          // { false };

  m_last_positions    : Vec<usize>,
  m_last_position_log : u32,
  m_restart_logs      : u32,


  // PROTECTED
  // -----------------------
  //
  // Conflict resolution
  //
  // -----------------------
  m_conflict_lvl    : u32,
  m_lemma           : LiteralVector,
  m_ext_antecedents : LiteralVector,


  m_diff_levels     : Vec<char>,

  // lemma minimization
  m_unmark          : BoolVariableVector,
  m_lvl_set         : LevelApproximateSet,
  m_lemma_min_stack : LiteralVector,


  // -----------------------
  //
  // Backtracking
  //
  // -----------------------

  m_user_scope_literals : LiteralVector,
  m_free_var_freeze     : Vec<BoolVariableVector>,
  m_aux_literals        : LiteralVector,
  m_user_bin_clauses    : Vec<BinaryClause>,


  // Auxiliary
  m_antecedents         : HashMap<u32, IndexSet>,
  m_todo_antecedents    : LiteralVector,
  m_binary_clause_graph : Vec<LiteralVector>,


}

/*
impl Default<'s> for Solver<'s> {
  fn default() -> Self {
    Self{
      // Data members that should be in SolverCore.
      pub resource_limit : ResourceLimit::new(),

      checkpoint_enabled: false,
      config            : Config::d>,
      statistics        : SolverStatistics,
      pub ext           : Option<Box<Extension>>,
      cut_simplifier    : Option<Box<CutSimplifier>>,
      par               : Parallel,
      pub drat          : DRAT, // DRAT for generating proofs
      cls_allocator     : ClauseAllocator,
      cls_allocator_idx : bool,
      rand              : RandomGenerator,
      cleaner           : Cleaner,
      model             : Model,
      mc                : ModelConverter,
      model_is_current  : bool,
      simplifier        : Simplifier,
      scc               : SCC,
      asymm_branch      : AsymmBranch,
      probing           : Probing,
      is_probing        : bool,              // defaults to false
      mus               : MinimalUnsatisfiableSet,               // MUS for minimal core extraction
      binspr            : BinarySPR,
      inconsistent      : bool,
      searching         : bool,

      // A conflict is usually a single justification. That is, a justification for false. If `not_l` is not
      // `Literal::NULL`, then `conflict` is a justification for `l`, and the conflict is union of `no_l` and `conflict`.
      conflict        : Justification,
      not_l           : Literal,
      pub clauses     : ClauseVector,
      learned         : ClauseVector,
      num_frozen      : u32,
      active_vars     : Vec<u32>,
      free_vars       : Vec<u32>,
      vars_to_reinit  : Vec<u32>,
      pub watches     : Vec<WatchList>,
      assignment      : LiftedBoolVector,
      justification   : Vec<Justification>,
      decision        : Vec<bool>,
      mark            : Vec<bool>,
      lit_mark        : Vec<bool>,
      eliminated      : Vec<bool>,
      external        : Vec<bool>,
      var_scope       : Vec<u32>,
      touched         : Vec<u32>,
      touch_index     : u32,
      replay_assign   : LiteralVector,

      // branch variable selection:
      activity        : Vec<u32>,
      activity_inc    : u32,
      last_conflict   : Vec<u64>,
      last_propagation: Vec<u64>,
      participated    : Vec<u64>,
      canceled        : Vec<u64>,
      reasoned        : Vec<u64>,
      action          : i32,
      step_size       : f64,

      // phase
      pub phase             : Vec<bool>,
      pub best_phase        : Vec<bool>,
      pub best_phase_size   : u32,
      prev_phase            : Vec<bool>,
      assigned_since_gc     : Vec<char>,
      search_state          : SearchState,
      search_unsat_conflicts: u32,
      search_sat_conflicts  : u32,
      search_next_toggle    : u32,
      phase_counter         : u32,
      rephase_lim           : u32,
      rephase_inc           : u32,
      reorder_lim           : u32,
      reorder_inc           : u32,
      case_split_queue      : VariableQueue,
      qhead                 : u32,
      scope_lvl             : u32,
      search_lvl            : u32,
      fast_glue_avg         : ExponentialMovingAverage,
      slow_glue_avg         : ExponentialMovingAverage,
      fast_glue_backup      : ExponentialMovingAverage,
      slow_glue_backup      : ExponentialMovingAverage,
      trail_avg             : ExponentialMovingAverage,
      pub trail             : LiteralVector,
      clauses_to_reinit     : ClauseWrapperVector,
      reason_unknown        : String,
      visited               : Vec<u32>,
      visited_ts            : u32,

      scopes            : Vec<Scope>,
      vars_lim          : ScopedLimitTrail,
      stopwatch         : Stopwatch,
      pub(crate) params : ParametersRef<'s>,
      clone             : Rc<Solver<'s>>,     // for debugging purposes
      assumptions       : LiteralVector,      // additional assumptions during check
      assumption_set    : LiteralSet,         // set of enabled assumptions
      ext_assumption_set: LiteralSet,         // set of enabled assumptions
      core              : LiteralVector,      // unsat core

      par_id             : u32,
      par_limit_in       : u32,
      par_limit_out      : u32,
      par_num_vars       : u32,
      par_syncing_clauses: bool,

      cuber         : Box<Cuber>,
      local_search  : Option<Box<dyn LocalSearchCore>>,
      aux_statistics: Statistics,

      // -----------------------
      //
      // Search
      //
      // -----------------------

      m_conflicts_since_init    : u32,  // { 0 };
      m_restarts                : u32,  // { 0 };
      m_restart_next_out        : u32,  // { 0 };
      m_conflicts_since_restart : u32,  // { 0 };
      m_force_conflict_analysis : bool, // { false };
      m_simplifications         : u32,  // { 0 };
      m_restart_threshold       : u32,  // { 0 };
      m_luby_idx                : u32,  // { 0 };
      m_conflicts_since_gc      : u32,  // { 0 };
      m_gc_threshold            : u32,  // { 0 };
      m_defrag_threshold        : u32,  // { 0 };
      m_num_checkpoints         : u32,  // { 0 };
      m_min_d_tk                : f64,  // { 0 } ;
      m_next_simplify           : u32,  // { 0 };
      m_simplify_enabled        : bool, // { true };
      m_restart_enabled         : bool, // { true };

      m_min_core          : LiteralVector,
      m_min_core_valid    : bool,          // { false };

      m_last_positions    : Vec<usize>,
      m_last_position_log : u32,
      m_restart_logs      : u32,


      // PROTECTED
      // -----------------------
      //
      // Conflict resolution
      //
      // -----------------------
      m_conflict_lvl    : u32,
      m_lemma           : LiteralVector,
      m_ext_antecedents : LiteralVector,


      m_diff_levels     : Vec<char>,

      // lemma minimization
      m_unmark          : BoolVariableVector,
      m_lvl_set         : LevelApproximateSet,
      m_lemma_min_stack : LiteralVector,


      // -----------------------
      //
      // Backtracking
      //
      // -----------------------

      m_user_scope_literals : LiteralVector,
      m_free_var_freeze     : Vec<BoolVariableVector>,
      m_aux_literals        : LiteralVector,
      m_user_bin_clauses    : Vec<BinaryClause>,


      // Auxiliary
      m_antecedents         : HashMap<u32, IndexSet>,
      m_todo_antecedents    : LiteralVector,
      m_binary_clause_graph : Vec<LiteralVector>,

    }
  }
}
*/

impl<'s> Solver<'s> {



  pub fn from_params_limit(params: ParametersRef, resource_limit: ArcRwResourceLimit) -> Self{
    Self{
      parameters: params,
      resource_limit: resource_limit.clone(),
      ..Self::default
    }
  }


  pub fn get_config(&self) -> &Config {
    &self.config
  }

  pub fn resource_limit(&self) -> ArcRwResourceLimit {
    self.resource_limit.clone()
  }

  pub fn collect_statistics(&self, st: &mut Statistics){
    self.statistics.collect_statistics(st);
    self.cleaner.collect_statistics(st);
    self.simplifier.collect_statistics(st);
    self.scc.collect_statistics(st);
    self.asymm_branch.collect_statistics(st);
    self.probing.collect_statistics(st);
    if let Some(ext) = &self.ext {
      ext.collect_statistics(st);
    }
    if let Some(local_search) = &self.local_search{
      local_search.collect_statistics(st);
    }
    if let Some(cut_simplifier) = &self.cut_simplifier{
      cut_simplifier.collect_statistics(st);
    }
    st.extend(&self.aux_statistics);
  }

  fn set_parallel(&mut self, parallel: &Parallel, parallel_id: usize) {
      self.parallel                 = parallel;
      self.parallel_variable_count  = self.number_of_variables();
      self.parallel_limit_in        = 0;
      self.parallel_limit_out       = 0;
      self.parallel_id              = parallel_id;
      self.parallel_syncing_clauses = false;
  }

  pub fn mk_clause_core(&mut self, literals: &LiteralVector, status: Status) -> Option<Box<Clause>> {
    let redundant = status.is_redundant();
    let literal_count = literals.len();

    trace!(
      "sat",
      format!(
        "mk_clause: {} {}\n",
        display_literal_vector(literals),
        if redundant {
          "learned"
        } else {
          "aux"
        }
      )
    );

    if !redundant || !status.is_satisfied() {
      let old_sz        = literals.len();
      let keep          = self.simplify_clause(literals);

      trace!(
        "sat_mk_clause",
        format!(
          "mk_clause (after simp), keep: {}\n{}\n",
          keep,
          display_literal_vector(literals)
        )
      );

      if !keep {
        return None; // Clause is equivalent to true.
      }

      // If an input clause is simplified, then log the simplified version as learned
      if self.config.drat && old_sz > literal_count {
        self.drat.add(literals, status);
        // drat_log_clause(literals, status);
      }

      self.statistics.non_learned_generation += 1;

      if !self.searching {
        self.mc.add_clause(literals);
      }
    }

    match literal_count {

      0 => {
        self.set_conflict();
        return None;
      }

      1 => {
        if self.config.drat && (!status.is_satisfied() || status.is_input()) {
          // drat_log_clause(literals, status);
          self.drat.add(literals, status);
        }
        self.assign_unit(literals[0]);

        return None;
      }

      2 => {
        self.mk_bin_clause(literals[0], literals[1], status);
        if redundant {
          if let Some(parallel) = self.parallel {
            parallel.share_literals(self, literals[0], literals[1]);
          }
        }
        return None;
      }

      3 => {
        if ENABLE_TERNARY {
          return self.mk_ter_clause(literals, status);
        }
        return self.mk_nary_clause(literals, status);
      }

      _ => {
        return self.mk_nary_clause(literals, status);
      }

    }
  }

  fn assign(&mut self, literal: Literal, justification: Justification) {

    trace!("sat_assign", "{} previous value: {} j: {}\n", literal,  self.value(l), justification);

    match self.value(literal) {
      LiftedBool::False     => self.set_conflict(justification, !literal),
      LiftedBool::Undefined => self.assign_core(literal,justification),
      LiftedBool::True      => self.update_assign(literal, justification)
    };
  }

  fn update_assign(&mut self, literal: Literal, justification: Justification) {
    if justification.level() == 0 {
      self.justification[literal.var()] = justification;
    }
  }

  fn assign_unit(&mut self, literal: Literal) {
    self.assign(literal, Justification::with_level(0))
  }



  /// Returns the `self.assignment` of the given `Literals`.
  fn get_literal_value(&self, literal: Literal) -> LiftedBool {
    self.assignment[literal.index()]
  }

  fn get_literal_level(&self, literal: Literal) -> u32 {
    self.justification[literal.var()].level()
  }

  // The template bool allows for compile-time optimization based on the value of `lvl0`.
  fn simplify_clause_core<const LEVEL_ZERO: bool>(&self, literals: &mut LiteralVector) -> bool {
    literals.sort_unstable();
    let previous_literal = Literal::NULL;
    let j = 0u32;

    for i in 0.. literals.len() {
      let current_literal = literals[i];
      let value: LiftedBool = self.get_literal_value(current_literal);

      if !LEVEL_ZERO && self.get_literal_level(current_literal) > 0 {
        value = LiftedBool::Undefined;
      }

      match value {

        LiftedBool::False => { /*  Ignore this literal */ },

        LiftedBool::Undefined => {
          if current_literal == !previous_literal {
              return false; // Clause is equivalent to true
            }
          if current_literal != previous_literal {
              previous_literal = current_literal;
              if i != j {
                  std::mem::swap(literals[j], literals[i]);
                }
              j += 1;
          }
        }

        LiftedBool::True => {
          return false; // Clause is equivalent to true
        }

      }
    }
    // num_lits = j;
    true
  }

  fn at_base_level(&self) -> bool {
    self.scope_level == 0
  }

  fn simplify_clause(&self, literals: &mut LiteralVector) -> bool {
        if self.at_base_level(){
          return self.simplify_clause_core::<true>(literals);
        }
        else {
          return self.simplify_clause_core::<false>(literals);
        }
    }

}
