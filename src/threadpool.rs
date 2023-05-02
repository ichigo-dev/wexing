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

    pub(crate) fn spawn( &self, f: impl FnOnce() + Send + 'static )
    {
        let task = Task::new(Box::new(f), 0);
        self.sender.send(task);
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
        let now = match std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            {
                Ok(s) => s.as_secs(),
                Err(_) => 0,
            };

        let thread_name = format!
        (
            "{}-no{}-{}",
            self.thread_suffix,
            self.num_live_thread() + 1,
            now
        );

        let worker = Worker::new(self.receiver.clone());
        std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || worker.work())?;

        Ok(())
    }

    fn num_live_thread( &self ) -> usize
    {
        self.receiver.count()
    }
}
