/*

    A timer that puts the thread to sleep for a period of time.

*/

use crate::timer::schedule_wake;
use crate::timer::error::TimerThreadNotStarted;

use core::future::Future;
use core::pin::Pin;
use core::task::{ Context, Poll, Waker };
use core::time::Duration;
use std::sync::{ Arc, Mutex };
use std::time::Instant;


//------------------------------------------------------------------------------
//  Returns after `deadline` .
//------------------------------------------------------------------------------
pub async fn sleep_until( deadline: Instant )
{
    SleepFuture::new(deadline).await.unwrap();
}


//------------------------------------------------------------------------------
//  Returns `duration` time from now.
//------------------------------------------------------------------------------
pub async fn sleep_for( duration: Duration )
{
    SleepFuture::new(Instant::now() + duration).await.unwrap();
}


//------------------------------------------------------------------------------
//  Future that sleeps for a certain period of time using a timer thread.
//------------------------------------------------------------------------------
pub struct SleepFuture
{
    deadline: Instant,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl SleepFuture
{
    //--------------------------------------------------------------------------
    //  Creates a new `SleepFuture` . When you add `.await` to this, current
    //  thread will sleep for the specified period.
    //--------------------------------------------------------------------------
    pub fn new( deadline: Instant ) -> Self
    {
        Self
        {
            deadline,
            waker: Arc::new(Mutex::new(None)),
        }
    }
}

impl Future for SleepFuture
{
    type Output = Result<(), TimerThreadNotStarted>;

    //--------------------------------------------------------------------------
    //  Use a timer thread to return `Poll::Ready` when the scheduled datetime
    //  come.
    //--------------------------------------------------------------------------
    fn poll( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Self::Output>
    {
        //  If the schedule datetime is in the past, returns `Poll::Ready`
        //  immediately.
        if self.deadline < Instant::now()
        {
            return Poll::Ready(Ok(()));
        }

        //  Schedules a `wake()` call on a timer thread.
        let old_waker = self.waker.lock().unwrap().replace(cx.waker().clone());
        if old_waker.is_none()
        {
            schedule_wake(self.deadline, self.waker.clone())?;
        }

        Poll::Pending
    }
}
