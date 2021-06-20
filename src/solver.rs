/*!
    Defines the `SolverCore` trait and its canonical implementation `Solver`.
*/

use crate::{
    LiteralVector,
    model::Model,
    ResourceLimit,
    status::Status,
    statistics::Statistics as GlobalStatistics,
    Literal
};

use crate::missing_types::{
    Justification,
    Extension,
    CutSimplifier,
    Parallel,
    DRAT,
    RandomGenerator,
    ClauseAllocator,
    Cleaner,
    ModelConverter,
    Simplifier,
    SCC,
    AsymmBranch,
    BinarySPR,
    MUS,
    Probing,
    ClauseVector,
    SearchState,
    WatchList,
    ExponentialMovingAverage,
    ClauseWrapperVector,
    VariableQueue,
    ScopedLimitTrail,
    Stopwatch,
    ParamsRef,
    Cuber
};
use crate::config::Config;
use std::rc::Rc;
use crate::lifted_bool::LiftedBoolVector;
use crate::literal::LiteralSet;
use crate::local_search::LocalSearchCore;


pub trait SolverCore {
    fn new(resource_limit: &ResourceLimit) -> Self;
    fn add_clause(n: u32, literals: LiteralVector, status: Status);
    fn at_base_level() -> bool;
    fn check(literals: Vec<u32>);
    fn get_core() -> &LiteralVector;
    fn get_model() -> &Model;
    fn get_reason_unknown() -> &char;
    fn is_inconsistent() -> bool;
    fn number_of_clauses() -> u32;
    fn number_of_variables() -> u32;
    fn pop_to_base_level();
}

/// Statistics collected about the (concrete) SAT solver.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Statistics {
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

impl Statistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect_statistics(&self, statistics: &Statistics) {
        statistics.update("sat mk clause 2ary", self.mk_bin_clause);
        statistics.update("sat mk clause 3ary", self.mk_ter_clause);
        statistics.update("sat mk clause nary", self.mk_clause);
        statistics.update("sat mk var", self.mk_var);
        statistics.update("sat gc clause", self.gc_clause);
        statistics.update("sat del clause", self.del_clause);
        statistics.update("sat conflicts", self.conflict);
        statistics.update("sat decisions", self.decision);
        statistics.update("sat propagations 2ary", self.bin_propagate);
        statistics.update("sat propagations 3ary", self.ter_propagate);
        statistics.update("sat propagations nary", self.propagate);
        statistics.update("sat restarts", self.restart);
        statistics.update("sat minimized lits", self.minimized_lits);
        statistics.update("sat subs resolution dyn", self.dyn_sub_res);
        statistics.update("sat blocked correction sets", self.blocked_corr_sets);
        statistics.update("sat units", self.units);
        statistics.update("sat elim bool vars res", self.elim_var_res);
        statistics.update("sat elim bool vars bdd", self.elim_var_bdd);
        statistics.update("sat backjumps", self.backjumps);
        statistics.update("sat backtracks", self.backtracks);
    }
}

struct Scope {
    pub m_trail_lim: u32,
    pub m_clauses_to_reinit_lim: u32,
    pub m_inconsistent: bool
}

pub struct Solver<'s> {
    // private
    // todo: Should the `Rc`s in this struct be `Arc`s?
    checkpoint_enabled: bool,
    config            : Config<'s>,
    stats             : Statistics,
    ext               : Rc<Extension>,
    cut_simplifier    : Rc<CutSimplifier>,
    par               : Parallel,
    drat              : DRAT,              // DRAT for generating proofs
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
    mus               : MUS,               // MUS for minimal core extraction
    binspr            : BinarySPR,
    inconsistent      : bool,
    searching         : bool,

    // A conflict is usually a single justification. That is, a justification
    // for false. If m_not_l is not null_literal, then m_conflict is a
    // justification for l, and the conflict is union of m_no_l and m_conflict;
    conflict        : Justification,
    not_l           : Literal,
    clauses         : ClauseVector,
    learned         : ClauseVector,
    num_frozen      : u32,
    active_vars     : Vec<u32>,
    m_free_vars     : Vec<u32>,
    m_vars_to_reinit: Vec<u32>,
    watches         : Vec<WatchList>,
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
    phase                 : Vec<bool>,
    best_phase            : Vec<bool>,
    prev_phase            : Vec<bool>,
    assigned_since_gc     : Vec<char>,
    search_state          : SearchState,
    search_unsat_conflicts: u32,
    search_sat_conflicts  : u32,
    search_next_toggle    : u32,
    phase_counter         : u32,
    best_phase_size       : u32,
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
    trail                 : LiteralVector,
    clauses_to_reinit     : ClauseWrapperVector,
    reason_unknown        : String,
    visited               : Vec<u32>,
    visited_ts            : u32,


    m_scopes            : Vec<Scope>,
    m_vars_lim          : ScopedLimitTrail,
    m_stopwatch         : Stopwatch,
    m_params            : ParamsRef,
    m_clone             : Rc<Solver<'s>>,     // for debugging purposes
    m_assumptions       : LiteralVector,      // additional assumptions during check
    m_assumption_set    : LiteralSet,         // set of enabled assumptions
    m_ext_assumption_set: LiteralSet,         // set of enabled assumptions
    m_core              : LiteralVector,      // unsat core

    m_par_id             : u32,
    m_par_limit_in       : u32,
    m_par_limit_out      : u32,
    m_par_num_vars       : u32,
    m_par_syncing_clauses: bool,

    m_cuber       : Box<Cuber>,
    m_local_search: Box<dyn LocalSearchCore>,
    m_aux_stats   : Statistics

}

impl Solver {
    pub fn collect_statistics(&self, st: &Statistics){
        self.stats.collect_statistics(st);
        self.cleaner.collect_statistics(st);
        self.simplifier.collect_statistics(st);
        self.scc.collect_statistics(st);
        self.asymm_branch.collect_statistics(st);
        self.probing.collect_statistics(st);
        if self.ext {
            self.ext.collect_statistics(st);
        }
        if self.local_search{
            self.local_search.collect_statistics(st);
        }
        if self.cut_simplifier{
            self.cut_simplifier.collect_statistics(st);
        }
        st.copy(m_aux_stats);
    }
}