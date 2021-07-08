/*!



*/

use core::default::Default;
use super::{
  BoolVariableVector,
  Literal,
  LiteralVector,
  CoefficientVector
};
use crate::data_structures::{
  ExponentialMovingAverage,
  TFVectors
};

pub struct VariableInfo {
  pub(crate) bias            : u32,     // Bias for current solution in percentage.
                                        // If bias is 0, then value is always false, if 100, then always true
  pub(crate) bin             : TFVectors<LiteralVector>,     // (trues, falses)
  pub(crate) break_prob      : f64,
  pub(crate) conf_change     : bool,    // Whether its configure changed since its last flip
  pub(crate) explain         : Literal, // Explanation for unit assignment
  pub(crate) flips           : u32,
  pub(crate) in_goodvar_stack: bool,
  pub(crate) neighbors       : BoolVariableVector,           // neighborhood variables
  pub(crate) score           : i32,
  pub(crate) slack_score     : i32,
  pub(crate) slow_break      : ExponentialMovingAverage,
  pub(crate) time_stamp      : u32,     // The flip time stamp
  pub(crate) unit            : bool,    // Whether this is a unit literal
  pub(crate) value           : bool,    // Current solution
  pub(crate) watch           : TFVectors<CoefficientVector>, // (true_coeffs, false_coeffs)
}

impl Default for VariableInfo {
  fn default() -> Self {
    Self{
      value: true,
      bias : 50u32,
      unit            : false,
      explain         : Literal(0),
      conf_change     : true,
      in_goodvar_stack: false,
      score           : 0,
      slack_score     : 0,
      time_stamp      : 0,
      neighbors       : vec![],
      watch           : TFVectors::default(),
      bin             : TFVectors::default(),
      flips           : 0,
      slow_break      : ExponentialMovingAverage::new(1e-5f64),
      break_prob      : 0f64,
    }
  }
}

impl VariableInfo {
  pub fn format(&self, v: u32) -> String {
    let truth =  // the following if block:
        if self.value {
          "true"
        } else {
          "false"
        };
    let unit_text =  // the following if block:
        if self.unit {
          format!(" u {}", self.explain)
        } else {
          "".to_owned()
        };
    format!("v{} := {} bias: {}{}\n", v, truth, self.bias, unit_text)
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
