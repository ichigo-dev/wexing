pub struct Task
{
    inner: Box<dyn FnOnce() + Send + Sync>,
    priority: usize,
}

impl Task
{
    //--------------------------------------------------------------------------
    //  Creates a task.
    //--------------------------------------------------------------------------
    pub fn new( f: Box<dyn FnOnce() + Send + Sync>, priority: usize ) -> Self
    {
        Self
        {
            inner: f,
            priority,
        }
    }

    //--------------------------------------------------------------------------
    //  Executes this task.
    //--------------------------------------------------------------------------
    pub fn execute( self )
    {
        (self.inner)();
    }
}
