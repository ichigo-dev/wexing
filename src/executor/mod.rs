/*

    Async Executor

    A safe async runtime.

*/

use crate::sync::{ self, Receiver };
use crate::threadpool::ThreadPool;
use crate::threadpool::error::NewThreadPoolError;

use core::cell::Cell;
use core::future::Future;
use core::pin::Pin;
use core::task::Poll;
use std::sync::mpsc::SyncSender;
use std::sync::{ Arc, Mutex, Weak };

type SpawnedTask =
    Arc<Mutex<Option<Box<dyn Future<Output = ()> + Send + Unpin>>>>;


//------------------------------------------------------------------------------
//  Thread local executor
//------------------------------------------------------------------------------
thread_local!
{
    static EXECUTOR: Cell<Weak<Executor>> = Cell::new(Weak::new());
}


//------------------------------------------------------------------------------
//  Gets the `Executor` from thread-local storage.
//
//  This is a low-level function.
//------------------------------------------------------------------------------
#[must_use]
pub fn get_thread_executor() -> Option<Arc<Executor>>
{
    EXECUTOR.with(|cell|
    {
        let weak = cell.take();
        let result = weak.upgrade();
        cell.set(weak);
        result
    })
}


//------------------------------------------------------------------------------
//  Sets `executor` as the `Executor` for the current thread, saving it to
//  thread-local storage.
//
//  Returns a guard struct. When the guard drops, it removes `executor` from
//  thread-local storage.
//
//  This is a low-level function.
//------------------------------------------------------------------------------
pub fn set_thread_executor( executor: Weak<Executor> ) -> ThreadExecutorGuard
{
    EXECUTOR.with(|cell| cell.set(executor));
    ThreadExecutorGuard {}
}


//------------------------------------------------------------------------------
//  Guard returned by `set_thread_executor` . On drop, it removes the
//  thread-local reference to the executor.
//------------------------------------------------------------------------------
pub struct ThreadExecutorGuard;

impl Drop for ThreadExecutorGuard
{
    fn drop( &mut self )
    {
        EXECUTOR.with(std::cell::Cell::take);
    }
}


//------------------------------------------------------------------------------
//  Async executor.
//------------------------------------------------------------------------------
pub struct Executor
{
    async_pool: ThreadPool,
    blocking_pool: ThreadPool,
}

impl Executor
{
    //--------------------------------------------------------------------------
    //  Creates a new executor with 4 async threads and 4 blocking threads.
    //--------------------------------------------------------------------------
    #[must_use]
    pub fn default() -> Arc<Self>
    {
        Self::new(4, 4).unwrap()
    }

    //--------------------------------------------------------------------------
    //  Creates a new executor.
    //
    //  `num_async_threads` is the number of threads to use for executing async
    //  tasks.
    //
    //  `num_blocking_threads` is the number of threads to use for executing
    //  blocking jobs like connecting TCP sockets and reading files.
    //--------------------------------------------------------------------------
    pub fn new
    (
        num_async_threads: usize,
        num_blocking_threads: usize,
    ) -> Result<Arc<Self>, NewThreadPoolError>
    {
        Self::with_name
        (
            "async", num_async_threads,
            "blocking", num_blocking_threads
        )
    }

    //--------------------------------------------------------------------------
    //  Creates a new executor with thread names prefixed with `name` .
    //--------------------------------------------------------------------------
    pub fn with_name
    (
        async_threads_name: &'static str,
        num_async_threads: usize,
        blocking_threads_name: &'static str,
        num_blocking_threads: usize,
    ) -> Result<Arc<Self>, NewThreadPoolError>
    {
        Ok(Arc::new(Self
        {
            async_pool: ThreadPool::new
            (
                async_threads_name,
                num_async_threads
            )?,
            blocking_pool: ThreadPool::new
            (
                blocking_threads_name,
                num_blocking_threads,
            )?,
        }))
    }

    //--------------------------------------------------------------------------
    //  Schedules a job to run on any available thread in blocking threadpool.
    //
    //  Use the returned receiver to get the result of the job.
    //  If the job panic, the receiver returns `RecvError` .
    //--------------------------------------------------------------------------
    pub fn schedule_blocking<T, F>( self: &Arc<Self>, func: F ) -> Receiver<T>
    where
        T: Send + 'static,
        F: (FnOnce() -> T) + Send + 'static,
    {
        let (sender, receiver) = sync::oneshot();
        let weak_self = Arc::downgrade(self);
        self.blocking_pool.schedule(move ||
        {
            let _guard = set_thread_executor(weak_self);
            let _result = sender.send(func());
        });
        receiver
    }

    //--------------------------------------------------------------------------
    //  Adds a task that will execute `fut` .
    //
    //  The task runs on any available worker thread. The task runs until `fut`
    //  completes or the Executor is dropped.
    //--------------------------------------------------------------------------
    pub fn spawn
    (
        self: &Arc<Self>,
        fut: impl (Future<Output = ()> ) + Send + 'static,
    )
    {
        self.spawn_unpin(Box::pin(fut));
    }

    pub fn spawn_unpin
    (
        self: &Arc<Self>,
        fut: impl (Future<Output = ()>) + Send + Unpin + 'static,
    )
    {
        let task: SpawnedTask = Arc::new(Mutex::new(Some(Box::new(fut))));
        let weak_self = Arc::downgrade(self);
        self.async_pool.schedule(move || poll_task(task, weak_self));
    }

    //--------------------------------------------------------------------------
    //  Executes the future on the current thread and returns its result.
    //
    //  `fut` can call `spawn` to create tasks. Those tasks run on the executor
    //  and will continue even after `fut` completes and this call returns.
    //--------------------------------------------------------------------------
    pub fn block_on<R>
    (
        self: &Arc<Self>,
        fut: impl (Future<Output = R>) + 'static,
    ) -> R
    {
        self.block_on_unpin(Box::pin(fut))
    }

    pub fn block_on_unpin<R>
    (
        self: &Arc<Self>,
        fut: impl (Future<Output = R>) + Unpin + 'static,
    ) -> R
    {
        let _guard = set_thread_executor(Arc::downgrade(self));
        block_on_unpin(fut)
    }
}

impl Default for Executor
{
    fn default() -> Self
    {
        Arc::try_unwrap(Executor::default()).unwrap_or_else(|_| unreachable!())
    }
}

//------------------------------------------------------------------------------
//  Schedules `func` to run on any available thread in the blocking threadpool.
//
//  Use the returned receiver to get the result of `func` . If `func` panics,
//  the receiver returns `RecvError` .
//------------------------------------------------------------------------------
pub fn schedule_blocking<T, F>( func: F ) -> Receiver<T>
where
    T: Send + 'static,
    F: (FnOnce() -> T) + Send + 'static,
{
    if let Some(executor) = get_thread_executor()
    {
        executor.schedule_blocking(func)
    }
    else
    {
        panic!("Called from outside a task; check for duplicate wexing crate.");
    }
}


//------------------------------------------------------------------------------
//  Creates a new task to execute `fut` and schedules it for immediate
//  execution.
//------------------------------------------------------------------------------
pub fn spawn( fut: impl (Future<Output = ()>) + Send + 'static )
{
    spawn_unpin(Box::pin(fut));
}

pub fn spawn_unpin( fut: impl (Future<Output = ()>) + Send + Unpin + 'static )
{
    if let Some(executor) = get_thread_executor()
    {
        let task: SpawnedTask = Arc::new(Mutex::new(Some(Box::new(fut))));
        let executor_weak = Arc::downgrade(&executor);
        executor
            .async_pool
            .schedule(move || poll_task(task, executor_weak));
    }
    else
    {
        panic!("Called from outside a task; check for duplicate wexing crate.");
    }
}


//------------------------------------------------------------------------------
//  Executes the future on the current thread and returns its result.
//------------------------------------------------------------------------------
pub fn block_on<R>( fut: impl (Future<Output = R>) + 'static ) -> R
{
    block_on_unpin(Box::pin(fut))
}

pub fn block_on_unpin<R>
(
    mut fut: impl (Future<Output = R>) + Unpin + 'static,
) -> R
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
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let waker = std::task::Waker::from
        (
            Arc::new(BlockOnTaskWaker(Mutex::new(Some(sender))))
        );
        let mut cx = std::task::Context::from_waker(&waker);
        if let Poll::Ready(result) = Pin::new(&mut fut).poll(&mut cx)
        {
            return result;
        }
        receiver.recv().unwrap();
    }
}


//------------------------------------------------------------------------------
//  TaskWaker
//------------------------------------------------------------------------------
struct TaskWaker
{
    task: SpawnedTask,
    executor: Weak<Executor>,
}

impl TaskWaker
{
    pub fn new( task: SpawnedTask, executor: Weak<Executor> ) -> Self
    {
        Self{ task, executor }
    }
}

impl std::task::Wake for TaskWaker
{
    fn wake( self: Arc<Self> )
    {
        if let Some(ref executor) = self.executor.upgrade()
        {
            let task_clone = self.task.clone();
            let executor_weak = Arc::downgrade(executor);
            executor
                .async_pool
                .schedule(move || poll_task(task_clone, executor_weak));
        }
    }
}


fn poll_task( task: SpawnedTask, executor: Weak<Executor> )
{
    if executor.strong_count() > 0
    {
        let waker = std::task::Waker::from
        (
            Arc::new(TaskWaker::new(task.clone(), executor.clone()))
        );

        let mut cx = std::task::Context::from_waker(&waker);
        let mut opt_fut_guard = task.lock().unwrap();
        if let Some(fut) = opt_fut_guard.as_mut()
        {
            let _guard = set_thread_executor(executor);
            match Pin::new(&mut *fut).poll(&mut cx)
            {
                Poll::Ready(()) => { opt_fut_guard.take(); },
                Poll::Pending => {},
            }
        }
    }
}
