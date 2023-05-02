use super::{ Task, GlobalTaskQueue, Worker, Stealer };
use crate::util::AtomicCounter;

use std::sync::mpsc::{ self, Receiver, Sender };
use std::sync::{ Arc, Mutex };

pub(crate) struct Inner
{
    name: &'static str,
    name_thread_cnt: AtomicCounter,
    size: usize,
    global_queue: GlobalTaskQueue,
    stealer: Stealer,
    receiver: Arc<Mutex<Receiver<Task>>>,
    sender: Sender<Task>,
}

impl Inner
{
    //--------------------------------------------------------------------------
    //  Creates a inner.
    //--------------------------------------------------------------------------
    pub fn new( name: &'static str, size: usize ) -> Self
    {
        let (sender, receiver) = mpsc::channel::<Task>();
        Self
        {
            name,
            name_thread_cnt: AtomicCounter::new(),
            size,
            global_queue: GlobalTaskQueue::new(),
            stealer: Stealer::new(),
            receiver: Arc::new(Mutex::new(receiver)),
            sender,
        }
    }

    //--------------------------------------------------------------------------
    //  Starts threads.
    //--------------------------------------------------------------------------
    pub fn start_threads( &self ) -> Result<(), std::io::Error>
    {
        while self.size > self.num_live_threads()
        {
            self.start_thread()?;
        }

        Ok(())
    }

    //--------------------------------------------------------------------------
    //  Start a worker thread.
    //--------------------------------------------------------------------------
    pub fn start_thread( &self ) -> Result<(), std::io::Error>
    {
        let receiver_clone = self.receiver.clone();
        let worker = Arc::new(Worker::new(receiver_clone));
        let thread_name = format!
        (
            "{}-{}",
            self.name,
            self.name_thread_cnt.next()
        );

        let worker_clone = worker.clone();
        std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || worker_clone.work())?;

        Ok(())
    }

    //--------------------------------------------------------------------------
    //  Returns the number of live thread.
    //--------------------------------------------------------------------------
    pub fn num_live_threads( &self ) -> usize
    {
        Arc::strong_count(&self.receiver)
    }

    //--------------------------------------------------------------------------
    //  Schedules a task.
    //--------------------------------------------------------------------------
    pub fn schedule( &self, task: Task )
    {
        let sender_clone = self.sender.clone();
        sender_clone.send(task).unwrap();
    }
}
