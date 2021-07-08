/*!

Minimal Exponential Moving Average using the same algorithm as z3.

 */

/*use std::rc::Rc;

pub struct ScopedSwap<T>{
  old_value: Rc<T>,
  value: T
}

impl<'a, T> ScopedSwap<'a, T>{
  fn new(old_value: Rc<T>, new_value: &mut T) -> Self {
    let mut result = ScopedSwap{
      value:      old_value,
      old_value:  old_value
    };

  }
}*/

use std::fmt::{Display, Formatter};

pub type EMA =  ExponentialMovingAverage;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct ExponentialMovingAverage {
  alpha : f64,
  beta  : f64,
  value : f64,
  period: u32,
  wait  : u32,
}

impl Display for ExponentialMovingAverage {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    self.mean.fmt(f)
  }
}

impl ExponentialMovingAverage {

  #[cfg(feature = "debug")]
  pub fn invariant(&self) -> bool {
    verify!(0f64 <= self.alpha && self.alpha <= self.beta && self.beta <= 1f64)
  }

  pub fn new(alpha: f64) -> Self {
    let new_ema =
        ExponentialMovingAverage {
          alpha,
          beta  : 1f64,
          value : 0f64,
          period: 0u32,
          wait  : 0u32
        };

    #[cfg(feature = "debug")]
    verify!(0f64 <= self.alpha && self.alpha <= self.beta && self.beta <= 1f64);

    new_ema
  }

  pub fn set_alpha(&mut self, alpha: f64) {
    self.alpha = alpha;

    #[cfg(feature = "debug")]
    verify!(0f64 <= self.alpha && self.alpha <= self.beta && self.beta <= 1f64);
  }

  pub fn update(&mut self, value: f64) {
    // todo: Should this not be at the end?
    #[cfg(feature = "debug")]
    verify!(0f64 <= self.alpha && self.alpha <= self.beta && self.beta <= 1f64);

    self.value += self.beta * (value - self.value);

    if self.beta <= self.alpha{
      return;
    }

    if self.wait != 0 {
      self.wait -= 1;
      return;
    }
    self.wait -= 1;
    self.period = 2*(self.period + 1) - 1;
    self.wait = self.period;
    self.beta *= 0.5;

    if self.beta < self.alpha {
      self.beta = self.alpha;
    }
  }

  pub fn set_value(&mut self, value: f64) {
    self.value = value;
  }

  pub fn mean(&self) -> f64 {
    self.value
  }

}

impl Default for ExponentialMovingAverage {
  fn default() -> Self {
    ExponentialMovingAverage::new(0f64)
  }
}

impl From<EMA> for f64 {
  fn from(ema: EMA) -> Self {
    ema.value
  }
}



#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
