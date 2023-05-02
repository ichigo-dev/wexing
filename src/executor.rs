use crate::threadpool::ThreadPool;

struct Executor
{
    pool: ThreadPool,
}

impl Executor
{
    fn new( size: usize ) -> Self
    {
        Self::with_name(size, "wexing")
    }

    fn with_name( size: usize, thread_suffix: &'static str ) -> Self
    {
        Self
        {
            pool: ThreadPool::new(size, thread_suffix),
        }
    }

    fn run( &self )
    {
        let _ = self.pool.start_threads();
    }

    fn spawn( &self, f: impl FnOnce() + Send + 'static )
    {
        self.pool.spawn(f);
    }
}

#[cfg(test)]
mod test
{
    use super::Executor;

    #[test]
    fn test_executor()
    {
        let executor = Executor::new(4);
        executor.run();
        executor.spawn(|| { println!("test"); });
        executor.spawn(|| { println!("test"); });
        executor.spawn(|| { println!("test"); });
        executor.spawn(|| { println!("test"); });
        executor.spawn(|| { println!("test"); });
        executor.spawn(|| { println!("test"); });
        executor.spawn(|| { println!("test"); });
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
