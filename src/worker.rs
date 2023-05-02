use crate::task::Task;
use crate::queue::{ Sender, Receiver };

pub(crate) struct Worker
{
    queue: Vec<Task>,
    sender: Sender<Task>,
    receiver: Receiver<Task>,
}

impl Worker
{
    pub(crate) fn new( sender: Sender<Task>, receiver: Receiver<Task> ) -> Self
    {
        Self
        {
            queue: Vec::new(),
            sender,
            receiver,
        }
    }

    pub(crate) fn work( &self )
    {
        loop
        {
            match self.receiver.recv()
            {
                Some(task) =>
                {
                    println!("{:?}", std::thread::current().name());
                    task.run(self.sender.clone())
                },
                None =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                },
            }
        }
    }
}
