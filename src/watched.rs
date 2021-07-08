/*!

A watched element is an element of the SAT solver watch list. It can be a(n):

  1) literal                       : for watched binary clauses
  2) pair of literals              : for watched ternary clauses
  3) pair (literal, clause-offset) : for watched clauses, where the first element of the pair is a literal of the
                                     clause.
  4) external constraint-idx       : for external constraints.

For binary clauses we store whether the binary clause was learned or not. Note that there are no clause
objects for binary clauses.

*/

use crate::{ExtensionConstraintIndex, Literal};
use crate::clause::ClauseOffset;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Watched {
  Binary{
    literal   : Literal,
    is_learned: bool
  },

  Ternary(Literal, Literal),

  Clause{
    blocked_literal: Literal,
    clause_offset  : ClauseOffset
  },

  ExtensionConstraint(ExtensionConstraintIndex)
}

impl Watched {

  /// Determines whether `self` is equivalent to `watched`. Comparison of `Watched::Clause` is done without respect to
  /// `blocked_literal`, and comparison of `Watched::Binary` is done without respect to `is_learned`.
  pub fn matches(&self, watched: &Watched) -> bool {
    match self {

      Watched::Clause {blocked_literal: _, clause_offset} => {
        if let Watched::Clause {blocked_literal: _, clause_offset: w_clause_offset} = watched {
          w_clause_offset == clause_offset
        } else {
          false
        }
      }

      Watched::Binary{ literal, is_learned: _} => {
        if let Watched::Binary{ literal: w_literal, is_learned: _} = watched {
          w_literal == literal
        } else {
          false
        }
      }

      _ => | w | w == watched,

    }
  }

  /*  watched_lt:
              if (w2.is_binary_clause()) return false;
            if (w1.is_binary_clause()) return true;
            if (w2.is_ternary_clause()) return false;
            if (w1.is_ternary_clause()) return true;
            return false;
   */
}

/// A wrapper for `Vec<Watched>` that provides find and erase methods that compare without respect to `is_learned`
/// or, for a `Watched::Clause`, its `literal`. The wrapped `Vec` is public to provide all the usual methods if needed.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct WatchList {
  pub list: Vec<Watched>
}

impl WatchList {
  /// Remove every `Watched` equivalent to `watched`. Comparison of `Watched::Clause` is done without respect to
  /// `blocked_literal`, and comparison of `Watched::Binary` is done without respect to `is_learned`.
  pub fn erase_watch(&mut self, watched: Watched) {
    self.list.retain(
      | w | !watched.matches(w)
    );
  }

  /// Finds the first element equivalent to `watched`. Comparison of `Watched::Clause` is done without respect to
  /// `blocked_literal`, and comparison of `Watched::Binary` is done without respect to `is_learned`.
  pub fn find(&self, watched: Watched) -> Option<&Watched> {
    self.list.iter().find(
      | w | watched.matches(w)
    )
  }
}
