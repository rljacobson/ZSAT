/*!

  `Status`

*/

use crate::Theory;
use std::fmt::{Display, Formatter};


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Status {
  Input(Theory),
  Asserted(Theory),
  Redundant(Theory),
  Deleted(Theory)
}

impl Status {

  pub fn input() -> Status {
    Status::Input(-1)
  }
  pub fn asserted() -> Status {
    Status::Asserted(-1)
  }
  pub fn redundent() -> Status {
    Status::Redundant(-1)
  }
  pub fn repeated() -> Status {
    Status::Deleted(-1)
  }

  pub fn from_theory(redundent: bool, theory: Theory) -> Status {
    if redundent {
      Status::Redundant(theory)
    } else {
      Status::Asserted(theory)
    }
  }

  pub fn is_satisfied(&self) -> bool {
    -1 == self.0
  }

}

impl Display for Status {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let c =
        match self {
          Status::Input(_) => 'i',
          Status::Asserted(_) => 'a',
          Status::Redundant(_) => 'r',
          Status::Deleted(_) => 'd',
        };

    if self.is_satisfied() {
      write!(f, "{}", c)
    } else {
      write!(f, "{} k!{}", c, c.0)
    }
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
