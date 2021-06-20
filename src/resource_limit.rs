/*!
  
  An aggregate type describing limits on the resources a solver is allowed to use.
  
*/

use std::sync::{RwLock, Arc, RwLockWriteGuard, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU32};

static ZSAT_CANCELED_MSG     : &str = "canceled";
static ZSAT_MAX_MEMORY_MSG   : &str = "max. memory exceeded";
static ZSAT_MAX_SCOPES_MSG   : &str = "max. scopes exceeded";
static ZSAT_MAX_STEPS_MSG    : &str = "max. steps exceeded";
static ZSAT_MAX_FRAMES_MSG   : &str = "max. frames exceeded";
static ZSAT_NO_PROOFS_MSG    : &str = "component does not support proof generation";
static ZSAT_MAX_RESOURCE_MSG : &str = "max. resource limit exceeded";

static GLOBAL_RESOURCE_LIMIT_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default)]
pub struct ResourceLimit {
  //friend class scoped_suspend_rlimit;
  cancel : AtomicU32,
  // pub(in ScopedSuspendedResourceLimit)
  suspend: bool,
  count  : u64, // todo: Shouldn't this be guarded? Or at least atomic?
  /// The min element of `self.limits`.
  limit: u64,
  /// A non-increasing sequence consisting of previous values of `self.limit`.
  // tod: Why are we keeping track of the limits anyway? 
  limits: Vec<u64>,
  children: Vec<ResourceLimit>, // todo: Is Arc needed here?
}

impl ResourceLimit {

  /// Sets `self.cancel` without acquiring a lock from the mutex.
  ///
  /// We allow this because we want the
  /// parent to be able to set the `cancel` of its children without acquiring a lock for each child.
  /// Only the parent needs to acquire a lock, and only the parent's `cancel` is set externally.
  fn set_cancel(&mut self, n: u32) {
    self.cancel = n.into();
    for child in &mut self.children{
      child.set_cancel(n +1)
    }
  }

  pub fn new() -> Self{
    Self::default()
  }

  /// The smallest of the existing limit and `new_limit` becomes the new limit, and the old limit is
  /// pushed onto `limits`.
  ///
  /// Trying to push `0` is equivalent to trying to pushing `u64::MAX`. Otherwise, it's a saturating
  //  add. One can think of `u64::MAX` as "unlimited".
  pub fn push(&mut self, delta_limit: u32) {
    let new_limit = match delta_limit as u64 {
      0 => u64::MAX,
      _ => self.count.saturating_add(delta_limit as u64)
    };

    self.limits.push_back(self.limit);
    self.limit = u64::min(new_limit, self.limit);

    // todo: Why is this not guarded? Also, why aren't the children also reset? (Could use `reset_cancel()`.
    self.cancel = 0.into();
  }

  pub fn pop(&mut self){
    if self.count > self.limit {
      self.count = self.limit;
    }
    self.limit = self.limits.pop().unwrap();
    self.cancel = 0.into();
  }

  pub fn push_child(&mut self, resource_limit: ResourceLimit){
    #[allow(dead_code)]
    let lock = GLOBAL_RESOURCE_LIMIT_MUTEX.lock().unwrap();
    self.children.push(resource_limit);
  }

  pub fn pop_child(&mut self){
    #[allow(dead_code)]
    let lock = GLOBAL_RESOURCE_LIMIT_MUTEX.lock().unwrap();
    self.children.pop();
  }
  
  /// Increments the `count` by `n` and returns `not_cancelled()`.
  // Todo: Why not return `is_cancelled()`?
  // Todo: Should `is_cancelled()`/`not_cancelled()` return an enum variant?
  pub fn inc_by(&mut self, n: u32) -> bool {
    self.count += n;
    self.not_canceled()
  }

  /// Increments the `count` by 1 and returns `not_cancelled()`.
  pub fn inc(&mut self) -> bool {
    self.inc_by(1)
  }

  /// Read-only accessor for Self.count
  pub fn count(&self) -> u64 {
    self.count
  }

  /// Read-only accessor for Self.suspend.
  // todo: Shouldn't we call this `suspend`? Or at least `is_suspended`?
  pub fn suspended(&self) -> bool {
    self.suspend
  }

  pub fn not_canceled(&self) -> bool {
    (self.cancel == 0 && self.count <= self.limit) || self.suspend
  }

  pub fn is_canceled(&self) -> bool {
    !self.not_canceled()
  }

  pub fn get_cancel_msg(&self) -> &'static str {
    return if self.cancel > 0 {
      ZSAT_CANCELED_MSG
    } else {
      ZSAT_MAX_RESOURCE_MSG
    }
  }

  pub fn cancel(&mut self) {
    #[allow(dead_code)]
    let lock = GLOBAL_RESOURCE_LIMIT_MUTEX.lock().unwrap();
    self.set_cancel(*self.cancel + 1)
    // `GLOBAL_RESOURCE_LIMIT_MUTEX` is unlocked as `lock` goes out of scope.
  }

  pub fn reset_cancel(&mut self){
    #[allow(dead_code)]
    let lock = GLOBAL_RESOURCE_LIMIT_MUTEX.lock().unwrap();
    self.set_cancel(0)
    // `GLOBAL_RESOURCE_LIMIT_MUTEX` is unlocked as `lock` goes out of scope.
  }

  pub fn inc_cancel(&mut self) {
    self.cancel();
  }

  pub fn dec_cancel(&mut self) {
    #[allow(dead_code)]
    let lock = GLOBAL_RESOURCE_LIMIT_MUTEX.lock().unwrap();
    if self.cancel > 0 {
      set_cancel(*self.cancel - 1);
    }
  }

}

/**
  A `ScopedResourceLimit` manages a single `ResourceLimit` during the `ScopedResourceLimit`'s
  lifetime, typically within its scope of creation. It pushes a `limit`/`u64` onto the
  `ResourceLimit` in its constructor and pops it in its destructor.
*/
pub  struct ScopedResourceLimit {
  resource_limit: Arc<ResourceLimit>
}

impl ScopedResourceLimit{
  pub fn new(mut resource_limit: Arc<ResourceLimit>, limit: u32) -> ScopedResourceLimit {
    resource_limit.push(limit);
    ScopedResourceLimit{
      resource_limit: resource_limit
    }
  }
}

impl Drop for ScopedResourceLimit{
  fn drop(&mut self) {
    self.resource_limit.pop()
  }
}

/**
  A `ScopedSuspendedResourceLimit` manages a single `ResourceLimit` during the
  `ScopedSuspendedResourceLimit`'s lifetime, typically within its scope of creation, during which
  time it keeps the `ResourceLimit` suspended. Alternatively, the `ScopedSuspendedResourceLimit`
  can be created with a provided suspend state, and the `ResourceLimit` under control is suspended
  if either it is already suspended or if the provided suspend state is true; otherwise it is not
  suspended.
*/
pub struct ScopedSuspendedResourceLimit {
  resource_limit        : Arc<ResourceLimit>,
  original_suspend_state: bool
}

impl ScopedSuspendedResourceLimit{
  pub fn new(mut resource_limit: Arc<ResourceLimit>) -> ScopedSuspendedResourceLimit {
    let original_suspend_state = resource_limit.suspend;

    resource_limit.suspend = true;

    ScopedSuspendedResourceLimit{
      resource_limit,
      original_suspend_state
    }
  }

  pub fn new_with_state(mut resource_limit: Arc<ResourceLimit>, suspend: bool) -> ScopedSuspendedResourceLimit {
    let original_suspend_state = resource_limit.suspend;
    resource_limit.suspend |= suspend;

    ScopedSuspendedResourceLimit {
      resource_limit,
      original_suspend_state
    }
  }

}

impl Drop for ScopedSuspendedResourceLimit{
  fn drop(&mut self) {
    self.resource_limit.suspend = self.original_suspend_state;
  }
}


/**
  Same as `ScopedResourceLimit`, except it keeps track of how many children are pushed onto the
  `ResourceLimit` and pops the same number in its destructor. Thus `ScopedResourceLimit` is a
  special case of this struct.
*/
pub  struct ScopedResourceLimits {
  resource_limit: Arc<ResourceLimit>,
  push_count: u32
}

impl ScopedResourceLimits{
  pub fn new(mut resource_limit: Arc<ResourceLimit>, limit: u32) -> ScopedResourceLimits {
    resource_limit.push(limit);
    ScopedResourceLimits{
      resource_limit,
      push_count: 0
    }
  }

  pub fn push(&mut self, delta_limit: u32){
    self.resource_limit.push(delta_limit);
    self.push_count += 1;
  }
}

impl Drop for ScopedResourceLimits{
  fn drop(&mut self) {
    for n in 0..self.push_count {
      self.resource_limit.pop()
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
