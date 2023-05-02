use crate::task::Task;
use crate::worker::Worker;
use crate::queue::{ self, Sender, Receiver };

pub(crate) struct ThreadPool
{
    thread_suffix: &'static str,
    size: usize,
    sender: Sender<Task>,
    receiver: Receiver<Task>,
}

impl ThreadPool
{
    pub(crate) fn new( size: usize, thread_suffix: &'static str ) -> Self
    {
        let (sender, receiver) = queue::channel();
        Self
        {
            thread_suffix,
            size,
            sender,
            receiver,
        }
    }

    pub(crate) fn start_threads( &self ) -> Result<(), std::io::Error>
    {
        for _ in 0..self.size
        {
            self.start_thread()?;
        }
        Ok(())
    }

    fn start_thread( &self ) -> Result<(), std::io::Error>
    {
        let worker = Worker::new(self.receiver.clone());
        std::thread::Builder::new()
            .name(format!("{}-{}", self.thread_suffix, self.num_live_thread()))
            .spawn(move || worker.work())?;

        Ok(())
    }

    fn num_live_thread( &self ) -> usize
    {
        self.receiver.count()
    }
}
