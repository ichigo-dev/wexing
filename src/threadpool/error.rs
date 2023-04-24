/*

    ThreadPool Errors

*/

use core::fmt::{ Debug, Display, Formatter };
use std::error::Error;
use std::io::ErrorKind;


fn err_eq( a: &std::io::Error, b: &std::io::Error ) -> bool
{
    a.kind() == b.kind() && format!("{}", a) == format!("{}", b)
}


//------------------------------------------------------------------------------
//  StartThreadsError
//------------------------------------------------------------------------------
#[derive(Debug)]
pub enum StartThreadsError
{
    //  The pool has no threads and `std::threads::Builder::spawn` returned the
    //  included error.
    NoThreads(std::io::Error),

    //  The pool has at least one thread and `std::threads::Builder::spawn`
    //  returned the included error.
    Respawn(std::io::Error),
}

impl Display for StartThreadsError
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            StartThreadsError::NoThreads(e) =>
            {
                write!
                (
                    f,
                    "ThreadPool workers all panicked, failed starting \\
                    replacement threads: {}",
                    e
                )
            },
            StartThreadsError::Respawn(e) =>
            {
                write!
                (
                    f,
                    "ThreadPool failed starting threads to replace panicked \\
                    threads: {}",
                    e
                )
            }
        }
    }
}

impl Error for StartThreadsError {}

impl PartialEq for StartThreadsError
{
    fn eq( &self, other: &Self ) -> bool
    {
        match (self, other)
        {
            (StartThreadsError::NoThreads(a), StartThreadsError::NoThreads(b))
            | (StartThreadsError::Respawn(a), StartThreadsError::Respawn(b))
            => err_eq(a, b),
            _ => false,
        }
    }
}


//------------------------------------------------------------------------------
//  NewThreadPoolError
//------------------------------------------------------------------------------
#[derive(Debug)]
pub enum NewThreadPoolError
{
    Parameter(String),

    //  `std::thread::Builder::spawn` returned the included error.
    Spawn(std::io::Error),
}

impl Display for NewThreadPoolError
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            NewThreadPoolError::Parameter(s) => write!(f, "{}", s),
            NewThreadPoolError::Spawn(e) =>
            {
                write!(f, "ThreadPool failed starting threads: {}", e)
            },
        }
    }
}

impl Error for NewThreadPoolError {}

impl PartialEq for NewThreadPoolError
{
    fn eq( &self, other: &Self ) -> bool
    {
        match (self, other)
        {
            (NewThreadPoolError::Parameter(a), NewThreadPoolError::Parameter(b))
            => a == b,
            (NewThreadPoolError::Spawn(a), NewThreadPoolError::Spawn(b))
            => err_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for NewThreadPoolError {}


//------------------------------------------------------------------------------
//  TryScheduleError
//------------------------------------------------------------------------------
#[derive(Debug)]
pub enum TryScheduleError
{
    QueueFull,

    //  The pool has no threads and `std::thread::Builder::spawn` returned the
    //  included error.
    NoThreads(std::io::Error),

    //  The pool has at least one thread and `std::thread::Builder::spawn`
    //  returned the included error.
    Respawn(std::io::Error),
}

impl Display for TryScheduleError
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            TryScheduleError::QueueFull => write!(f, "ThreadPool queue is full"),
            TryScheduleError::NoThreads(e) =>
            {
                write!
                (
                    f,
                    "ThreadPool workers all panicked, failed starting \\
                    replacement threads: {}",
                    e
                )
            },
            TryScheduleError::Respawn(e) =>
            {
                write!
                (
                    f,
                    "ThreadPool failed starting threads to replace panicked \\
                    threads: {}",
                    e
                )
            },
        }
    }
}

impl Error for TryScheduleError {}

impl PartialEq for TryScheduleError
{
    fn eq( &self, other: &Self ) -> bool
    {
        match (self, other)
        {
            (TryScheduleError::QueueFull, TryScheduleError::QueueFull) => true,
            (TryScheduleError::NoThreads(a), TryScheduleError::NoThreads(b))
            | (TryScheduleError::Respawn(a), TryScheduleError::Respawn(b))
            => err_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for TryScheduleError {}


//------------------------------------------------------------------------------
//  Error conversion
//------------------------------------------------------------------------------
impl From<StartThreadsError> for NewThreadPoolError
{
    fn from( err: StartThreadsError ) -> Self
    {
        match err
        {
            StartThreadsError::NoThreads(e) | StartThreadsError::Respawn(e) =>
            {
                NewThreadPoolError::Spawn(e)
            }
        }
    }
}

impl From<NewThreadPoolError> for std::io::Error
{
    fn from( new_thread_pool_error: NewThreadPoolError ) -> Self
    {
        match new_thread_pool_error
        {
            NewThreadPoolError::Parameter(s) =>
            {
                std::io::Error::new(ErrorKind::InvalidInput, s)
            },
            NewThreadPoolError::Spawn(s) =>
            {
                std::io::Error::new
                (
                    ErrorKind::Other,
                    format!("failed to start threads: {}", s)
                )
            },
        }
    }
}

impl From<StartThreadsError> for TryScheduleError
{
    fn from( err: StartThreadsError ) -> Self
    {
        match err
        {
            StartThreadsError::NoThreads(e) => TryScheduleError::NoThreads(e),
            StartThreadsError::Respawn(e) => TryScheduleError::Respawn(e),
        }
    }
}

impl From<TryScheduleError> for std::io::Error
{
    fn from( try_schedule_error: TryScheduleError ) -> Self
    {
        match try_schedule_error
        {
            TryScheduleError::QueueFull =>
            {
                std::io::Error::new
                (
                    ErrorKind::WouldBlock,
                    "TryScheduleError::QueueFull"
                )
            },
            TryScheduleError::NoThreads(e) =>
            {
                std::io::Error::new
                (
                    e.kind(),
                    format!
                    (
                        "ThreadPool workers all panicked, failed starting \\
                        replacement threads: {}",
                        e
                    )
                )
            },
            TryScheduleError::Respawn(e) =>
            {
                std::io::Error::new
                (
                    e.kind(),
                    format!
                    (
                        "ThreadPool failed starting threads to replace \\
                        panicked threads: {}",
                        e
                    )
                )
            },
        }
    }
}
