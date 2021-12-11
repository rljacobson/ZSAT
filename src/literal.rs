/*!
A boolean `Literal` type that encodes its sign and corresponding variable in a single
`usize`.

A literal is an instance of a variable appearing either with or without negation. Thus, the variable
$x_j$ has corresponding literals $x_j$ and $\overline{x}_j$. One may think of a literal as a signed
version of its variable.

That is, the `Literal` b is represented by the value 2*b, and the `Literal` (not b) by the value 2*b
+ 1. We distinguish `Literal(NULL_BOOL_VAR << 1) == Literal(BoolVariable::MAX - 1)` as the "null"
literal. Note that `BoolVariable::MAX - 1` is the largest `BoolVariable` value that can be
represented with a `Literal`.

`Literal` is a wrapper around `BoolVariable` that stores the _value_ of the variable. The
`BoolVariable` itself stores the _name_ (identity/index) of a variable.

*/

use std::fmt::{Display, Formatter};

use crate::{BoolVariable, NULL_BOOL_VAR, UIntSet};
use crate::data_structures::OredIntegerSet;

pub type LiteralVector = Vec<Literal>;
pub type LiteralApproximateSet = OredIntegerSet<usize, Literal>;

// region Literal

// todo: Clarify when `Literal` is converted to `BoolVariable` using `Literal::var()` versus just setting
//       `BoolVariable` to `literial.0`. Seems like the second option should never happen, but it does.
//       Likewise for converting `BoolVariable` to `Literal`.

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Default)]
pub struct Literal(pub BoolVariable);

impl Literal {
  // todo: Is "null" supposed to be `Literal(NULL_BOOL_VAR << 1)` and zero `Literal(0)`?
  pub(crate) const NULL: Self = Literal(NULL_BOOL_VAR << 1);
  pub(crate) const ZERO: Self = Literal(0);

  pub fn new(v: BoolVariable, sign: bool) -> Literal {
    if sign {
      Literal((v << 1) + 1)
    } else {
      Literal(v << 1)
    }
  }

  /// Gives the value this `Literal` represents.
  pub const fn var(&self) -> BoolVariable {
    self.0 >> 1
  }

  /// Returns `true` if the `Literal` is tagged as true, otherwise `false`.
  // todo: Rename `sign` to, say, `is_positive` or `is_not_negated`. Likewise with `unsign` (abs).
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

  /// Gives underlying `BoolVar` with sign encoded in LSB.
  pub const fn index(&self) -> BoolVariable {
    self.0
  }
}

impl Default for Literal {
  fn default() -> Self {
    Self::NULL
  }
}

/// `Not` and `Neg` are the same operation for `Literal`.
impl std::ops::Not for Literal {
  type Output = Self;

  fn not(self) -> Self::Output {
    Literal(self.0 ^ 1)
  }
}

/// `Not` and `Neg` are the same operation for `Literal`.
impl std::ops::Neg for Literal {
  type Output = Self;

  fn neg(self) -> Self::Output {
    !self
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

// impl From<Literal> for usize {
//   fn from(literal: Literal) -> Self {
//     literal.0
//   }
// }

impl Display for Literal {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self{
      NULL_BOOL_VAR   => write!(f, "null"),
      _t if _t.sign() => write!(f, "-{}", self.var()),
      _               => write!(f, "{}", self.var()),
    }
  }
}

// endregion

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LiteralSet {
  set: UIntSet
}

/// Negates all literals in the vector in-place.
pub fn negate_literals(literals: &mut LiteralVector) {
  for literal in literals {
    literal.negate();
  }
}
