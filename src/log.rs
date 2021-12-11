/*!

  Handles emitting debug, assert, verbose, and generic log messages.

*/

pub use verbosity::*;
pub use assertions::*;
pub use trace::*;

// todo: Make thread safe.
// todo: Make generic over string type.

pub(crate) mod assertions {
  use crate::Z3_FULL_VERSION;

  // pub(crate) static mut ASSERTION_STREAM: Stdout = stdout();
  pub(crate) static mut ASSERTIONS_ENABLED: bool = true;

  /// Prints assertion violation to `stderr`.
  pub fn notify_assertion_violation(code: &str, file: &str, line: usize){
    eprintln!(
      "ASSERTION VIOLATION\n\
      File: {}\n\
      Line: {}\n\
      {}",
      file,
      line,
      code,
    );

    #[cfg(feature = "debug")]{
      eprintln!( "{}\nPlease file an issue with this message and more detail about how you encountered it at \
      https://github.com/Z3Prover/z3/issues/new\n", Z3_FULL_VERSION)
    }
  }

  pub fn invoke_debugger(){
    unimplemented!();
  }

  /// A logged assert that includes source location on failure, where failure is non-fatal, and
  /// invokes debugger. Equivalent to `SASSERT` in Z3.
  #[macro_export]
  macro_rules! log_assert {
    ($cond:expr)=>{
      {
        #[cfg(feature = "debug")]
        {
          let  assertions_enabled = true;
          unsafe{
            assertions_enabled = $crate::log::assertions::ASSERTIONS_ENABLED;
          }
          if assertions_enabled && !($cond) {
            $crate::log::assertions::notify_assertion_violation(stringify!($cond), file!(), line!());
            $crate::log::assertions::invoke_debugger();
          }
        }
      }
    }
  }

  /// A logged assert that includes source location on failure, where failure is non-fatal.
  /// Unlike `log_assert`, `verify` is not guarded by a feature flag nor does it invoke the debugger.
  #[macro_export]
  macro_rules! verify {
    ($cond:expr)=>{
      {
        if !($cond){
          // $crate::log::assertions::log_assert(stringify!($a), file!(), line!());
          $crate::log::assertions::notify_assertion_violation(
            format!("Failed to verify: {}\n", stringify!($cond)).as_str(),
            file!(),
            line!()
          );
          panic!();
        }
      }
    }
  }
}

pub(crate) mod trace {

  use std::io::{Stdout, stdout, Write};
  use std::collections::HashMap;

  pub(crate) static mut TRACE_STREAM: Stdout = stdout();
  pub(crate) static mut ENABLED_TRACES: HashMap<&str, bool> = HashMap::new();

  fn print_trace(text: &str) {
    unsafe {
      write!(TRACE_STREAM, "{}\n", text);
    }
  }

  /// Auxiliary helper for `trace!`, do not use directly.
  pub fn trace_prefix(tag: &str, function: &str, filename: &str, line_number: usize) {
    print_trace(
      format!(
        "-------- [{}] {} {}:{} ---------",
        tag,
        function,
        filename,
        line_number
      ).as_str()
    );
  }
  /// Auxiliary helper for `trace!`, do not use directly.
  pub fn trace_suffix(){
   print_trace("------------------------------------------------");
  }

  pub fn is_trace_enabled(tag: &str) -> bool {
    // Accessing mutable `static` is unsafe.
    unsafe {
      *ENABLED_TRACES.get(tag).unwrap_or(&false)
    }
  }

  pub fn update_trace(tag: &str, enable: bool) {
    unsafe {
      ENABLED_TRACES.insert(tag, enable);
    }
  }

  #[macro_export]
  macro_rules! trace {
    ($tag:expr, $code:expr) => {
      {
        if ($crate::log::trace::is_trace_enabled($tag)) {
          $crate::log::trace::trace_prefix($tag, function!(), file!(), line!()-2);
          $code ;
          $crate::log::trace::trace_suffix();
        }
      }
    }
  }
}

// Global control over verbose messaging.
pub(crate) mod verbosity {
  use std::io::{Stdout, stdout, Write};

  // todo: Make `VERBOSITY` an enum. Discriminants must be numerically compatible with Z3.
  // todo: Put `VERBOSITY` behind a mutex to get rid of `unsafe` and make thread safe.
  pub(crate) static mut VERBOSITY     : i32    = 0;
  pub(crate) static mut VERBOSE_STREAM: Stdout = stdout();

  fn verbosity_is_at_least(lvl: i32) -> bool{
    // Mutable static variables require `unsafe`, as they are not thread safe.
    unsafe{
      lvl >= VERBOSITY
    }
  }

  pub fn set_verbosity(new_value: i32) {
    unsafe {
      VERBOSITY = new_value;
    }
  }

  pub(crate) fn verbose_emit(msg: &str) {
    unsafe{
      VERBOSE_STREAM.write(msg.as_bytes())?;
    }
  }

  /// Equivalent to z3's `CASSERT`.
  // todo: Actually, `CASSERT` only runs if assertions are enabled and invokes the debugger.
  pub(crate) fn log_at_level(level: i32, msg: &str){
    if verbosity_is_at_least(level){
      verbose_emit(msg);
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
