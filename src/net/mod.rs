/*

    Asynchronous support for standard library network communication mechanisms.

*/

mod tcp_stream;
pub use tcp_stream::*;

mod tcp_listener;
pub use tcp_listener::*;

use core::time::Duration;

async fn sleep()
{
    crate::timer::sleep_for(Duration::from_millis(25)).await;
}
