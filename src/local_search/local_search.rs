/*!

The struct `LocalSearch` implements standard stochastic local search, more or less
[WalkSAT](https://en.wikipedia.org/wiki/WalkSAT).

`LocalSearchCore` is an interface, implemented by `LocalSearch` which may be implemented by other search/SMT
strategies in the context of an SMT solver. It is not strictly necessary just for SAT, but I brought it in from z3
anyway.

*/

use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;
use std::time::{Duration, Instant};

use itertools::Itertools;
use num_traits::abs;

use crate::{
  BoolVariable,
  BoolVariableVector,
  data_structures::RandomGenerator,
  errors::Error,
  LiftedBool,
  Literal,
  LiteralVector,
  log::log_at_level,
  missing_types::{Parallel},
  Model,
  NULL_BOOL_VAR,
  ResourceLimit,
  Solver,
  Statistics,
};
// use crate::local_search::;

use super::{
  config::LocalSearchConfig,
  constraint::Constraint,
  LocalSearchStatistics,
  PbCoefficient,
  variable_info::VariableInfo
};
use crate::missing_types::ParametersRef;

type RcRc<T> = Rc<RefCell<T>>;

pub trait LocalSearchCore {
  fn add(&mut self, solver: &Solver);
  fn update_parameters(&mut self, p: ParametersRef);
  fn set_seed(&mut self, s: u32);
  fn check(&self, assumptions: LiteralVector, parallel: &Parallel) -> LiftedBool;
  fn reinit_with_solver(&mut self, solver: &Solver);
  fn num_non_binary_clauses(&self) -> u32;
  fn resource_limit(&self) -> &ResourceLimit; // todo: probably use `Arc<ResourceLimit>`
  fn get_model(&self) -> &Model;
  fn collect_statistics(&self, statistics: &Statistics);
  fn get_priority(&self, _bool_var: BoolVariable) -> f64  {
    return 0f64;
  }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct LocalSearch {

  stats : LocalSearchStatistics,
  config: LocalSearchConfig,

  vars                  : Vec<VariableInfo>,  // variables
  best_phase            : Vec<bool>,          // best value in round
  units                 : BoolVariableVector, // unit clauses
  constraints           : Vec<Constraint>,    // all constraints
  assumptions           : LiteralVector,      // temporary assumptions
  prop_queue            : LiteralVector,      // propagation queue
  num_non_binary_clauses: usize,
  is_pb                 : bool,
  is_unsat              : bool,
  unsat_stack           : Vec<u32>,           // store all the unsat constraints
  index_in_unsat_stack  : Vec<usize>,           // which position is a constraint in the unsat_stack

  // configuration changed decreasing variables (score>0 and conf_change==true)
  goodvar_stack: BoolVariableVector,
  initializing : bool,


  // information about solution
  best_unsat          : usize,
  best_unsat_rate     : f64,
  last_best_unsat_rate: f64,
  // for non-known instance, set as maximal
  best_known_value    : i32, // best known value for this instance

  max_steps: u32,

  // dynamic noise
  noise      : f64, // normalized by 10000
  noise_delta: f64,

  limit    :  ResourceLimit,
  rand     :  RandomGenerator,
  parallel :  Rc<RefCell<Parallel>>,
  model    :  Model,
}

impl LocalSearch {
  pub fn new() -> Self{
    Self {
      best_known_value  :  i32::MAX,
      max_steps         :  (1 << 30),
      noise             :  9800f64,
      noise_delta       :  0.05,
      ..Self::default()
    }
  }

  // region private methods

  fn score(&self, v: BoolVariable) -> i32 {
    return self.vars[v].score;
  }
  fn inc_score(&mut self, v: BoolVariable) {
    self.vars[v].score = self.vars[v].score + 1;
  }
  fn dec_score(&mut self, v: BoolVariable) {
    self.vars[v].score = self.vars[v].score - 1;
  }

  fn slack_score(&self, v: BoolVariable) -> i32 {
    return self.vars[v].slack_score;
  }
  fn inc_slack_score(&mut self, v: BoolVariable) {
    self.vars[v].slack_score = self.vars[v].slack_score + 1;
  }
  fn dec_slack_score(&mut self, v: BoolVariable) {
    self.vars[v].slack_score = self.vars[v].slack_score - 1;
  }
  fn already_in_goodvar_stack(&self, v: BoolVariable) -> bool {
    return self.vars[v].in_goodvar_stack;
  }
  fn conf_change(&self, v: BoolVariable) -> bool {
    return self.vars[v].conf_change;
  }
  fn time_stamp(&self, v: BoolVariable) -> u32 {
    return self.vars[v].time_stamp;
  }

  fn set_best_unsat(&mut self) {
    self.best_unsat = self.unsat_stack.len();
    self.best_phase.reserve(self.vars.len());
    for i in 0..self.vars.len() {
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
    return self.constraints.len();
  }

  fn constraint_slack(&self, ci: u32) -> i64  {
    return self.constraints[ci as usize].slack;
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
    self.vars.push(VariableInfo::default());

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
    let sentinel         = self.vars.last_mut().unwrap();
    sentinel.score       = i32::MIN;
    sentinel.conf_change = false;
    sentinel.slack_score = i32::MIN;
    sentinel.time_stamp  = self.max_steps + 1;

    for v in self.vars.iter_mut() {
      v.time_stamp       = 0;
      v.conf_change      = true;
      v.in_goodvar_stack = false;
      v.score            = 0;
      v.slack_score      = 0;
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
      log_at_level(0, "unsat during reinit\n");
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

  fn init_scores(&mut self) {
    for v in 0..self.num_vars() {
      let is_true = self.cur_solution(v);
      let truep   = &self.vars[v].watch[is_true];
      let falsep  = &self.vars[v].watch[!is_true];

      for coeff in falsep {
        c = self.constraints[coeff.constraint_id];

        if c.slack <= 0 {
          self.dec_slack_score(v);
          if c.slack == 0 {
            self.dec_score(v);
          }
        }
      }
      for coeff in truep {
        let c = self.constraints[coeff.constraint_id];
        if c.slack <= -1 {
          self.inc_slack_score(v);
          if c.slack == -1 {
            self.inc_score(v);
          }
        }
      }
    }
  }

  fn init_goodvars(&mut self) {
    self.goodvar_stack.clear();
    for v in 0..num_vars(){
      if self.score(v) > 0 { // && conf_change[v] == true
        self.vars[v].in_goodvar_stack = true;
        self.goodvar_stack.push(v);
      }
    }
  }

  fn pick_flip_lookahead(&mut self) {
    // Randomly select an element from `self.unsat_stack` and get the corresponding constraint.
    let num_unsat     = self.unsat_stack.len();
    let c             = &self.constraints[self.unsat_stack[self.rand() % num_unsat] as usize];
    let mut best      = Literal::NULL;
    let mut best_make = usize::MAX;                                                            // Infinity

    let filtered_literals
        = c.literals
           .iter()
           .filter(
             | &&l | {
               !self.is_unit_literal(lit) && self.is_true_literal(lit)
             }
           );
    for &lit in filtered_literals {
      self.flip_walksat(lit.var());
      if self.propagate(!lit) && best_make > self.unsat_stack.len() {
        best      = lit;
        best_make = self.unsat_stack.len();
      }
      self.flip_walksat(lit.var());
      self.propagate(lit);
    }
    if best != Literal::NULL {
      self.flip_walksat(best.var());
      self.propagate(!best);
    }
    else {
      log_at_level(1, "(sat.local-search no best)\n");
    }
  }

  fn pick_flip_walksat(&mut self) {
    'reflip: loop{ // Loop is used as a goto target only.
      // Randomly select an element from `self.unsat_stack` and get the corresponding constraint.
      let mut num_unsat: usize        = self.unsat_stack.len();
      let c            : &Constraint  = &self.constraints[self.unsat_stack[self.rand() % num_unsat] as usize];
      let mut best_var : BoolVariable = NULL_BOOL_VAR;
      let mut n        : usize        = 1;
      // let mut v        : BoolVariable = NULL_BOOL_VAR;
      // let mut reflipped: usize        = 0;
      // let mut is_core  : bool         = self.unsat_stack.len() <= 10;
      let mut filtered_literals
          = c.literals
             .iter()
             .filter(
               | &&l| self.is_true_literal(l) && !self.is_unit_literal(l)
             );

      // Take this branch with 98% probability.
      if self.rand() % 10000 <= self.noise {
        // Find the first one in order to fast break the rest.
        let mut best_bsb = 0u64;
        let mut c_next   = filtered_literals.next();

        if c_next.is_none() {
          if c.k < self.constraint_value(&c) {
            log_at_level(0, format!("unsat clause\n{}", self.format_constraint(&c)).as_str());
            self.is_unsat = true;
            return;
          }
          continue 'reflip;
        }

        // Previous `if` block guarantees `unwrap()` will succeed.
        best_var   = c_next.unwrap().var();
        let tt     = self.cur_solution(best_var);
        let falsep = self.vars[best_var].get_watch(!tt);

        for pb_coefficient in falsep {
          let slack = self.constraint_slack(pb_coefficient.constraint_id);
          if slack < 0 {
            best_bsb += 1;
          } else if slack < (pb_coefficient.coefficient as i64) {
            best_bsb += num_unsat;
          }
        }

        for l in filtered_literals {
          let v       = l.var();
          let mut bsb = 0u64;
          let tt      = self.cur_solution(v);
          let falsep  = self.vars[v].get_watch(!tt);
          let mut it  = falsep.iter();

          for pb_coefficient in falsep {
            let slack = self.constraint_slack(pb_coefficient.constraint_id) as i64;
            if slack < 0 {
              if bsb == best_bsb {
                break;
              }
              else {
                bsb += 1;
              }
            }
            else if slack < (pb_coefficient.coeff as i64) {
              bsb += num_unsat;
              if bsb > best_bsb {
              break;
              }
            }

          }
          if let None = it.next() {
            if bsb < best_bsb {
              best_bsb = bsb;
              best_var = v;
              n = 1;
            }
            else {// if (bsb == best_bb)
              n += 1;
              if self.rand() % n == 0 {
                best_var = v;
              }
            }
          }
        }
      }
      else {
        for l in filtered_literals {
          if self.rand() % n == 0 {
            best_var = l.var();
          }
          n += 1;
        }
      }

      if best_var == NULL_BOOL_VAR {
        log_at_level(1, "(sat.local_search :unsat)\n");
        return;
      }

      if self.is_unit(best_var) {
        continue 'reflip;
      }

      self.flip_walksat(best_var);

      let lit = Literal::new(best_var, !self.cur_solution(best_var));
      if !self.propagate(lit) {
        if self.is_true_literal(lit) {
          self.flip_walksat(best_var);
        }
        self.add_unit(!lit, Literal::NULL);
        if !self.propagate(!lit) {
          log_at_level(2, "unsat\n");
          self.is_unsat = true;
          return;
        }
        if self.unsat_stack.empty(){
          return;
        }
        continue 'reflip;
      }

      // if false && is_core && C.k < constraint_value(C) {
      //   reflipped += 1;
      //   continue 'reflip;
      // }

      break;
    }
  }

  fn flip_walksat(&mut self, flipvar: BoolVariable) {

    self.stats.count_of_flips += 1;
    verify!(!self.is_unit(flipvar));

    let flipvar_info    = &mut self.vars[flipvar];
    flipvar_info.value  = !self.cur_solution(flipvar);
    flipvar_info.flips += 1;

    flipvar_info.slow_break.update(abs(flipvar_info.slack_score as f64));

    let flip_is_true = self.cur_solution(flipvar);
    let true_part    = flipvar_info.get_watch(flip_is_true);
    let false_part   = flipvar_info.get_watch(!flip_is_true);

    for pb_constraint in true_part {
      let constraint_id  = pb_constraint.constraint_id;
      let constraint     = &mut self.constraints[constraint_id as usize];
      let old_slack      = constraint.slack;
      constraint.slack  -= pb_constraint.coeff;                           // Subtrace

      #[cfg(feature = "debug")]
      verify!(self.constraint_value(constraint) + constraint.slack == constraint.k);

      // When slack transitions from non-negative to negative, the constraint goes from sat to unsat.
      if constraint.slack < 0 && old_slack >= 0 { // from non-negative to negative: sat -> unsat
        self.unsat(constraint_id);
      }
    }

    for pb_constraint in false_part {
      let constraint_id  = pb_constraint.constraint_id;
      let constraint     = &mut self.constraints[constraint_id as usize];
      let old_slack      = constraint.slack;
      constraint.slack  += pb_constraint.coeff;

      #[cfg(feature = "debug")]
      verify!(self.constraint_value(constraint) + constraint.slack == constraint.k);

      if constraint.slack < 0 && old_slack >= 0 { // from non-negative to negative: sat -> unsat
        self.sat(constraint_id);
      }
    }

    #[cfg(feature = "debug")]
    self.verify_unsat_stack();
  }

  fn propagate(&mut self, literal: Literal) -> bool  {
    let unit = self.is_unit(lit);
    verify!(self.is_true(lit));

    self.prop_queue.reset();
    self.add_propagation(lit);

    for i in 0u32..min(self.prop_queue.len(), self.vars.len()) as u32 {
      let literal_from_prop_queue = self.prop_queue[i];
      if !self.is_true(literal_from_prop_queue) {
        if self.is_unit(literal_from_prop_queue) {
          return false;
        }
        self.flip_walksat(literal_from_prop_queue.var());
        self.add_propagation(literal_from_prop_queue);
      }
    }
    if self.prop_queue.len() >= self.vars.len() {
      log_at_level(0, "propagation loop\n");
      return false;
    }
    if unit {
      for lit in self.prop_queue {
        verify!(self.is_true_literal(lit2));
        self.add_unit(lit2, lit);
      }
    }
    return true;
  }

  fn add_propagation(&mut self, literal: Literal) {
    verify!(self.is_true_literal(literal));
    for lit in self.vars[literal.var()].bin[literal.sign()] {
      if !self.is_true_literal(lit) {
        self.prop_queue.push(lit);
      }
    }
  }

  fn progress(&self, tries: u32, flips: u32, elapsed_time: f64) {
    if tries % 10 == 0 || self.unsat_stack.is_empty() {
      let rounded_elapsed_time = if elapsed_time < 0.001 {
        0.0
      } else {
        elapsed_time
      };
      log_at_level(
        1,
        format!(
          "(sat.local-search, :flips {} :noise {} :unsat {} :constraints {} :time {}\n",
          flips,
          self.noise,
          self.best_unsat,
          self.constraints.len(),
          rounded_elapsed_time
        ).as_str()
      );
    }
  }

  fn walksat(&mut self) {
    self.best_unsat_rate = 1f64;
    self.last_best_unsat_rate = 1f64;

    self.reinit();
    #[cfg(feature = "debug")]
    self.verify_slack();

    // usage: timer.elapsed().as_secs();
    let timer = Instant::now();

    let mut total_flips = 0u32;
    let mut tries = 0u32;

    while !self.unsat_stack.is_empty() && self.limit.inc(){
      // Semantically different from z3 in that z3 always sets tries = 1, while here we allow tries == 0 if body
      // never runs.
      tries += 1;
      self.stats.num_restarts += 1;
      let mut step = 0u32;

      while step < self.max_steps && !self.unsat_stack.empty() {
        self.pick_flip_walksat();

        if self.unsat_stack.len() < self.best_unsat {
          self.set_best_unsat();
          self.last_best_unsat_rate = self.best_unsat_rate;
          self.best_unsat_rate = self.unsat_stack.len() as f64 / self.num_constraints() as f64;
        }

        if self.is_unsat {
          return;
        }

        step += 1;
      }

      total_flips += step;
      self.progress(tries, total_flips, timer.elapsed().as_secs_f64());

      if self.parallel {
        let mut max_avg = 0f64;

        // Find the max of
        for  v in 0..self.num_vars() {
          max_avg = f64::max(max_avg, f64::from(self.vars[v].slow_break));
        }

        // Compute the exponential mean of deltas
        let mut sum = 0f64;
        for  v in 0..self.num_vars() {
          sum += f64::exp(self.config.itau() * (self.vars[v].slow_break - max_avg));
        }
        if sum == 0f64 {
          sum = 0.01;
        }

        // Compute the weight (break probability) for each variable
        for  v in 0..self.num_vars() {
          self.vars[v].break_prob = f64::exp(self.config.itau() * (self.vars[v].slow_break - max_avg)) / sum;
        }

        self.par.to_solver(self);
      }

      if self.par && self.par.from_solver(self)
          || tries % 10 == 0 && !self.unsat_stack.empty() {
        self.reinit();
      }
    }

    self.progress(0, total_flips, timer.elapsed().as_secs_f64());
  }

  /// Pushes `constraint` onto the `unsat_stack` and updates `index_in_unsat_stack` accordingly.
  fn unsat(&mut self, constraint: u32) {
    self.index_in_unsat_stack[constraint as usize] = self.unsat_stack.len();
    self.unsat_stack.push(constraint);
  }

  /// Removes a constraint from the `unsat_stack`.
  fn sat(&mut self, constraint: u32) {
    // Swap the deleted one with the last one and pop
    // todo: Do we need to check that `unsat_stack` is nonempty?
    let last_unsat_constraint = *self.unsat_stack.last().unwrap();
    let index = self.index_in_unsat_stack[constraint as usize];
    self.unsat_stack[index] = last_unsat_constraint;
    self.index_in_unsat_stack[last_unsat_constraint as usize] = index;
    self.unsat_stack.pop();
  }

  fn set_parameters(&mut self) {
    self.rand.set_seed(self.config.random_seed());
    self.best_known_value = self.config.best_known_value();

    self.max_steps = u32::min(
      20u32 * self.num_vars() as u32,
      1u32 << 17u32  // Cut steps off at ~100K (131,072).
    );

    trace!(
      "sat",
      println!(
        "seed:\t{}\n\
        best_known_value:\t{}\n\
        max_steps:\t{}\n",

        self.config.random_seed(),
        self.config.best_known_value(),
        self.max_steps
      )
    );
  }

  fn verify_solution(&self) {
    log_at_level(10, "verifying solution\n");
    for constraint in self.constraints{
      self.verify_constraint(&constraint)
    }
  }

  fn verify_unsat_stack(&self) {
    for i in self.unsat_stack {
      let constraint = &self.constraints[i as usize];
      if constraint.k >= self.constraint_value(constraint) {
        log_at_level(
          0,
          format!("{} {}\n", i, self.format_constraint(constraint)).as_str()
        );
        log_at_level(
          0,
          format!("units {:?}", self.units.join(" ")).as_str()
        );
      }
      verify!(constraint.k < constraint.constraint_value(constraint));
    }
  }

  fn verify_constraint(&self, constraint: &Constraint) {
    let value = self.constraint_value(constraint);
    log_at_level(11, &*format!("verify {}", c));
    trace!("sat", &*format!("verify {}", c));
    if constraint.k < value {
      log_at_level(
        0,
        format!(
          "violated constraint: {}value: {}",
          self.format_constraint(constraint),
          value
        ).as_str()
      );
    }
  }

  fn verify_slack_with_constraint(&self, constraint: &Constraint) {
    verify!(self.constraint_value(constraint) + constraint.slack == constraint.k);
  }

  // inlined
  // fn verify_slack(&self) {}

  fn verify_goodvar(&self) -> bool {
    let mut g = 0usize;
    for v in 0..self.num_vars() {
      if self.conf_change(v) && self.score(v) > 0 {
        g += 1;
      }
    }
    return g == self.goodvar_stack.len();
  }

  fn constraint_value(&self, constraint: &Constraint) -> usize  {
    let mut value = 0usize;
    for t in constraint {
      if self.is_true(t) {
        value += self.constraint_coeff(c, t);
      }
    }
    return value;
  }

  fn constraint_coefficient_with_literal(&self, c: &Constraint, l: Literal) -> u32  {
    for pb in self.vars[l.var()].get_watch(self.is_pos(l)) {
      if pb.constraint_id == c.id as u32 {
        return pb.coeff;
      }
    }
    unreachable!();
  }

  fn print_info(&self) {
    for variable in self.num_vars() {
      println!(
        "v{}\t{}\t{}\t{}\t{}\t{}",
        variable,
        self.vars[variable].neighbors.len(),
        self.cur_solution(variable),
        self.conf_change(variable),
        self.score(variable),
        self.slack_score(variable),
      );
    }
  }

  fn extract_model(&mut self) {
    self.model.clear();
    for v in self.num_vars() {
      self.model.push(
        if self.cur_solution(v) {
          LiftedBool::True
        } else {
          LiftedBool::False
        }
      );
    }
  }

  fn add_clause(&mut self, constraint: &LiteralVector) {
    // todo: Should this be just len? I.e. is sz one-based and k zero-based?
    let k = constraint.len() - 1;
    self.add_cardinality(constraint, k);
  }

  fn add_unit(&mut self, literal: Literal, explain: Literal) {
    let variable = literal.var();

    if self.is_unit(usize::from(literal)) {
      if self.vars[variable].value == literal.sign() {
        self.is_unsat = true;
      }
      return;
    }

    sassert!(!self.units.contains(&variable));

    if self.vars[variable].value == literal.sign() && !self.initializing {
      self.flip_walksat(variable);
    }
    self.vars[variable].value = !literal.sign();
    self.vars[variable].bias  = if literal.sign() { 0 } else { 100 };
    self.vars[variable].unit  = true;
    self.vars[variable].explain = explain;
    self.units.push(variable);

    #[cfg(feature = "debug")]
    self.verify_unsat_stack();
  }

  fn num_vars(&self) -> usize  {
    // var index from 1 to num_vars
    return self.vars.len() - 1;
  }

  /// Formats the `Constraints` and variables for printing out to the log (console by default).
  /// The analog of `local_search::display(std::ostream& out)`.
  fn format_constraints_and_vars(&self) -> String {
    format!(
      "{}{}",
      self.constraints.iter().map(self.format_constraint).join(""),
      self.vars.iter().enumerate().map(|v, vi | vi.format(v)).join("")
    )
  }

  /// Formats the `Constraint` with coefficients and values for printing out to the log (console by default).
  /// The analog of `local_search::display(std::ostream& out, constraint const& c)`.
  fn format_constraint(&self, constraint: &Constraint) -> String {
    let literals_list
        = constraint.iter()
                    .map(
                      |literal| {
                        let coeff = self.constraint_coefficient(constraint, literal);
                        if coeff > 1 {
                          format!("{} * {} ", coeff, literal)
                        } else {
                          format!("{} ", literal)
                        }
                      }
                    )
                    .join("");
    format!("{} <= {} lhs value: {}\n", literals_list, constraint.k, self.constraint_value(constraint))
  }

  /*
  /// Formats the `VariableInfo` for printing out to the log (console by default).
  /// The analog of `local_search::display(std::ostream& out, unsigned v, var_info const& vi)`
  fn format_var_info(v: i32, vi: VariableInfo) -> String {
    let truth
        = if vi.value {
            "true"
          } else {
            "false"
          };
    let unit_text
        = if vi.unit {
            format!(" u {}", vi.explain)
          } else {
            "".to_owned()
          };
    format!("v{} := {} bias: {}{}\n", v, truth, vi.bias, unit_text)
  }
  */

  // endregion private methods

  // region public methods

  pub fn rlimit(&self) -> &ResourceLimit  {
    return &self.limit;
  }

  pub fn check(&mut self, assumptions: &LiteralVector, parallel: RcRc<Parallel>) -> LiftedBool  {
    let mut old_parallel: RcRc<Parallel> = self.parallel.clone(); //Rc::new(RefCell::new(Parallel::default()));
    self.parallel = parallel;

    self.model.reset();
    let num_units = self.units.len();
    self.assumptions.reset();
    self.assumptions.extend(assumptions);
    self.init();

    if self.is_unsat {
      self.parallel = old_parallel;
      return LiftedBool::False;
    }

    self.walksat();

    trace!("sat", format!("{:?}\n", self.units));

    // Remove unit clauses
    for i in (num_units..self.units.len()).rev() {
      self.vars[self.units[i]].unit = false;
    }

    self.units.truncate(num_units);

    trace!("sat", {/* pass */});

    let result= // The result of the following if-else block:
      if self.is_unsat {
        LiftedBool::False
      }
      else if self.unsat_stack.empty() {
        self.verify_solution();
        self.extract_model();
        LiftedBool::True
      }
      else {
        LiftedBool::Undefined
      };

    // Remove sentinel variable
    self.vars.pop();

    log_at_level(1, format!("(sat.local-search {})\n", result).as_str());
    log_at_level(20, ""); // todo: What's the point?

    return result;
  }

  pub fn num_non_binary_clauses(&self) -> usize  {
    return self.num_non_binary_clauses;
  }

  pub fn add(&mut self, s: &Solver) {
    self.import(s, false);
  }

  pub fn get_model(&self) -> &Model  {
    return &self.model;
  }

  pub fn collect_statistics(&self, statistics: &Statistics) {
    if self.config.dbg_flips() {
      for (i, var_info) in self.vars.iter().enumerate() {
        log_at_level(
          0,
          format!(
            "flips: {} {} {}\n",
            i,
            var_info.flips,
            var_info.slow_break
          ).as_str()
        );
      }
    }
    statistics.update("local-search-flips", self.stats.count_of_flips);
    statistics.update("local-search-restarts", self.stats.count_of_restarts);
  }

  pub fn update_params(&self, _parameters: ParametersRef) {
    /* No parameters to update; pass. */
  }

  pub fn set_seed(&mut self, n: u32) {
    self.config.set_random_seed(n);
  }

  pub fn reinit_with_solver(&mut self, solver: &Solver) {
    self.import(solver, true);
    if solver.best_phase_size > 0 {
      for i in (0..self.num_vars()).rev() {
        self.set_phase(i, solver.best_phase[i]);
      }
    }
  }

  // Used by unit-walk
  pub fn set_phase(&mut self, v: BoolVariable, f: bool) {
    let mut variable = self.vars.get_mut(v).unwrap();
    if f  && variable.bias < 100 { variable.bias += 1; }
    if !f && variable.bias > 0   { variable.bias -= 1; }
  }

  pub fn set_bias(&mut self, v: BoolVariable, f: LiftedBool) {
    match f {
      LiftedBool::True => self.vars[v].bias = 99,
      LiftedBool::False => self.vars[v].bias = 1,
      _ => { /* pass */ }
    }
  }

  pub fn get_best_phase(&self, v: BoolVariable) -> bool  {
    return self.best_phase[v];
  }

  pub fn cur_solution(&self, v: BoolVariable) -> bool  {
    return self.vars[v].value;
  }

  pub fn get_priority(&self, v: BoolVariable) -> f64  {
    return self.vars[v].break_prob;
  }

  pub fn import(&mut self, s: &Solver, init: bool) -> Result<(), Error> {
    let old_initializing_value = self.initializing;
    self.initializing = true;
    self.is_pb = false;
    self.vars.reset();
    self.constraints.reset();
    self.units.reset();
    self.unsat_stack.reset();
    self.vars.reserve(s.num_vars());
    self.config.set_config(s.get_config());

    if self.config.phase_sticky() {
      for (v, vi) in self.vars.iter_mut().enumerate() {
        vi.bias = if s.phase[v] { 98 } else { 2 };
      }
    }

    // Copy units
    let trail_sz = s.init_trail_size();
    for i in 0usize..trail_sz {
      let singleton = vec![s.trail[i]];
      self.add_clause(&singleton);
    }

    // Copy binary clauses
    {
      let sz = s.watches.len();
      for l_idx in 0..sz {
        let l1 = !Literal(l_idx);
        let wlist = s.watches.get(l_idx).unwrap();
        for w in wlist {
          if !w.is_binary_non_learned_clause() {
            continue;
          }
          let l2 = w.get_literal();
          if l1.index() > l2.index() {
            continue;
          }
          // todo: WRONG
          ls = vec![l1, l2];
          self.add_clause(ls);
        }
      }
    }

    // copy clauses
    for clause in &s.clauses {
      self.add_clause(clause.literals());
    }
    self.num_non_binary_clauses = s.clauses.len();


    // Copy cardinality clauses
    // todo: Refactor `Extension::exrtract_pb()` to not borrow self twice.
    if let Some(ext) = &s.ext {
      // Used to extract PB from extension.
      // std::function<void(unsigned, literal const*, unsigned)>
      // [&](unsigned sz, literal const* c, unsigned k)
      let card = |c, k| self.add_cardinality(c, k);
      /*
      std::function<void(unsigned sz, literal const* c, unsigned const* coeffs, unsigned k)> pb =
          [&](unsigned sz, literal const* c, unsigned const* coeffs, unsigned k)
      {
        add_pb(sz, c, coeffs, k);
      }; */
      let pb = |c, coeffs, k| self.add_pb(c, coeffs, k);

      // Local search is incomplete with extensions beyond PB.

      if !ext.is_pb() || !ext.extract_pb(card, pb) {
        self.initializing = old_initializing_value;
        return Err(Error::IncompleteExtension);
      }

    }// end if Some(ext)

    if init {
      self.init();
    }

    self.initializing = old_initializing_value;
    Ok(())
  }

  pub fn add_cardinality(&mut self, c: &LiteralVector, k: usize) {
    if k == 0 && c.len() == 1 {
      self.add_unit(c[0], Literal::NULL);
      return;
    }

    if k == 1 && c.len() == 2 {
      log_at_level(0, format!("bin: {} + {} <= 1\n", !c[0], !c[1]).as_str());
      for i in 0..2 {
        let (t, s) = (c[i], c[1-i]);

        self.vars.reserve(t.var() + 1);
        self.vars[t.var()].bin[self.is_pos(t)].push(s);
      }
    }

    let id = self.constraints.len();
    self.constraints.push(Constraint::new(k, id));

    for i in 0..c.len() {
      self.vars.reserve(c[i].var() + 1);
      let t = !c[i];

      self.vars[t.var()]
          .watch[is_pos(t)]
          .push(
            PbCoefficient{
              constraint_id: id as u32,
              coefficient: 1
            }
          );

      self.constraints.last_mut().push(t);
    }

  }

  pub fn add_pb(&mut self, c: &LiteralVector, coeffs: Vec<u32>, k: u32) {
    if c.len() == 1 && k == 0 {
      self.add_unit(!c[0], Literal::NULL);
      return;
    }
    self.is_pb = true;
    let id = self.constraints.len();
    self.constraints.push(constraint(k, id));
    for i in 0..c.len() {
      self.vars.reserve(c[i].var() + 1);
      let t = c[i];
      self.vars[t.var()]
          .get_watch(self.is_pos(t))
          .push(
            PbCoefficient {
              constraint_id: id as u32,
              coefficient: coeffs[i]
            }
          );
      self.constraints.last_mut().push(t);
    }
  }

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
