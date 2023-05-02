use crate::task::TaskState;
use crate::threadpool::ThreadPool;

use std::future::Future;
use std::sync::{ Arc, Mutex };
use std::marker::Unpin;
use std::pin::Pin;
use std::task::Poll;
use std::sync::mpsc::SyncSender;

struct Executor
{
    pool: ThreadPool,
}

impl Executor
{
    fn new( size: usize ) -> Self
    {
        Self::with_name(size, "wexing")
    }

    fn with_name( size: usize, thread_suffix: &'static str ) -> Self
    {
        Self
        {
            pool: ThreadPool::new(size, thread_suffix),
        }
    }

    fn run( &self )
    {
        let _ = self.pool.start_threads();
    }

    fn block_on<F, R>( &self, fut: F ) -> R
    where
        F: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        self.block_on_unpin(Box::pin(fut))
    }

    fn block_on_unpin<F, R>( &self, mut fut: F ) -> R
    where
        F: Future<Output = R> + Send + Unpin + 'static,
        R: Send + 'static,
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

        let (res_sender, res_receiver) = std::sync::mpsc::sync_channel(1);
        self.pool.start_threads().unwrap();
        self.pool.spawn(move ||
        {
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            let waker = std::task::Waker::from
            (
                Arc::new(BlockOnTaskWaker(Mutex::new(Some(sender))))
            );
            let mut cx = std::task::Context::from_waker(&waker);
            if let Poll::Ready(result) = Pin::new(&mut fut).poll(&mut cx)
            {
                res_sender.send(result).unwrap();
                return TaskState::Done;
            }
            receiver.recv().unwrap();
            TaskState::Pending
        });
        res_receiver.recv().unwrap()
    }
}

#[cfg(test)]
mod test
{
    use super::Executor;
    use std::pin::Pin;
    use std::task::{ Context, Poll };

    #[test]
    fn block_on()
    {
        enum HelloWorldState
        {
            Waiting,
            Hello,
            World,
            End
        }

        struct HelloWorldFuture
        {
            state: HelloWorldState,
        }

        impl HelloWorldFuture
        {
            fn new() -> Self
            {
                Self { state: HelloWorldState::Waiting }
            }
        }

        impl std::future::Future for HelloWorldFuture
        {
            type Output = usize;

            fn poll
            (
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Self::Output>
            {
                let waker = cx.waker().clone();
                match self.state
                {
                    HelloWorldState::Waiting =>
                    {
                        self.state = HelloWorldState::Hello;
                        waker.wake();
                    },
                    HelloWorldState::Hello =>
                    {
                        println!("Hello");
                        self.state = HelloWorldState::World;
                        waker.wake();
                    },
                    HelloWorldState::World =>
                    {
                        println!("World");
                        self.state = HelloWorldState::End;
                        waker.wake();
                    },
                    HelloWorldState::End =>
                    {
                        return Poll::Ready(200);
                    },
                }

                Poll::Pending
            }
        }

        let executor = Executor::new(4);
        let code = executor.block_on(async
        {
            HelloWorldFuture::new().await
        });
        assert_eq!(code, 200);
    }
}
