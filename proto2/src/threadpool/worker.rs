use super::{ Task, LocalTaskQueue };

use std::sync::mpsc::Receiver;
use std::sync::{ Arc, Mutex };
use std::time::Duration;

pub(crate) struct Worker
{
    inner: Arc<Inner>,
    receiver: Arc<Mutex<Receiver<Task>>>,
    local_queue: Arc<LocalTaskQueue>,
}

impl Worker
{
    //--------------------------------------------------------------------------
    //  Creates a worker.
    //--------------------------------------------------------------------------
    pub fn new( receiver: Arc<Mutex<Receiver<Task>>> ) -> Self
    {
        Self
        {
            receiver,
            local_queue: Arc::new(LocalTaskQueue::new()),
        }
    }

    //--------------------------------------------------------------------------
    //  The function to be executed in this worker thread.
    //--------------------------------------------------------------------------
    pub(crate) fn work( &self )
    {
        println!()
        /*
        loop
        {
            let recv_result = self
                .receiver
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_millis(500));

            match recv_result
            {
                Ok(f) =>
                {
                }
            }
        }
        */
    }
}
