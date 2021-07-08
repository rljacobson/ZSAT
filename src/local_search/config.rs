/*!



*/






use core::default::Default;
use super::{
  LocalSearchMode
};

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}

// region LocalSearchConfig
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LocalSearchConfig {
  pub random_seed     : u32,
  pub best_known_value: i32,
  pub mode            : LocalSearchMode,
  pub phase_sticky    : bool,
  pub dbg_flips       : bool, // todo: Only define when in debug mode?
  pub itau            : f64,
}

impl LocalSearchConfig {
  pub fn new() -> Self {
    Self::default()
  }
  pub fn mode(&self) -> LocalSearchMode {
    self.mode
  }
  pub fn phase_sticky(&self) -> bool {
    self.phase_sticky
  }
  pub fn dbg_flips(&self) -> bool {
    self.dbg_flips
  }
  pub fn itau(&self) -> f64 {
    self.itau
  }
  pub fn random_seed(&self) -> u32 {
    self.random_seed
  }
  pub fn best_known_value(&self) -> i32 {
    self.best_known_value
  }
  pub fn set_random_seed(&mut self, random_seed: u32) {
    self.random_seed = random_seed;
  }
  pub fn set_best_known_value(&mut self, best_known_value: i32) {
    self.best_known_value = best_known_value;
  }

  pub(crate) fn set_config(&mut self, cfg: &LocalSearchConfig) {
    self.mode         = cfg.local_search_mode;
    self.random_seed  = cfg.random_seed;
    self.phase_sticky = cfg.phase_sticky;
    self.dbg_flips    = cfg.local_search_dbg_flips;
  }
}

impl Default for LocalSearchConfig {
  fn default() -> Self {
    LocalSearchConfig {
      random_seed     : 0u32,
      best_known_value: i32::MAX,
      mode            : LocalSearchMode::WSAT,
      phase_sticky    : false,
      dbg_flips       : false,
      itau            : 0.5f64,
    }
  }
}
