/*

    A timer to see if the task is completed within a certain period of time.

*/

use crate::timer::schedule_wake;
use crate::timer::error::{ DeadlineError, DeadlineExceeded };

use core::future::Future;
use core::pin::Pin;
use core::task::{ Context, Poll, Waker };
use core::time::Duration;
use std::sync::{ Arc, Mutex };
use std::time::Instant;


//------------------------------------------------------------------------------
//  Awaits `inner` , but returns `DeadlineExceeded` after `deadline` .
//------------------------------------------------------------------------------
pub async fn with_deadline<Fut: Future>
(
    inner: Fut,
    deadline: Instant,
) -> Result<Fut::Output, DeadlineExceeded>
{
    match DeadlineFuture::new(Box::pin(inner), deadline).await
    {
        Ok(result) => Ok(result),
        Err(DeadlineError::DeadlineExceeded) => Err(DeadlineExceeded),
        Err(DeadlineError::TimerThreadNotStarted) =>
        {
            panic!("TimerThreadNotStarted");
        },
    }
}


//------------------------------------------------------------------------------
//  Awaits `inner` , but returns `DeadlineExceeded` after `duration` time from
//  now.
//------------------------------------------------------------------------------
pub async fn with_timeout<Fut: Future>
(
    inner: Fut,
    duration: Duration,
) -> Result<Fut::Output, DeadlineExceeded>
{
    with_deadline(inner, Instant::now() + duration).await
}


//------------------------------------------------------------------------------
//  Future that monitors whether the task is completed by the deadline.
//------------------------------------------------------------------------------
pub struct DeadlineFuture<Fut: Future + Unpin>
{
    inner: Fut,
    deadline: Instant,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<Fut: Future + Unpin> DeadlineFuture<Fut>
{
    //--------------------------------------------------------------------------
    //  Creates a new `DeadlineFuture` .
    //--------------------------------------------------------------------------
    pub fn new( inner: Fut, deadline: Instant ) -> Self
    {
        Self
        {
            inner,
            deadline,
            waker: Arc::new(Mutex::new(None)),
        }
    }
}

impl<Fut: Future + Unpin> Future for DeadlineFuture<Fut>
{
    type Output = Result<Fut::Output, DeadlineError>;

    //--------------------------------------------------------------------------
    //--------------------------------------------------------------------------
    fn poll
    (
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Self::Output>
    {
        //  If the schedule datetime is in the past, returns `DeadlineExceeded`
        //  immediately.
        if self.deadline < Instant::now()
        {
            return Poll::Ready(Err(DeadlineError::DeadlineExceeded));
        }

        //  Polls `inner` and if finished the task, returns `Poll::Ready` .
        match Pin::new(&mut self.inner).poll(cx)
        {
            Poll::Ready(r) => return Poll::Ready(Ok(r)),
            Poll::Pending => {},
        }

        //  Schedules a `wake()` call on a timer thread.
        let old_waker = self.waker.lock().unwrap().replace(cx.waker().clone());
        if old_waker.is_none()
        {
            schedule_wake(self.deadline, self.waker.clone())
                .map_err(|_| DeadlineError::TimerThreadNotStarted)?;
        }

        Poll::Pending
    }
}
