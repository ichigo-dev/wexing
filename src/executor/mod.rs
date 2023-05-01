/*

    Async executors.

*/

mod task;

use crate::threadpool::ThreadPool;
use task::Task;

use std::task::Poll;
use std::pin::Pin;
use std::marker::Unpin;
use std::future::Future;
use std::sync::mpsc::{ self, Sender, Receiver };


pub struct Executor
{
    pool: ThreadPool,
    task_sender: Sender<Task>,
    task_queue: Receiver<Task>,
}

impl Executor
{
    //--------------------------------------------------------------------------
    //  Creates an executor.
    //--------------------------------------------------------------------------
    pub fn new() -> Self
    {
        let (sender, receiver) = mpsc::channel();
        Self
        {
            pool: ThreadPool::new("wexing", 4),
            task_sender: sender,
            task_queue: receiver,
        }
    }

    //--------------------------------------------------------------------------
    //  Spawns a task and schedule it.
    //--------------------------------------------------------------------------
    pub fn spawn
    (
        &self,
        fut: impl Future<Output = ()> + Send + Unpin + 'static
    )
    {
        let task = Task::new
        (
            Pin::new(Box::new(fut)), self.task_sender.clone(),
        );
        self.schedule(task);
    }

    //--------------------------------------------------------------------------
    //  Schedules a task.
    //--------------------------------------------------------------------------
    pub fn schedule( &self, task: Task )
    {
        self.task_sender.send(task).unwrap();
    }

    //--------------------------------------------------------------------------
    //  Executes receive task.
    //--------------------------------------------------------------------------
    pub fn run( self )
    {
        loop
        {
            let mut task = self.task_queue.recv().unwrap();
            match task.poll()
            {
                Poll::Pending => { self.schedule(task) },
                Poll::Ready(()) => {},
            }
        }
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

            fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output>
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
        executor.spawn(TestFuture { state: 0 });
        executor.spawn(TestFuture { state: 0 });
        executor.spawn(TestFuture { state: 0 });
        executor.spawn(TestFuture { state: 0 });
        executor.run();
    }
}
