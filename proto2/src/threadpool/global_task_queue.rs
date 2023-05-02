use super::TaskQueue;

pub(crate) struct GlobalTaskQueue
{
    task_queue: TaskQueue,
}

impl GlobalTaskQueue
{
    //--------------------------------------------------------------------------
    //  Creates a global task queue.
    //--------------------------------------------------------------------------
    pub fn new() -> Self
    {
        Self
        {
            task_queue: TaskQueue::new(),
        }
    }
}
