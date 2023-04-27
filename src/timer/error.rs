/*

    Errors for timer.

*/

use std::error::Error;
use core::fmt::{ Display, Formatter };


//------------------------------------------------------------------------------
//  TimerThreadNotStarted
//------------------------------------------------------------------------------
#[derive(Debug, Eq, PartialEq)]
pub struct TimerThreadNotStarted {}

impl Display for TimerThreadNotStarted
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Error for TimerThreadNotStarted {}


//------------------------------------------------------------------------------
//  DeadlineExceeded
//------------------------------------------------------------------------------
#[derive(Debug, PartialEq)]
pub struct DeadlineExceeded;

impl From<DeadlineExceeded> for std::io::Error
{
    fn from( _error: DeadlineExceeded ) -> Self
    {
        std::io::Error::new(std::io::ErrorKind::TimedOut, "DeadlineExceeded")
    }
}

impl Display for DeadlineExceeded
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Error for DeadlineExceeded {}


//------------------------------------------------------------------------------
//  DeadlineError
//------------------------------------------------------------------------------
#[derive(Debug, PartialEq)]
pub enum DeadlineError
{
    TimerThreadNotStarted,
    DeadlineExceeded,
}

impl From<DeadlineError> for std::io::Error
{
    fn from( error: DeadlineError ) -> Self
    {
        match error
        {
            DeadlineError::TimerThreadNotStarted =>
            {
                std::io::Error::new
                (
                    std::io::ErrorKind::Other,
                    "TimerThreadNotStarted"
                )
            },
            DeadlineError::DeadlineExceeded =>
            {
                std::io::Error::new
                (
                    std::io::ErrorKind::TimedOut,
                    "DeadlineExceeded"
                )
            },
        }
    }
}

impl Display for DeadlineError
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Error for DeadlineError {}
