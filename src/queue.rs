use std::collections::BinaryHeap;
use std::sync::{ Arc, Mutex };
use std::sync::atomic::{ AtomicUsize, Ordering };


pub(crate) fn channel<T: std::cmp::Ord>() -> (Sender<T>, Receiver<T>)
{
    let inner = Inner
    {
        queue: Mutex::new(BinaryHeap::new()),
        cnt_sender: AtomicUsize::new(0),
        cnt_receiver: AtomicUsize::new(0),
    };
    let shared_inner = Arc::new(inner);

    (
        Sender { inner: shared_inner.clone() },
        Receiver { inner: shared_inner.clone() },
    )
}

struct Inner<T>
{
    queue: Mutex<BinaryHeap<T>>,
    cnt_sender: AtomicUsize,
    cnt_receiver: AtomicUsize,
}

//------------------------------------------------------------------------------
//  Sender
//------------------------------------------------------------------------------
pub(crate) struct Sender<T>
{
    inner: Arc<Inner<T>>,
}

impl<T: std::cmp::Ord> Sender<T>
{
    pub(crate) fn send( &self, item: T )
    {
        let mut queue = self.inner.queue.lock().unwrap();
        queue.push(item);
    }

    pub(crate) fn count( &self ) -> usize
    {
        self.inner.cnt_sender.load(Ordering::Relaxed)
    }
}

impl<T> Clone for Sender<T>
{
    fn clone( &self ) -> Self
    {
        self.inner.cnt_sender.fetch_add(1, Ordering::SeqCst);
        Sender
        {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Drop for Sender<T>
{
    fn drop( &mut self )
    {
        self.inner.cnt_sender.fetch_sub(1, Ordering::SeqCst);
    }
}

//------------------------------------------------------------------------------
//  Receiver
//------------------------------------------------------------------------------
pub(crate) struct Receiver<T>
{
    inner: Arc<Inner<T>>,
}

impl<T: std::cmp::Ord> Receiver<T>
{
    pub(crate) fn recv( &self ) -> Option<T>
    {
        let mut queue = self.inner.queue.lock().unwrap();
        queue.pop()
    }

    pub(crate) fn count( &self ) -> usize
    {
        self.inner.cnt_receiver.load(Ordering::Relaxed)
    }
}

impl<T> Clone for Receiver<T>
{
    fn clone( &self ) -> Self
    {
        self.inner.cnt_receiver.fetch_add(1, Ordering::SeqCst);
        Receiver
        {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Drop for Receiver<T>
{
    fn drop( &mut self )
    {
        self.inner.cnt_receiver.fetch_sub(1, Ordering::SeqCst);
    }
}
