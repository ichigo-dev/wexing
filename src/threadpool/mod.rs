/*

    ThreadPool

*/

mod error;

use crate::util::{ sleep_ms, AtomicCounter };
use crate::threadpool::error::*;

use core::fmt::{ Debug, Formatter };
use core::time::Duration;
use std::sync::mpsc::{ Receiver, RecvTimeoutError, SyncSender, TrySendError };
use std::sync::{ Arc, Mutex };
use std::time::Instant;


//------------------------------------------------------------------------------
//  Internal data held by each thread.
//------------------------------------------------------------------------------
struct Inner
{
    name: &'static str,
    next_name_num: AtomicCounter,
    size: usize,
    receiver: Mutex<Receiver<Box<dyn FnOnce() + Send>>>,
}

impl Inner
{
    //--------------------------------------------------------------------------
    //  Start threads for the specified number of threads.
    //--------------------------------------------------------------------------
    fn start_threads( self: &Arc<Self> ) -> Result<(), StartThreadsError>
    {
        while self.num_live_threads() < self.size
        {
            self.start_thread()?;
        }
        Ok(())
    }

    //--------------------------------------------------------------------------
    //  Start a thread.
    //--------------------------------------------------------------------------
    fn start_thread( self: &Arc<Self> ) -> Result<(), StartThreadsError>
    {
        let self_clone = self.clone();
        let num_live_threads = self.num_live_threads() - 1;

        if num_live_threads < self.size
        {
            self.spawn_thread
            (
                format!("{}-{}", self.name, self.next_name_num.next()),
                move || self_clone.work(),
            )
            .map_err(|e|
            {
                if num_live_threads == 0
                {
                    StartThreadsError::NoThreads(e)
                }
                else
                {
                    StartThreadsError::Respawn(e)
                }
            })?;
        }

        Ok(())
    }

    //--------------------------------------------------------------------------
    //  Spawn a thread.
    //--------------------------------------------------------------------------
    fn spawn_thread
    (
        &self,
        name: String,
        f: impl FnOnce() + Send + 'static,
    ) -> Result<(), std::io::Error>
    {
        std::thread::Builder::new().name(name).spawn(f)?;
        Ok(())
    }

    //--------------------------------------------------------------------------
    //  Returns the number of live threads.
    //--------------------------------------------------------------------------
    fn num_live_threads( self: &Arc<Self> ) -> usize
    {
        Arc::strong_count(self) - 1
    }

    //--------------------------------------------------------------------------
    //  Receive a job to run from a channel and execute it.
    //--------------------------------------------------------------------------
    fn work( self: &Arc<Self> )
    {
        loop
        {
            let recv_result = self
                .receiver
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_millis(500));
 
            //  Receive a job as a function and execute it.
            match recv_result
            {
                Ok(f) =>
                {
                    let _ignored = self.start_threads();
                    f();
                },
                Err(RecvTimeoutError::Timeout) => {},
                Err(RecvTimeoutError::Disconnected) => return,
            };

            //  Check for dead threads and restart them.
            let _ignored = self.start_threads();
        }
    }
}


//------------------------------------------------------------------------------
//  A collection of threads and a queue for jobs they execute.
//
//  Threads stop when they execute a job that panics. If one thread survives,
//  it will recreate all the threads. The next call to `schedule` and
//  `try_schedule` also recreates threads. 
//
//  If your threadpool load is bursty and you want to automatically recover from
//  an all-threads-panicked state, you could
//
//  After drop, threads stop as they become idle.
//------------------------------------------------------------------------------
pub struct ThreadPool
{
    inner: Arc<Inner>,
    sender: SyncSender<Box<dyn FnOnce() + Send>>,
}

impl ThreadPool
{
    //--------------------------------------------------------------------------
    //  Creates a new threadpool containing `size` threads. The threads all
    //  start immediately.
    //
    //  Threads are named with `name` with a number.
    //
    //  After the `ThreadPool` struct drops, the threads continue processing
    //  jobs and stop when the queue is empty.
    //--------------------------------------------------------------------------
    pub fn new
    (
        name: &'static str,
        size: usize,
    ) -> Result<Self, NewThreadPoolError>
    {
        if name.is_empty()
        {
            return Err(NewThreadPoolError::Parameter
            (
                "ThreadPool::new called with empty name".to_string(),
            ));
        }

        if size < 1
        {
            return Err(NewThreadPoolError::Parameter(format!
            (
                "ThreadPool::new called with invalid size value: {:?}",
                size
            )));
        }

        //  Use a channel with bounded size.
        //  If the channel was unbounded, the process could OOM (Out-Of-Memory)
        //  when throughput goes down.
        let (sender, receiver) = std::sync::mpsc::sync_channel(size * 200);
        let pool = ThreadPool
        {
            inner: Arc::new(Inner
            {
                name,
                next_name_num: AtomicCounter::new(),
                size,
                receiver: Mutex::new(receiver),
            }),
            sender,
        };

        pool.inner.start_threads()?;
        Ok(pool)
    }

    //--------------------------------------------------------------------------
    //  Returns the number of threads in the pool.
    //--------------------------------------------------------------------------
    #[must_use]
    pub fn size( &self ) -> usize
    {
        self.inner.size
    }

    //--------------------------------------------------------------------------
    //  Returns the number of threads currently alive.
    //--------------------------------------------------------------------------
    #[must_use]
    pub fn num_live_threads( &self ) -> usize
    {
        self.inner.num_live_threads()
    }

    //--------------------------------------------------------------------------
    //  Adds a job to the queue. The next idle thread will execute it. Jobs are
    //  started in FIFO order.
    //
    //  When the queue is full, try again untill more jobs can be added.
    //--------------------------------------------------------------------------
    pub fn schedule<F: FnOnce() + Send + 'static>( &self, f: F )
    {
        type OptBox = Option<Box<dyn FnOnce() + Send + 'static>>;
        let mut opt_box_f: OptBox = Some(Box::new(f));

        loop
        {
            //  The threads may be stopped, so check first and start.
            match self.inner.start_threads()
            {
                Ok(()) | Err(StartThreadsError::Respawn(_)) => {},
                Err(StartThreadsError::NoThreads(_)) =>
                {
                    sleep_ms(10);
                    continue;
                }
            }

            //  Send job to thread via channel.
            opt_box_f = match self.sender.try_send(opt_box_f.take().unwrap())
            {
                Ok(()) => return,
                Err(TrySendError::Disconnected(_)) => unreachable!(),
                Err(TrySendError::Full(box_f)) => Some(box_f),
            };

            //  If the channel is full, wait for a bit and retry.
            sleep_ms(10);
        }
    }

    //--------------------------------------------------------------------------
    //  Adds a job to the queue and then starts threads to replace any panicked
    //  threads. The next idle thread will execute the job. Starts jobs in FIFO
    //  order.
    //--------------------------------------------------------------------------
    pub fn try_schedule<F: FnOnce() + Send + 'static>
    (
        &self,
        f: F
    ) -> Result<(), TryScheduleError>
    {
        match self.sender.try_send(Box::new(f))
        {
            Ok(_) => {},
            Err(TrySendError::Disconnected(_)) => unreachable!(),
            Err(TrySendError::Full(_)) =>
            {
                return Err(TryScheduleError::QueueFull)
            },
        };

        self.inner.start_threads().map_err(std::convert::Into::into)
    }

    //--------------------------------------------------------------------------
    //  Consumes the thread pool and waits for all threads to stop.
    //--------------------------------------------------------------------------
    pub fn join( self )
    {
        let inner = self.inner.clone();
        drop(self);
        while inner.num_live_threads() > 0
        {
            sleep_ms(10);
        }
    }

    //--------------------------------------------------------------------------
    //  Consumes the thread pool and waits for all threads to stop.
    //--------------------------------------------------------------------------
    pub fn try_join( self, timeout: Duration ) -> Result<(), String>
    {
        let inner = self.inner.clone();
        drop(self);
        let deadline = Instant::now() + timeout;
        loop
        {
            if inner.num_live_threads() <= 0
            {
                return Ok(());
            }

            if deadline < Instant::now()
            {
                return Err
                (
                    "Timed out waiting for ThreadPool workers to stop"
                    .to_string()
                );
            }

            sleep_ms(10);
        }
    }
}

impl Debug for ThreadPool
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), core::fmt::Error>
    {
        write!
        (
            f,
            "ThreadPool{{{:?}, size={:?}}}",
            self.inner.name,
            self.inner.size
        )
    }
}
