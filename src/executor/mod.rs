/*

    Async executors.

*/

use crate::threadpool::{ Task, ThreadPool };

use core::pin::Pin;
use core::task::Poll;
use std::future::Future;
use std::sync::{ Arc, Mutex };
use std::sync::mpsc::{ self, SyncSender };

pub struct Executor
{
    pool: ThreadPool,
}

impl Executor
{
    //--------------------------------------------------------------------------
    //  Creates an executor.
    //--------------------------------------------------------------------------
    pub fn new() -> Self
    {
        Self
        {
            pool: ThreadPool::new("wexing", 4),
        }
    }

    pub fn run( &self ) -> Result<(), std::io::Error>
    {
        self.pool.run()
    }

    pub fn schedule
    (
        &self,
        mut fut: impl (Future<Output=()>) + Send + Unpin + 'static + Sync,
    )
    {
        struct TaskWaker(Mutex<Option<SyncSender<()>>>);
        impl std::task::Wake for TaskWaker
        {
            fn wake( self: Arc<Self> )
            {
                if let Some(sender) = self.0.lock().unwrap().take()
                {
                    let _ = sender.send(());
                }
            }
        }

        let task = Task::new
        (
            Box::new(move ||
            {
                loop
                {
                    let (sender, receiver) = mpsc::sync_channel(1);
                    let waker = std::task::Waker::from(Arc::new(TaskWaker(Mutex::new(Some(sender)))));
                    let mut cx = std::task::Context::from_waker(&waker);

                    match Pin::new(&mut fut).poll(&mut cx)
                    {
                        Poll::Ready(_) => return,
                        _ => return,
                    }
                    receiver.recv().unwrap();
                }
            }),
            0
        );
        self.pool.schedule(task);
    }
}

#[cfg(test)]
mod test
{
    use super::Executor;
    use core::pin::Pin;
    use core::task::{ Poll, Context };
    use std::future::Future;

    #[test]
    fn executor()
    {
        struct TestFuture
        {
            state: usize,
        }

        impl Future for TestFuture
        {
            type Output = ();

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
            {
                if self.state == 0
                {
                    self.state = 1;
                    println!("Hello");
                    return Poll::Pending;
                }
                else if self.state == 1
                {
                    self.state = 2;
                    println!("World");
                    return Poll::Pending;
                }
                else if self.state == 2
                {
                    println!("Async");
                    return Poll::Ready(());
                }
                else
                {
                    self.state = 0;
                    return Poll::Pending;
                }
            }
        }

        let executor = Executor::new();
        executor.run();
        executor.schedule(TestFuture { state: 0 });
        executor.schedule(TestFuture { state: 0 });
        executor.schedule(TestFuture { state: 0 });
        executor.schedule(TestFuture { state: 0 });
    }
}
