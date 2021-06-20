//! `LogicalBool` is a nullable boolean type.

use std::fmt::{Display, Formatter};

pub type LiftedBoolVector = Vec<LiftedBool>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
#[repr(i8)]
pub enum LiftedBool {
  False     = -1,
  Undefined = 0,
  True      = 1,
}

impl LiftedBool {
  pub fn to_sat_str(&self) -> &'static str {
    match self{
      LiftedBool::True      => "unsatisfied",
      LiftedBool::False     => "satisfied",
      LiftedBool::Undefined => "unknown",
    }
  }
}

impl std::ops::Not for LiftedBool {
  type Output = Self;

  fn not(self) -> Self::Output {
    // todo: Just multiply by -1 instead of this.
    match self{
      LiftedBool::True      => LiftedBool::False,
      LiftedBool::False     => LiftedBool::True,
      LiftedBool::Undefined => LiftedBool::Undefined,
    }
  }
}

impl From<bool> for LiftedBool {
  fn from(b: bool) -> Self {
    match b {
      true  => LiftedBool::True,
      false => LiftedBool::False
    }
  }
}

impl Display for LiftedBool {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    // todo: Should we have `l_true`, etc., for z3 compatibility?
    write!(f, "{:?}", self)

    // match self {
    //   LiftedBool::True => write!(f, "LiftedBool::True"),
    //   LiftedBool::False => write!(f, "LiftedBool::False"),
    //   LiftedBool::Undefined => write!(f, "LiftedBool::Undefined")
    // }
  }
}
