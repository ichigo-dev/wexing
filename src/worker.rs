use crate::task::Task;
use crate::queue::Receiver;

pub(crate) struct Worker
{
    queue: Vec<Task>,
    receiver: Receiver<Task>,
}

impl Worker
{
    pub(crate) fn new( receiver: Receiver<Task> ) -> Self
    {
        Self
        {
            queue: Vec::new(),
            receiver,
        }
    }

    pub(crate) fn work( &self )
    {
        loop
        {
            match self.receiver.recv()
            {
                Some(task) => task.run(),
                None =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                },
            }
        }
    }
}
