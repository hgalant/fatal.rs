//! Macros for reporting fatal errors and exiting with an error code.
//!
//! All macros:
//! - Are equivalent to [`process::exit`](::std::process::exit) when no arguments are given.
//! - Return [`!`](https://doc.rust-lang.org/std/primitive.never.html).
//!
//! # (Pseudo-)Example:
//! ```ignore
//! fn execute() -> Result<A, ExecutionError> { /* ... */ }
//!
//! fn main() {
//!     let a: A = execute().wrap_or_else(|e| fatal::error!("execution failed ({})", e));
//!     /// ...
//! }
//! ```

#[cfg(feature = "color")] use std::io::Write;

#[macro_export]
/// Prints to standard-error and exits with an error-code. Returns [`!`](https://doc.rust-lang.org/std/primitive.never.html).
///
/// Equivalent to [`eprintln!`](::std::eprintln) followed by [`process::exit`](::std::process::exit).
macro_rules! fatal {
  () => { ::std::process::exit(1) };
  ($($arg:tt)*) => {
    {
      ::std::eprintln!($($arg)*);
      $crate::fatal!()
     }
  };
}

/// Yields the error prefix string.
///
/// This is a macro to minimize code generation (compared to a `println!("{}", ERROR_PREFIX_CONST)`).
macro_rules! get_error_prefix { () => {"Error: "} }

#[doc(hidden)]
/// Write the error prefix for the [error!](error) macro.
///
/// This function is internal.
pub fn internal_write_error_prefix() {
  #[cfg(feature = "color")]
  if !internal_write_red_error_prefix() { eprint!(get_error_prefix!()); }

  #[cfg(not(feature = "color"))]
  eprint!(get_error_prefix!());
}

#[doc(hidden)]
#[cfg(feature = "color")]
/// Prints the error prefix for the [error!](error) macro in red.
///
/// Returns whether the function printed, regardless if it succeeded or not.
/// In other words, if false, we should retry but fallback to normal printing.
fn internal_write_red_error_prefix() -> bool {
  let mut stderr = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
  if termcolor::WriteColor::set_color(&mut stderr, termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Red))).is_err() { return false }
  let did_write = write!(&mut stderr, get_error_prefix!()).is_err();
  termcolor::WriteColor::reset(&mut stderr)
    .ok(); // ignore any potential error, we passed the point of no return.
  did_write
}

#[macro_export]
/// Prints an error message to standard-error and exits with an error code.
///
/// Equivalent to [`fatal!`](fatal), but prefixes the message (when present) with “Error: ”.
/// If the `color` flag is set, will attempt to color the prefix in red.
macro_rules! error {
  () => { $crate::fatal!() };
  ($($arg:tt)*) => {
    {
      $crate::internal_write_error_prefix();
      $crate::fatal!($($arg)*);
    }
  };
}