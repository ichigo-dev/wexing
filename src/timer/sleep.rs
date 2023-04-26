use crate::timer::schedule_wake;
use crate::timer::error::TimerThreadNotStarted;

use core::future::Future;
use core::pin::Pin;
use core::task::{ Context, Poll, Waker };
use core::time::Duration;
use std::sync::{ Arc, Mutex };
use std::time::Instant;


pub async fn sleep_until( deadline: Instant )
{
    SleepFuture::new(deadline).await.unwrap();
}

pub async fn sleep_for( duration: Duration )
{
    SleepFuture::new(Instant::now() + duration).await.unwrap();
}

pub struct SleepFuture
{
    deadline: Instant,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl SleepFuture
{
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

    fn poll( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Self::Output>
    {
        if self.deadline < Instant::now()
        {
            return Poll::Ready(Ok(()));
        }

        let old_waker = self.waker.lock().unwrap().replace(cx.waker().clone());
        if old_waker.is_none()
        {
            schedule_wake(self.deadline, self.waker.clone())?;
        }

        Poll::Pending
    }
}
