/*

    Asynchronous support for standard library synchronization mechanisms.


    ```rust
    use std::sync::Arc;

    async fn some_async_fn() {}

    let shared_counter: Arc<wexing::sync::Mutex<u32>> =
        Arc::new(wexing::sync::Mutex::new(0));
    {
        let mut counter_guard = shared_counter.lock().await;
        *counter_guard += 1;
        //  some_async_fn().await;  //  Cannot await while holding a MutexGuard.
    }
    some_async_fn().await;  //  Await is ok after releasing MutexGuard.
    ```

*/

mod mutex;
pub use mutex::*;

mod channel;
pub use channel::*;
