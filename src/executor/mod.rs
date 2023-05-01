/*

    Async executors.

*/

mod task;

use crate::threadpool::ThreadPool;
use task::Task;

use std::task::Poll;
use std::marker::Unpin;
use std::pin::Pin;
use std::future::Future;
use std::sync::{ Mutex, Arc };
use std::sync::mpsc::{ self, SyncSender, Sender, Receiver };


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
    //  Block on
    //--------------------------------------------------------------------------
    pub fn block_on<T>( &self, fut: impl Future<Output = T> + Send + 'static )
    {
        self.block_on_unpin(Box::pin(fut));
    }

    pub fn block_on_unpin<T>
    (
        &self,
        mut fut: impl Future<Output = T> + Send + Unpin + 'static
    ) -> T
    {
        struct BlockOnTaskWaker(Mutex<Option<SyncSender<()>>>);
        impl std::task::Wake for BlockOnTaskWaker
        {
            fn wake( self: Arc<Self> )
            {
                if let Some(sender) = self.0.lock().unwrap().take()
                {
                    let _ = sender.send(());
                }
            }
        }

        loop
        {
            let (sender, receiver) = mpsc::sync_channel(1);
            let waker = std::task::Waker::from(Arc::new
            (
                BlockOnTaskWaker(Mutex::new(Some(sender)))
            ));
            let mut cx = std::task::Context::from_waker(&waker);
            if let Poll::Ready(result) = Pin::new(&mut fut).poll(&mut cx)
            {
                return result;
            }
            receiver.recv().unwrap();
        }
    }

    //--------------------------------------------------------------------------
    //  Spawns a task and schedule it.
    //--------------------------------------------------------------------------
    pub fn spawn( &self, fut: impl Future<Output = ()> + Send + 'static )
    {
        self.spawn_unpin(Box::pin(fut));
    }

    pub fn spawn_unpin
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
    fn block_on()
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
                    cx.waker().clone().wake();
                    return Poll::Pending;
                }
                else if self.state == 1
                {
                    self.state = 2;
                    println!("World");
                    cx.waker().clone().wake();
                    return Poll::Pending;
                }
                else if self.state == 2
                {
                    println!("Async");
                    cx.waker().clone().wake();
                    return Poll::Ready(());
                }
                else
                {
                    self.state = 0;
                    cx.waker().clone().wake();
                    return Poll::Pending;
                }
            }
        }

        let executor = Executor::new();
        executor.block_on(async
        {
            TestFuture{ state: 0 }.await;
            TestFuture{ state: 0 }.await;
            TestFuture{ state: 0 }.await;
            TestFuture{ state: 0 }.await;
        });
        //executor.run();
    }

    #[test]
    fn run()
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
                    cx.waker().clone().wake();
                    return Poll::Pending;
                }
                else if self.state == 1
                {
                    self.state = 2;
                    println!("World");
                    cx.waker().clone().wake();
                    return Poll::Pending;
                }
                else if self.state == 2
                {
                    println!("Async");
                    cx.waker().clone().wake();
                    return Poll::Ready(());
                }
                else
                {
                    self.state = 0;
                    cx.waker().clone().wake();
                    return Poll::Pending;
                }
            }
        }

        let executor = Executor::new();
        executor.spawn(async
        {
            TestFuture{ state: 0 }.await;
            TestFuture{ state: 0 }.await;
            TestFuture{ state: 0 }.await;
            TestFuture{ state: 0 }.await;
        });
        executor.run();
    }
}
