/*

    Timer

*/

mod error;
mod sleep;
mod deadline;
pub use sleep::*;
pub use deadline::*;

use error::TimerThreadNotStarted;
use once_cell;

use core::cmp::Reverse;
use core::task::Waker;
use core::fmt::Debug;
use std::collections::BinaryHeap;
use std::sync::mpsc::{ Receiver, RecvTimeoutError, SyncSender };
use std::sync::{ Arc, Mutex };
use std::time::Instant;

type TimerThread = once_cell::sync::OnceCell<SyncSender<ScheduledWake>>;

static TIMER_THREAD_SENDER: TimerThread = once_cell::sync::OnceCell::new();


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
            .spawn(|| timer_thread_task(receiver))
            .unwrap();
        sender
    });
}

fn timer_thread_task( receiver: Receiver<ScheduledWake> )
{
    let mut heap: BinaryHeap<Reverse<ScheduledWake>> = BinaryHeap::new();
    loop
    {
        if let Some(Reverse(peeked_wake)) = heap.peek()
        {
            let now = Instant::now();
            if peeked_wake.instant < now
            {
                heap.pop().unwrap().0.wake();
            }
            else
            {
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
            heap.push(Reverse(receiver.recv().unwrap()));
        }
    }
}


//------------------------------------------------------------------------------
//  ScheduledWake
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct ScheduledWake
{
    instant: Instant,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl ScheduledWake
{
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
