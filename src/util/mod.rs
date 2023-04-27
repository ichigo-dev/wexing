pub mod atomic_counter;
pub use atomic_counter::*;

use core::time::Duration;


//------------------------------------------------------------------------------
//  Utilities
//------------------------------------------------------------------------------
pub(crate) fn sleep_ms( ms: u64 )
{
    std::thread::sleep(Duration::from_millis(ms));
}

pub(crate) fn err_eq( a: &std::io::Error, b: &std::io::Error ) -> bool
{
    a.kind() == b.kind() && format!("{}", a) == format!("{}", b)
}

