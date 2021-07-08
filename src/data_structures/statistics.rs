/*!

  This `Statistics` struct is the only crate-level statistics struct. Other submodules have their
   own local `Statistics` struct for the statistics collected by its items.

*/

use std::collections::HashMap;
use std::fmt::{Display, Formatter};


pub type Statistics = HashMap<&'static str, Statistic>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Statistic {
  Integer(usize),
  Float(f64)
}

impl From<f64> for Statistic {
  fn from(r: f64) -> Self {
    Statistic::Float(r)
  }
}

impl From<usize> for Statistic {
  fn from(n: usize) -> Self {
    Statistic::Integer(n)
  }
}


impl From<u32> for Statistic {
  fn from(n: u32) -> Self {
    Statistic::Integer(n as usize)
  }
}


impl Display for Statistic{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self{
      Statistic::Integer(n) => write!(f, "{}", n),
      Statistic::Float(r)    => write!(f, "{}", r)
    }
  }
}
