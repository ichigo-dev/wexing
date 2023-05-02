use super::TaskQueue;

pub(crate) struct LocalTaskQueue
{
    task_queue: TaskQueue,
}

impl LocalTaskQueue
{
    //--------------------------------------------------------------------------
    //  Creates a local task queue.
    //--------------------------------------------------------------------------
    pub fn new() -> Self
    {
        Self
        {
            task_queue: TaskQueue::new(),
        }
    }
}
