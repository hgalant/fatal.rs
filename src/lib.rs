#[macro_export]
macro_rules! fatal {
  () => { ::std::process::exit(1) };
  ($($arg:tt)*) => {
    {
      ::std::eprintln!($($arg)*);
      $crate::fatal!()
     }
  };
}
