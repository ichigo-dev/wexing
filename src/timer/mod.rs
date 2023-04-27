/*

    Timer to schedule Waker.


    ```rust
    use core::time::Duration;

    wexing::timer::start_timer_thread();
    let duration = Duration::from_secs(10);
    wexing::timer::sleep_for(duration).await;
    ```

    ```rust
    use core::time::Duration;
    use std::time::Instant;

    wexing::timer::start_timer_thread();
    let deadline = Instant::now() + Duration::from_secs(1);
    wexing::timer::sleep_until(deadline).await;
    ```

    ```rust
    use core::time::Duration;
    use std::time::Instant;

    async fn read_request() -> Result<(), std::io::Error> { Ok(()) }
    async fn read_data( id: () ) -> Result<(), std::io::Error> { Ok(()) }
    async fn write_data( data: () ) -> Result<(), std::io::Error> { Ok(()) }
    async fn send_response( res: () ) -> Result<(), std::io::Error> { Ok(()) }

    wexing::timer::start_timer_thread();
    let deadline = Instant::now() + Duration::from_secs(1);

    let req = wexing::timer::with_deadline(read_request(), deadline).await??;
    let data = wexing::timer::with_deadline(read_data(req), deadline).await??;
    wexing::timer::with_timeout
    (
        write_data(data),
        Duration::from_secs(1)
    ).await??;
    wexing::timer::with_timeout
    (
        send_response(()),
        Duration::from_secs(1)
    ).await??;
    ```

*/

mod error;
mod sleep;
mod deadline;
pub use sleep::*;
pub use deadline::*;

use error::TimerThreadNotStarted;
use once_cell::sync::OnceCell;

use core::cmp::Reverse;
use core::task::Waker;
use core::fmt::Debug;
use std::collections::BinaryHeap;
use std::sync::mpsc::{ Receiver, RecvTimeoutError, SyncSender };
use std::sync::{ Arc, Mutex };
use std::time::Instant;

type TimerThreadSender = OnceCell<SyncSender<ScheduledWake>>;


//------------------------------------------------------------------------------
//  Sender for sending tasks to the global timer thread.
//------------------------------------------------------------------------------
static TIMER_THREAD_SENDER: TimerThreadSender = OnceCell::new();


//------------------------------------------------------------------------------
//  Starts the worker thread, if it's not already started. You must call this
//  before calling `sleep_until` or `sleep_for` .
//------------------------------------------------------------------------------
pub fn start_timer_thread()
{
    TIMER_THREAD_SENDER.get_or_init(||
    {
        let (sender, receiver) = std::sync::mpsc::sync_channel(0);
        std::thread::Builder::new()
            .name("timer".to_string())
            .spawn(|| timer_thread(receiver))
            .unwrap();
        sender
    });
}

fn timer_thread( receiver: Receiver<ScheduledWake> )
{
    let mut heap: BinaryHeap<Reverse<ScheduledWake>> = BinaryHeap::new();
    loop
    {
        //  Takes the top of the heap ordered by scheduled datetime and compares
        //  it to the current datetime.
        if let Some(Reverse(peeked_wake)) = heap.peek()
        {
            let now = Instant::now();
            if peeked_wake.instant < now
            {
                //  Calls `wake()` if the scheduled datetime is exceeded.
                heap.pop().unwrap().0.wake();
            }
            else
            {
                //  Waits until the next scheduled datetime, but if the receiver
                //  receives a new task on the way, updates the heap and resumes
                //  processing.
                match receiver.recv_timeout
                (
                    peeked_wake.instant.saturating_duration_since(now)
                )
                {
                    Ok(new_wake) => { heap.push(Reverse(new_wake)); },
                    Err(RecvTimeoutError::Timeout) => {},
                    Err(RecvTimeoutError::Disconnected) => unreachable!(),
                }
            }
        }
        else
        {
            //  Locks this thread until task is received.
            heap.push(Reverse(receiver.recv().unwrap()));
        }
    }
}


//------------------------------------------------------------------------------
//  Schedules a `wake()` call on a timer thread.
//------------------------------------------------------------------------------
fn schedule_wake
(
    instant: Instant,
    waker: Arc<Mutex<Option<Waker>>>,
) -> Result<(), TimerThreadNotStarted>
{
    let sender = TIMER_THREAD_SENDER.get().ok_or(TimerThreadNotStarted {})?;
    sender.send(ScheduledWake { instant, waker }).unwrap();
    Ok(())
}


//------------------------------------------------------------------------------
//  A structure for executing a scheduled `wake()` . `instant` contains the
//  scheduled datetime. The timer thread compares the scheduled datetime with
//  the current datetime, and if the scheduled datetime is earlier than the
//  current datetime, `wake` is called.
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct ScheduledWake
{
    instant: Instant,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl ScheduledWake
{
    //--------------------------------------------------------------------------
    //  Calls the `wake()` of the inner waker.
    //--------------------------------------------------------------------------
    pub fn wake( &self )
    {
        if let Some(waker) = self.waker.lock().unwrap().take()
        {
            waker.wake();
        }
    }
}

impl PartialEq for ScheduledWake
{
    fn eq( &self, other: &Self ) -> bool
    {
        std::cmp::PartialEq::eq(&self.instant, &other.instant)
    }
}

impl Eq for ScheduledWake {}

impl PartialOrd for ScheduledWake
{
    fn partial_cmp( &self, other: &Self ) -> Option<core::cmp::Ordering>
    {
        std::cmp::PartialOrd::partial_cmp(&self.instant, &other.instant)
    }
}

impl Ord for ScheduledWake
{
    fn cmp( &self, other: &Self ) -> core::cmp::Ordering
    {
        std::cmp::Ord::cmp(&self.instant, &other.instant)
    }
}
