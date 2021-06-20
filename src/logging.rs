/*!
  
  Handles emitting debug, assert, verbose, and generic log messages.
  
*/


// Global control over verbose messaging.
mod verbosity {
  use std::io::{Stdout, stdout, Write};
  use std::fmt::Debug;

  // todo: put `VERBOSITY` behind a mutex
  pub(crate) static mut VERBOSITY     : i32    = 0;
  pub(crate) static mut VERBOSE_STREAM: Stdout = stdout();

  pub(crate) fn is_at_least(lvl: i32) -> bool{
    unsafe{
      lvl >= VERBOSITY
    }
  }
  pub(crate) fn emit(msg: &str) {
    unsafe{
      VERBOSE_STREAM.write(msg.as_bytes())?;
    }
  }
  pub(crate) fn emit_if_level(lvl: i32, msg: &str){
    if is_at_least(lvl){
      emit(msg);
    }
  }
}

mod debugging{

}


#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
