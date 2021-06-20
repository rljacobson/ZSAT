/*!

  A boolean `Literal` type that stores its sign in the LS bit. That is, the `Literal` b is represented
  by the value 2*b, and the `Literal` (not b) by the value 2*b + 1.

  `Literal` is a wrapper around `BoolVariable` that stores the _value_ of the variable. The
  `BoolVariable` itself stores the _name_ (identity/index) of a variable.

*/

use std::fmt::{Display, Formatter};

use crate::{BoolVariable, NULL_BOOL_VAR, UIntSet};
use crate::approximate_set::OredIntegerSet;

pub type LiteralVector = Vec<Literal>;

/// Negates all literals in the vector in-place.
pub fn negate_literals(literals: &mut LiteralVector) {
  for literal in literals {
    literal.negate();
  }
}


// region Literal

// Todo: Is `LiteralApproximateSet` ever used?
// pub type LiteralApproximateSet  = OredIntegerSet<u64, BoolVar>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default)]
pub struct Literal(BoolVariable);

impl Literal {
  pub const fn new(v: BoolVariable, sign: bool) -> Literal {
    if sign {
      Literal((v << 1) + 1)
    } else {
      Literal(v << 1)
    }
  }

  pub const fn var(&self) -> BoolVariable {
    self.0 >> 1
  }

  /// Returns `true` if the `Literal` is tagged as true, otherwise `false`.
  // todo: Rename `sign` to, say, `is_positive` or `is_not_negated`. Likewise with `unsign`.
  pub const fn sign(&self) -> bool {
    (self.0 & 1) != 0
  }

  /// Returns the "unsigned" copy of `self`, a kind of absolute value.
  pub const fn unsign(&self) ->Literal {
    // Observe that `!1` has LS bit zero and all other bits one.
    Literal(self.0 & !1)
  }

  /// In-place negation.
  pub const fn negate(&mut self) {
    self.0 = self.0 ^ 1;
  }

  /// Gives underlying `BoolVar` with sign encoded un LSB.
  pub const fn index(&self) -> BoolVariable {
    self.0
  }
}

impl Default for Literal {
  fn default() -> Self {
    Literal(NULL_BOOL_VAR << 1)
  }
}

impl std::ops::Not for Literal {
  type Output = Self;

  fn not(self) -> Self::Output {
    Literal(self.0 ^ 1)
  }
}

impl std::ops::Neg for Literal {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Literal(self.0 ^ 1)
  }
}

impl From<Literal> for BoolVariable {
  fn from(literal: Literal) -> Self {
    literal.0
  }
}

impl From<BoolVariable> for Literal {
  fn from(b: BoolVariable) -> Self {
    Literal(b)
  }
}

impl Display for Literal {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self{
      NULL_BOOL_VAR   => write!(f, "null"),
      _t if _t.sign() => write!(f, "-{}", self.var),
      _               => write!(f, "{}", self.var),
    }
  }
}

// endregion

pub struct LiteralSet {
  set: UIntSet
}