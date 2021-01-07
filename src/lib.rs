//! Utilities for reporting fatal errors and exiting with an error code.
//!
//! The behavior in this crate is different than the one in [`panic!`](::std::panic)-based exits,
//! in that the ones here are suited for display to end-users, i.e. no "thread `main` panicked at", no backtrace mentions, etc.
//!
//! # Usage
//! - Use [`error!`](error) to report context + error.
//! - Use [`unwrap`](unwrap) to report [`Result`](Result) error when context is provided/obvious.
//! - Use [`fatal!`](fatal) when [`error!`](error)'s prefix is unwelcome.
//!
//! # (Pseudo-)Example:
//! ```ignore
//! fn main() {
//!     let constr: String = std::env::var("DB_CONNECTION_STRING").unwrap_or_else(|e|
//!         fatal::error!("Failed to read the 'DB_CONNECTION_STRING' environment variable ({})", e)
//!     );
//!     println!("Connecting to database..");
//!     let db: Database = fatal::unwrap(Database::connect(&constr));
//!     println!("Total users: {}", db.query_total_users());
//! }
//! ```

use std::fmt::Display;

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
  use std::io::Write;
  let mut stderr = termcolor::StandardStream::stderr(termcolor::ColorChoice::Auto);
  if termcolor::WriteColor::set_color(&mut stderr, termcolor::ColorSpec::new().set_fg(Some(termcolor::Color::Red))).is_err() { return false }
  let did_write = write!(&mut stderr, get_error_prefix!()).is_ok();
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

/// Unwraps a result or reports its error and exits.
///
/// The error is reported with [`error!`](error).
///
/// See [`UnwrapExt`](UnwrapExt) for an extension trait version.
///
/// # User Experience
/// Be mindful to not be too lazy because error values usually don't have the context to report even remotely acceptable messages.
/// If context wasn't provided or isn't otherwise obvious, you should probably use [`error!`](error).
pub fn unwrap<T,E: Display>(result: Result<T,E>) -> T {
  result.unwrap_or_else(|e| error!("{}", e))
}

/// An extension trait for [`unwrap`](unwrap).
pub trait UnwrapExt {
  type T;

  /// An extension synonym for [`unwrap`](unwrap).
  fn unwrap_fatal(self) -> Self::T;
}

impl<T,E: Display> UnwrapExt for Result<T,E> {
  type T = T;
  fn unwrap_fatal(self) -> Self::T { unwrap(self) }
}