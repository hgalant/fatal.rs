//! Utilities for reporting fatal errors and exiting with an error code.
//!
//! The behavior in this crate is different than the one in [`panic!`](::std::panic!)-based exits,
//! in that the ones here are suited for display to end-users, i.e. no "thread `main` panicked at", no backtrace mentions, etc.
//!
//! # Usage
//! For unwrapping [`Result`](Result)s:
//! - Use [`unwrap_message!`](unwrap_message) to provide context.
//! - Use [`unwrap_format!`](unwrap_format) to have more control over the message's format.
//! - Use [`unwrap`](unwrap) / [`unwrap_fatal`](UnwrapExt::unwrap_fatal) to report the error when context is provided/obvious.
//!
//! For aborting:
//! - Use [`error!`](error) to report context + error.
//! - Use [`fatal!`](fatal) when [`error!`](error)'s prefix is unwelcome.
//!
//! # (Pseudo-)Example:
//! ```ignore
//! const DB_CONSTR_VAR: &str = "DB_CONNECTION_STRING";
//!
//! fn main() {
//!     let constr: String = fatal::unwrap_message!(std::env::var(DB_CONSTR_VAR), "failed to read the `{}` environment variable", DB_CONSTR_VAR);
//!     // when doesn't exist, will print: "Error: failed to read the `DB_CONNECTION_STRING` environment variable (environment variable not found)"
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
///
/// # User Experience
/// The message you write in the arguments is in the middle of a sentence, so you may or may not want to capitalize the beginning (unless it's a proper-noun, of course).
/// Grammatically, either way is valid so it's just a matter of style.
///
/// E.g.
/// ```ignore
/// error!("bad input") // "Error: bad input"
/// error!("Bad input") // "Error: Bad input"
/// ```
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
/// If context wasn't provided or isn't otherwise obvious, you should probably use [`expect!`](expect).
pub fn unwrap<T,E: Display>(result: Result<T,E>) -> T {
  result.unwrap_or_else(|e| error!("{}", e))
}

/// Unwraps a result or reports the given message with the error and exits.
///
/// The error is reported with [`error!`](error).
///
/// See [`UnwrapExt`](UnwrapExt) for an extension trait version.
pub fn expect<T,E: Display>(result: Result<T,E>, message: impl Display) -> T {
  result.unwrap_or_else(|e| error!("{} ({})", message, e))
}

/// An extension trait for [`unwrap`](unwrap).
pub trait UnwrapExt {
  type T;

  /// An extension synonym for [`unwrap`](unwrap).
  fn unwrap_fatal(self) -> Self::T;

  /// An extension synonym for [`expect`](expect).
  fn expect_fatal(self, message: impl Display) -> Self::T;
}

impl<T,E: Display> UnwrapExt for Result<T,E> {
  type T = T;
  fn unwrap_fatal(self) -> Self::T { unwrap(self) }
  fn expect_fatal(self, message: impl Display) -> Self::T { expect(self, message) }
}

#[macro_export]
/// Unwraps the result or formats an error message and exits.
///
/// This is like [`unwrap`](unwrap) but enables formatting. The error message is in the [named parameter](https://doc.rust-lang.org/std/fmt/index.html#named-parameters)
/// `error` (i.e. `{error}` will show it).
///
/// The first argument should be a [Result](Result) such that its error implements either [`Debug`](std::fmt::Debug) or [`Display`](std::fmt::Display).
/// The rest of the arguments are as in [format!](::std::format).
///
/// The `{error}` named parameter must be used!
macro_rules! unwrap_format {
  ($result:expr, $msg:tt) => {
    $result.unwrap_or_else(|e| $crate::error!($msg, error=e))
  };
  ($result:expr, $fmt:tt, $($param:tt)*) => {
    $result.unwrap_or_else(|e| $crate::error!($fmt, $($param)*, error=e))
  };
}

/// Unwraps the result or reports the error with the error description and exits.
///
/// This is like [`unwrap_format!`](unwrap_format) but always appends the error message at the end.
#[macro_export]
macro_rules! unwrap_message {
  ($result:expr, $msg:tt) => {
    $result.unwrap_or_else(|e| $crate::error!(::std::concat!($msg, " ({error})"), error=e))
  };
  ($result:expr, $msg:tt, $($param:tt)*) => {
    $result.unwrap_or_else(|e| $crate::error!(::std::concat!($msg, " ({error})"), $($param)*, error=e))
  };
}

#[cfg(test)]
mod test {
  use super::*;

  #[allow(unreachable_code, dead_code)]
  // Just tests that expansions produce code that can even compile.
  fn test_expansions_compiles() {
    let r = Ok::<(),bool>(());
    let r = &r;

    unwrap_format!(r, "Error {error}");
    unwrap_format!(r, "Err{} {error}", "or");

    unwrap_message!(r, "Error");
    unwrap_message!(r, "Error {error}");
    unwrap_message!(r, "Err{} {error}", "or");

    error!();
    error!("Error");
    error!("Err{}", "or");

    fatal!();
    fatal!("Error");
    fatal!("Err{}", "or");
  }
}