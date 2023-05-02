use std::sync::atomic::{ AtomicUsize, Ordering };


//------------------------------------------------------------------------------
//  Thread-safe counter that cannot be interrupted by other processes.
//------------------------------------------------------------------------------
pub struct AtomicCounter
{
    next_value: AtomicUsize,
}

impl AtomicCounter
{
    //--------------------------------------------------------------------------
    //  Creates an atomic counter.
    //--------------------------------------------------------------------------
    pub fn new() -> Self
    {
        Self
        {
            next_value: AtomicUsize::new(0),
        }
    }

    //--------------------------------------------------------------------------
    //  Counts up counter.
    //--------------------------------------------------------------------------
    pub fn next( &self ) -> usize
    {
        self.next_value.fetch_add(1, Ordering::AcqRel)
    }
}


//------------------------------------------------------------------------------
//  Tests
//------------------------------------------------------------------------------
#[cfg(test)]
mod tests
{
    use crate::util::AtomicCounter;
    use std::sync::{ mpsc, Arc };

    #[test]
    fn atomic_counter()
    {
        let counter = Arc::new(AtomicCounter::new());
        assert_eq!(0, counter.next());
        assert_eq!(1, counter.next());
        assert_eq!(2, counter.next());
    }

    #[test]
    fn atomic_counter_many_readers()
    {
        let receiver =
        {
            let counter = Arc::new(AtomicCounter::new());
            let (sender, receiver) = mpsc::channel();

            for _ in 0..10
            {
                let counter_clone = counter.clone();
                let sender_clone = sender.clone();

                std::thread::spawn(move ||
                {
                    for _ in 0..10
                    {
                        sender_clone.send(counter_clone.next()).unwrap();
                    }
                });
            }
            receiver
        };

        let mut values: Vec<usize> = receiver.iter().collect();
        values.sort_unstable();
        assert_eq!((0_usize..100).collect::<Vec<usize>>(), values);
    }
}
