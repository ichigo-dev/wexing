/*

    ThreadPool

*/

mod inner;

mod task;
pub use task::Task;

mod task_queue;
pub(crate) use task_queue::TaskQueue;

mod global_task_queue;
pub(crate) use global_task_queue::GlobalTaskQueue;

mod local_task_queue;
pub(crate) use local_task_queue::LocalTaskQueue;

mod worker;
pub(crate) use worker::Worker;

mod stealer;
pub(crate) use stealer::Stealer;

use inner::Inner;

pub struct ThreadPool
{
    inner: Inner,
}

impl ThreadPool
{
    //--------------------------------------------------------------------------
    //  Creates a threadpool.
    //--------------------------------------------------------------------------
    pub fn new( name: &'static str, size: usize ) -> Self
    {
        Self
        {
            inner: Inner::new(name, size),
        }
    }

    //--------------------------------------------------------------------------
    //  Runs threadpool.
    //--------------------------------------------------------------------------
    pub fn run( &self ) -> Result<(), std::io::Error>
    {
        self.inner.start_threads()
    }

    //--------------------------------------------------------------------------
    //  Schedules a task.
    //--------------------------------------------------------------------------
    pub fn schedule( &self, task: Task )
    {
        self.inner.schedule(task);
    }

    //--------------------------------------------------------------------------
    //  Returns the number of live thread.
    //--------------------------------------------------------------------------
    pub fn num_live_threads( &self ) -> usize
    {
        self.inner.num_live_threads()
    }
}

#[cfg(test)]
mod test
{
    use super::{ Task, ThreadPool };

    #[test]
    fn threadpool()
    {
        let tp = ThreadPool::new("wexing", 10);
        tp.run().unwrap();
        assert_eq!(tp.num_live_threads(), 10);

        let task = Task::new(Box::new(|| { println!("do task"); }), 0);
        tp.schedule(task);
    }
}
