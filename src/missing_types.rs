/*!
  
  Types that have not yet been implemented.
  
*/

use std::rc::Rc;
use std::cell::RefCell;

pub type EventHandler = ();
/*
switch (eh.caller_id()) {
    case UNSET_EH_CALLER: break;
    case CTRL_C_EH_CALLER:
        set_reason_unknown("interrupted from keyboard");
        break;
    case TIMEOUT_EH_CALLER:
        set_reason_unknown("timeout");
        break;
    case RESLIMIT_EH_CALLER:
        set_reason_unknown("max. resource limit exceeded");
        break;
    case API_INTERRUPT_EH_CALLER:
        set_reason_unknown("interrupted");
        break;
    }
 */
pub type ASTManager = ();
pub type AsymmBranch = ();
pub type BinarySPR = ();
pub type Clause = ();
pub type ClauseAllocator = ();
pub type ClauseVector
  = Vec<Rc<Clause>>;
pub type ClauseWrapper = ();
pub type ClauseWrapperVector
  = Vec<ClauseWrapper>;
// A priority queue
pub type Cleaner = ();
pub type Cuber = ();

pub type CutSimplifier = ();
pub type DRAT = ();
pub type ExponentialMovingAverage = ();
pub type Expression = ();
pub type ExpressionVector
  = Vec<Rc<Expression>>;
pub type Extension = ();
pub type Justification = ();
pub type ModelConverter = ();
pub type MUS = ();
/// Binary Set-Propogation-Redundent Clauses
pub type Parallel = ();
pub type ParameterDescriptions = ();
pub type Parameters = ();
pub type ParametersRef
  = Rc<RefCell<Parameters>>;
pub type ParamsRef = ();
pub type Probing = ();
pub type Proof = ();
pub type RandomGenerator = ();
pub type SCC = ();
pub type ScopedLimitTrail = ();
pub type SearchState = ();
pub type Simplifier = ();
pub type Stopwatch = ();
pub type VariableQueue = ();

pub type Watched = ();
pub type WatchList
  = Vec<Watched>;


/*
  Not yet implemented:

   * Solver
   * LocalSearch
   * Statistics

  Methods Not Implemented:

    * config::Config
    * literal::LiteralSet

*/
