/*!
  
  Parameters governing how the solver functions.
  
*/

use std::rc::Rc;

use crate::symbol_table::Symbol;
use crate::missing_types::{Parameters, ParameterDescriptions};
use crate::local_search::LocalSearchMode;

// region Enums used in `Config`

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum PhaseSelection {
  AlwaysTrue,
  AlwaysFalse,
  BasicCaching,
  SATCaching,
  Frozen,
  Random
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum RestartStrategy {
  Geometric,
  Luby,
  Ema,
  Static
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GcStrategy {
  DynPsm,
  Psm,
  Glue,
  GluePsm,
  PsmGlue
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum BranchingHeuristic {
  Vsids,
  Chb
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum PbResolve {
  Cardinality,
  Rounding
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum PbLemmaFormat {
  Cardinality,
  Pb
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum RewardType {
  Ternary,
  UnitLiteral,
  HeuleSchur,
  HeuleUnit,
  MarchCu
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum CutoffType {
  Depth,
  Freevars,
  PSAT,
  AdaptiveFreevars,
  AdaptivePSAT
}

// endregion

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Config<'s> {
  max_memory            : u64,
  phase                 : PhaseSelection,
  search_sat_conflicts  : u32,
  search_unsat_conflicts: u32,
  pub(in local_search) phase_sticky: bool,
  rephase_base          : u32,
  reorder_base          : u32,
  reorder_itau          : f64,
  reorder_activity_scale: u32,
  propagate_prefetch    : bool,
  restart               : RestartStrategy,
  restart_fast          : bool,
  restart_initial       : u32,
  restart_factor        : f64,             // for geometric case
  restart_margin        : f64,             // for EMA
  restart_max           : u32,
  activity_scale        : u32,
  fast_glue_avg         : f64,
  slow_glue_avg         : f64,
  inprocess_max         : u32,
  inprocess_out         : Symbol<'s>,
  random_freq           : f64,
  pub(in local_search) random_seed: u32,
  burst_search        : u32,
  enable_pre_simplify : bool,
  max_conflicts       : u32,
  num_threads         : u32,
  ddfw_search         : bool,
  ddfw_threads        : u32,
  prob_search         : bool,
  local_search_threads: u32,
  local_search        : bool,
  pub(in local_search) local_search_mode     : LocalSearchMode,
  pub(in local_search) local_search_dbg_flips: bool,
  binspr          : bool,
  cut_simplify    : bool,
  cut_delay       : u32,
  cut_aig         : bool,
  cut_lut         : bool,
  cut_xor         : bool,
  cut_npn3        : bool,
  cut_dont_cares  : bool,
  cut_redundancies: bool,
  cut_force       : bool,
  anf_simplify    : bool,
  anf_delay       : u32,
  anf_exlin       : bool,

  lookahead_simplify             : bool,
  lookahead_simplify_bca         : bool,
  lookahead_cube_cutoff          : CutoffType,
  lookahead_cube_fraction        : f64,
  lookahead_cube_depth           : u32,
  lookahead_cube_freevars        : f64,
  lookahead_cube_psat_var_exp    : f64,
  lookahead_cube_psat_clause_base: f64,
  lookahead_cube_psat_trigger    : f64,
  lookahead_reward               : RewardType,
  lookahead_f64                  : bool,
  lookahead_global_autarky       : bool,
  lookahead_delta_fraction       : f64,
  lookahead_use_learned          : bool,

  incremental   : bool,
  next_simplify1: u32,
  simplify_mult2: f64,
  simplify_max  : u32,
  simplify_delay: u32,

  variable_decay: u32,

  gc_strategy   : GcStrategy,
  gc_initial    : u32,
  gc_increment  : u32,
  gc_small_lbd  : u32,
  gc_k          : u32,
  gc_burst      : bool,
  gc_defrag     : bool,

  force_cleanup : bool,

  // backtracking
  backtrack_scopes        : u32,
  backtrack_init_conflicts: u32,

  minimize_lemmas         : bool,
  dyn_sub_res             : bool,
  core_minimize           : bool,
  core_minimize_partial   : bool,

  // DRAT proofs
  drat            : bool,
  drat_binary     : bool,
  drat_file       : Symbol<'s>,
  drat_check_unsat: bool,
  drat_check_sat  : bool,
  drat_activity   : bool,

  card_solver     : bool,
  xor_solver      : bool,
  pb_resolve      : PbResolve,     // Pseudo-boolean Resolve
  pb_lemma_format : PbLemmaFormat, // Pseudo-boolean Resolve

  // branching heuristic settings
  branching_heuristic: BranchingHeuristic,
  anti_exploration   : bool,
  step_size_init     : f64,
  step_size_dec      : f64,
  step_size_min      : f64,
  reward_multiplier  : f64,
  reward_offset      : f64,

  // simplifier configurations used outside of `SatSimplifier`
  elim_vars: bool,

}

impl Config{

  pub fn new(parameters: Rc<Parameters>){
    unimplemented!();
  }

  pub fn update_parameters(parameters: Rc<Parameters>){
    unimplemented!();
  }

  pub fn collect_parameter_descriptions(descriptions: &mut ParameterDescriptions){
    unimplemented!();
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
