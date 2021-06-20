/*!
  
  Says on the tin.
  
*/
use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;

use crate::{
  BoolVariable,
  BoolVariableVector,
  LiftedBool,
  Literal,
  LiteralVector,
  Model,
  ResourceLimit,
  Solver,
  verbosity
};
use crate::config::Config;
use crate::missing_types::{Parallel, Parameters, ParametersRef, RandomGenerator};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum LocalSearchMode {
  GSAT,
  WSAT
}


// region structs

// region LocalSearchConfig
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LocalSearchConfig {
  pub random_seed     : u32,
  pub best_known_value: i32,
  mode                : LocalSearchMode,
  phase_sticky        : bool,
  dbg_flips           : bool,
  itau                : f64,
}

impl LocalSearchConfig {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn mode(&self) -> LocalSearchMode {
    self.mode
  }
  pub fn phase_sticky(&self) -> bool {
    self.phase_sticky
  }
  pub fn dbg_flips(&self) -> bool {
    self.dbg_flips
  }
  pub fn itau(&self) -> f64 {
    self.itau
  }

  fn set_config(&mut self, cfg: &Config) {
    self.mode         = cfg.local_search_mode;
    self.random_seed  = cfg.random_seed;
    self.phase_sticky = cfg.phase_sticky;
    self.dbg_flips    = cfg.local_search_dbg_flips;
  }
}

impl Default for LocalSearchConfig {
  fn default() -> Self {
    LocalSearchConfig {
      random_seed     : 0u32,
      best_known_value: i32::MAX,
      mode            : LocalSearchMode::WSAT,
      phase_sticky    : false,
      dbg_flips       : false,
      itau            : 0.5f64,
    }
  }
}

// endregion

// region Module-private structs

/// Pseudo-boolean Coefficient
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
struct PbCoefficient {
  constraint_id: u32,
  coefficient  : u32,
}
type CoefficientVector = Vec<PbCoefficient>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
struct Statistics {
  count_of_flips   : u32,
  count_of_restarts: u32,
}
impl Statistics {
  pub fn reset(&mut self) {
    self.count_of_flips    = 0;
    self.count_of_restarts = 0;
  }
  pub fn new(&mut self) -> Self {
    Self::default()
  }
}


struct VariableInfo {
  bias            : u32,     // bias for current solution in percentage.
                             // if bias is 0, then value is always false, if 100, then always true
  bin             : LiteralVector,
  break_prob      : f64,
  conf_change     : bool,    // whether its configure changes since its last flip
  explain         : Literal, // explanation for unit assignment
  flips           : u32,
  in_goodvar_stack: bool,
  neighbors       : BoolVariableVector, // neighborhood variables
  score           : i32,
  slack_score     : i32,
  slow_break      : f64,
  time_stamp      : u32,     // the flip time stamp
  unit            : bool,    // is this a unit literal
  value           : bool,    // current solution
  watch           : (CoefficientVector, CoefficientVector), // (true_coeffs, false_coeffs)
}

impl Default for VariableInfo {
  fn default() -> Self {
    Self{
      value: true,  // current solution
      bias : 50u32, // bias for current solution in percentage. if bias is 0, then value is always
                    // false, if 100, then always true
      unit            : false,      // Is this a unit literal? (no)
      explain         : Literal(0), // explanation for unit assignment
      conf_change     : true,       // whether its configure changes since its last flip
      in_goodvar_stack: false,
      score           : 0,
      slack_score     : 0,
      time_stamp      : 0,
      neighbors       : vec![],     // neighborhood variables
      watch           : vec![],
      bin             : vec![],
      flips           : 0,
      slow_break      : 1e-5f64,
      break_prob      : 0f64,
    }
  }
}


#[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
struct Constraint {
  id: u32,
  k: u32,
  slack: i64,
  literals: LiteralVector,
}
impl Constraint{
  fn new(k: u32, id: u32) -> Self{
    Self{
      id,
      k,
      ..Self::default()
    }
  }
  fn push(&mut self, literal: Literal) {
    self.literals.push(literal)
  }
  fn size(&self) -> usize {
    self.literals.len()
  }
  fn operator(idx: u32) -> Literal { literals[idx] }

  fn iter(&self) -> std::slice::Iter<'_, Literal> {
    self.literals.iter()
  }
}


// endregion
// endregion


pub trait LocalSearchCore {
  fn add(solver: &Solver);
  fn update_parameters(p: ParametersRef);
  fn set_seed(s: u32);
  fn check(assumptions: LiteralVector, parallel: &Parallel) -> LiftedBool;
  fn reinit_with_solver(solver: &Solver);
  fn num_non_binary_clauses() -> u32;
  fn resource_limit() -> &ResourceLimit; // todo: probably use `Arc<ResourceLimit>`
  fn get_model() -> &Model;
  fn collect_statistics(statistics: &Statistics);
  fn get_priority(_bool_var: BoolVariable) -> f64  {
    return 0f64;
  }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct LocalSearch {

  stats : Statistics,
  config: LocalSearchConfig,

  vars                  : Vec<VariableInfo>,  // variables
  best_phase            : Vec<bool>,          // best value in round
  units                 : BoolVariableVector, // unit clauses
  constraints           : Vec<Constraint>,    // all constraints
  assumptions           : LiteralVector,      // temporary assumptions
  prop_queue            : LiteralVector,      // propagation queue
  num_non_binary_clauses: u32,
  is_pb                 : bool,
  is_unsat              : bool,
  unsat_stack           : Vec<u32>,           // store all the unsat constraints
  index_in_unsat_stack  : Vec<u32>,           // which position is a constraint in the unsat_stack

  // configuration changed decreasing variables (score>0 and conf_change==true)
  goodvar_stack: BoolVariableVector,
  initializing : bool,


  // information about solution
  best_unsat          : u32,
  best_unsat_rate     : f64,
  last_best_unsat_rate: f64,
  // for non-known instance, set as maximal
  best_known_value    : i32, // best known value for this instance

  max_steps: u32,

  // dynamic noise
  noise      : f64, // normalized by 10000
  noise_delta: f64,

  limit: ResourceLimit,
  rand : RandomGenerator,
  par  : Box<Parallel>,
  model: Model,
}

impl LocalSearch {
  pub fn new() -> Self{
    Self {
      best_known_value: i32::MAX,
      max_steps: (1 << 30),
      noise: 9800f64,
      noise_delta: 0.05,
      ..Self::default()
    }
  }

  // region private methods

  fn score(&self, v: BoolVariable) -> i32 { 
    return self.vars[v].score; 
  }
  fn inc_score(&self, v: BoolVariable) { 
    self.vars[v].score = self.vars[v].score + 1; 
  }
  fn dec_score(&self, v: BoolVariable) { 
    self.vars[v].score = self.vars[v].score - 1; 
  }

  fn slack_score(&self, v: BoolVariable) -> i32 { 
    return self.vars[v].slack_score; 
  }
  fn inc_slack_score(&self, v: BoolVariable) { 
    self.vars[v].slack_score = self.vars[v].slack_score + 1; 
  }
  fn dec_slack_score(&self, v: BoolVariable) { 
    self.vars[v].slack_score = self.vars[v].slack_score - 1; 
  }

  fn already_in_goodvar_stack(&self, v: BoolVariable) -> bool { 
    return self.vars[v].in_goodvar_stack; 
  }
  fn conf_change(&self, v: BoolVariable) -> bool { 
    return self.vars[v].conf_change; 
  }
  fn time_stamp(&self, v: BoolVariable) -> i32 { 
    return self.vars[v].time_stamp; 
  }

  fn set_best_unsat(&mut self) {
    self.best_unsat = self.unsat_stack.size();
    self.best_phase.reserve(self.vars.size());
    for i in 0..self.vars.size() {
      self.best_phase[i] = self.vars[i].value;
    }
  }


  fn is_pos(&self, t: Literal) -> bool  { 
    return !t.sign(); 
  }
  fn is_true(&self, v: BoolVariable) -> bool  { 
    return self.current_solution(v);
  }
  fn is_true_literal(&self, l: Literal) -> bool  { 
    return self.cur_solution(l.var()) != l.sign(); 
  }
  fn is_false(&self, l: Literal) -> bool  { 
    return self.cur_solution(l.var()) == l.sign(); 
  }
  fn is_unit(&self, v: BoolVariable) -> bool  { 
    return self.vars[v].unit; 
  }
  fn is_unit_literal(&self, l: Literal) -> bool  { 
    return self.vars[l.var()].unit; 
  }

  /// constraint index from 1 to num_constraint
  fn num_constraints(&self) -> usize  {
    return self.constraints.size();
  }
  fn constraint_slack(&self, ci: u32) -> u64  { 
    return self.constraints[ci].slack; 
  }

  fn init(&mut self) {
    let old_initializing_state = self.initializing;
    self.initializing = true;
    
    self.unsat_stack.clear();
    self.add_clauses(&self.assumptions);
    if self.is_unsat {
      return;
    }

    // add sentinel variable.
    self.vars.push_back(VariableInfo::default());

    let value_assigner = 
      if self.config.phase_sticky() {
        | vi: &mut VariableInfo | vi.value = vi.bias > 50
      }
      else {
        | vi: &mut VariableInfo | vi.value = (0 == (self.rand() % 2))
      };
    self.vars
        .iter_mut()
        .filter(|&vi| !vi.unit )
        .for_each(value_assigner);
    
    self.index_in_unsat_stack.resize(self.num_constraints(), 0);
    self.set_parameters();

    self.initializing = old_initializing_state;
  }
  
  fn reinit(&mut self) {

    //
    // the following method does NOT converge for pseudo-boolean
    // can try other way to define "worse" and "better"
    // the current best noise is below 1000
    //
    if self.best_unsat_rate > self.last_best_unsat_rate {
      // worse
      self.noise -= self.noise * 2 * self.noise_delta;
      self.best_unsat_rate *= 1000.0;
    }
    else {
      // better
      self.noise += (10000 - self.noise) * self.noise_delta;
    }

    self.constraints
        .iter_mut()
        .for_each(| c | c.slack = c.k as i64);


    // init unsat stack
    self.is_unsat = false;
    self.unsat_stack.clear();

    // init solution using the bias
    self.init_cur_solution();

    // init variable information
    // The last variable is the virtual sentinel variable.
    let sentinel = self.vars.last_mut().unwrap();
    sentinel.score = i32::MIN;
    sentinel.conf_change = false;
    sentinel.slack_score = i32::MIN;
    sentinel.time_stamp = self.max_steps + 1;
    for v in self.vars.iter_mut() {
      v.time_stamp = 0;
      v.conf_change = true;
      v.in_goodvar_stack = false;
      v.score = 0;
      v.slack_score = 0;
    }
    self.init_slack();
    self.init_scores();
    self.init_goodvars();
    self.set_best_unsat();

    for bv in self.units{
      if self.is_unsat {
        break;
      }
      self.propagate(literal(bv, !self.cur_solution(bv)));
    }

    if self.is_unsat {
      verbosity::emit_if_level(0, "unsat during reinit\n");
    }
    #[cfg(debug_assertions)]
    verify_slack();
  }

  fn init_cur_solution(&mut self) {
    for var_info in self.vars.iter_mut() {
      if !var_info.unit {
        if self.config.phase_sticky() {
          var_info.value = ((self.rand() % 100) as u32) < var_info.bias;
        }
        else {
          var_info.value = (self.rand() % 2) == 0;
        }
      }
    }
  }

  fn init_slack(&mut self) {
    for v in 0..self.num_vars() {
      let is_true = self.cur_solution(v as BoolVariable);
      let true_variable_coefficients =
        match is_true {
          false => &self.vars[v].watch.0,
          true  => &self.vars[v].watch.1,
        };
      for pb_coefficient in true_variable_coefficients {
        let constraint = self.constraints //[coeff.constraint_id];
                                 .get_mut(pb_coefficient.constraint_id)
                                 .unwrap();
        constraint.slack -= pb_coefficient.coefficient;
      }
    }
    for c in 0..self.num_constraints() {
      // Violate the at-most-k constraint
      if self.constraints[c].slack < 0 {
        self.unsat(c as u32);
      }
    }
  }

  fn init_scores(&self) {
    for v in 0..self.num_vars() {
      let is_true = self.cur_solution(v);
      let truep = self.vars[v].watch[is_true];
      let falsep = self.vars[v].watch[!is_true];
      for coeff in falsep {
        c = self.constraints[coeff.constraint_id];

        if c.m_slack <= 0 {
          self.dec_slack_score(v);
          if c.m_slack == 0 {
            self.dec_score(v);
          }
        }
      }
      for coeff in truep {
        let c = self.constraints[coeff.constraint_id];
        if c.slack <= -1 {
          self.inc_slack_score(v as BoolVariable);
          if c.slack == -1 {
            self.inc_score(v);
          }
        }
      }
    }
  }


  fn init_goodvars(&self) {}
  fn pick_flip_lookahead(&self) {}
  fn pick_flip_walksat(&self) {}
  fn flip_walksat(&self, v: BoolVariable) {}
  fn propagate(&self, lit: Literal) -> bool  {}
  fn add_propagation(&self, lit: Literal) {}
  fn walksat(&self) {}
  fn unsat(&self, c: u32) {}
  fn sat(&self, c: u32) {}
  fn set_parameters(&self) {}
  fn verify_solution(&self) {}
  fn verify_unsat_stack(&self) {}
  fn verify_constraint(&self, c: &Constraint) {}
  fn verify_slack_with_constraint(&self, c: &Constraint) {}
  fn verify_slack(&self) {}
  fn verify_goodvar(&self) -> bool {}
  fn constraint_value(&self, c: &Constraint) -> u64  {}
  fn constraint_coefficient_with_literal(&self, c: &Constraint, l: Literal) -> u32  {}
  fn print_info(&self, out: &std::ostream) {}
  fn extract_model(&self) {}
  fn add_clause(&self, c: Literal) {}
  fn add_clauses(&self, c: &LiteralVector) {}
  fn add_unit(&self, lit: Literal, explain: Literal) {}

  // fn display(&self, out: &std::ostream) -> &std::ostream  {}
  // fn display(&self, out: &std::ostream, c: &Constraint) -> &std::ostream  {}
  // fn display(&self, out: &std::ostream, v: u32, vi: &var_info) -> &std::ostream  {}

  fn check_self(&self) -> LiftedBool  {}

  fn num_vars(&self) -> usize  {
    // var index from 1 to num_vars
    return self.vars.size() - 1; 
  }     

  // endregion private methods
  
  // region public methods

  pub fn rlimit(&self) -> &ResourceLimit  { 
    return &self.limit; 
  }

  pub fn check(&self, sz: u32, assumptions: &Literal, p: &Parallel) -> LiftedBool  {}
  pub fn num_non_binary_clauses(&self) -> u32  { 
    return self.num_non_binary_clauses; 
  }

  pub fn add(&self, s: &Solver) { 
    self.import(s, false); 
  }

  pub fn get_model(&self) -> &Model  { 
    return &self.model; 
  }

  pub fn collect_statistics(&self, st: &Statistics) {}
  pub fn updt_params(&self, p: ParametersRef) {}

  pub fn set_seed(&mut self, n: u32) {
    self.config.set_random_seed(n);
  }

  pub fn reinit_with_solver(&self, s: &Solver) {}
  // used by unit-walk
  pub fn set_phase(&self, v: BoolVariable, f: bool) {}
  pub fn set_bias(&self, v: BoolVariable, f: LiftedBool) {}
  pub fn get_best_phase(&self, v: BoolVariable) -> bool  { 
    return self.best_phase[v]; 
  }

  pub fn cur_solution(&self, v: BoolVariable) -> bool  { 
    return self.vars[v].value; 
  }

  pub fn get_priority(&self, v: BoolVariable) -> f64  { 
    return self.vars[v].break_prob; 
  }

  pub fn import(&self, s: &Solver, init: bool) {}
  pub fn add_cardinality(&self, sz: u32, c: &Literal, k: u32) {}
  pub fn add_pb(&self, sz: u32, c: &Literal, coeffs: Vec<u32>, k: u32) {}
  pub fn config(&self) -> &LocalSearchConfig  { 
    return &self.config;  
  }

  // endregion public methods
  
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
