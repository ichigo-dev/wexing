use crate::timer::schedule_wake;
use crate::timer::error::{ DeadlineError, DeadlineExceeded };

use core::future::Future;
use core::pin::Pin;
use core::task::{ Context, Poll, Waker };
use core::time::Duration;
use std::sync::{ Arc, Mutex };
use std::time::Instant;

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

pub async fn with_timeout<Fut: Future>
(
    inner: Fut,
    duration: Duration,
) -> Result<Fut::Output, DeadlineExceeded>
{
    with_deadline(inner, Instant::now() + duration).await
}

pub struct DeadlineFuture<Fut: Future + Unpin>
{
    inner: Fut,
    deadline: Instant,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<Fut: Future + Unpin> DeadlineFuture<Fut>
{
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

    fn poll
    (
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Self::Output>
    {
        if self.deadline < Instant::now()
        {
            return Poll::Ready(Err(DeadlineError::DeadlineExceeded));
        }

        match Pin::new(&mut self.inner).poll(cx)
        {
            Poll::Ready(r) => return Poll::Ready(Ok(r)),
            Poll::Pending => {},
        }

        let old_waker = self.waker.lock().unwrap().replace(cx.waker().clone());
        if old_waker.is_none()
        {
            schedule_wake(self.deadline, self.waker.clone())
                .map_err(|_| DeadlineError::TimerThreadNotStarted)?;
        }

        Poll::Pending
    }
}
