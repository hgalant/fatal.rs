#[macro_export]
/// Prints to standard-error and exits with an error-code. Returns [!](https://doc.rust-lang.org/std/primitive.never.html).
///
/// Equivalent to [eprintln!](::std::eprintln) followed by [process::exit](::std::process::exit).
macro_rules! fatal {
  () => { ::std::process::exit(1) };
  ($($arg:tt)*) => {
    {
      ::std::eprintln!($($arg)*);
      $crate::fatal!()
     }
  };
}
