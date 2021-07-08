/*!

Implements the same pseudorandom number algorithm as Z3.

*/

pub struct RandomGenerator {
  pub data: u32
}

impl RandomGenerator {
  const MAX_VALUE: u32 = 0x7fff;

  pub fn new() -> Self{
    RandomGenerator::with_seed(0)
  }

  pub fn with_seed(seed: u32) -> Self {
    RandomGenerator {
      data: seed
    }
  }

  pub fn set_seed(&mut self, seed: u32) {
    self.data = seed;
  }

  pub fn next(&mut self) -> u32 {
    self.data = self.data * 214013 + 2531011;
    (self.data >> 16) & MAX_VALUE
  }

  pub fn at_most(&mut self, n: u32) -> u32 {
    self.next() % n
  }

}

impl FnOnce<()> for RandomGenerator {
  type Output = u32;

  fn call_once(mut self, args: ()) -> Self::Output {
    self.next()
  }
}

impl FnMut<()> for RandomGenerator{
  fn call_mut(&mut self, args: ()) -> Self::Output {
    self.next()
  }
}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
