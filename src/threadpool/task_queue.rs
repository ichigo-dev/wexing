use super::Task;

pub(crate) struct TaskQueue
{
    queue: Vec<Task>,
}

impl TaskQueue
{
    //--------------------------------------------------------------------------
    //  Creates a task queue.
    //--------------------------------------------------------------------------
    pub fn new() -> Self
    {
        Self
        {
            queue: Vec::new(),
        }
    }
}
