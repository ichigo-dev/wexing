use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::marker::Unpin;
use std::future::Future;
use std::sync::mpsc::Sender;

pub struct Task
{
    future: Pin<Box<dyn Future<Output = ()> + Send + Unpin + 'static>>,
    task_sender: Sender<Task>,
}

impl Task
{
    //--------------------------------------------------------------------------
    //  Creates a task.
    //--------------------------------------------------------------------------
    pub fn new
    (
        fut: Pin<Box<impl Future<Output = ()> + Send + Unpin + 'static>>,
        task_sender: Sender<Task>,
    ) -> Self
    {
        Self
        {
            future: fut,
            task_sender,
        }
    }

    //--------------------------------------------------------------------------
    //  Polls task.
    //--------------------------------------------------------------------------
    pub fn poll( &mut self ) -> Poll<()>
    {
        let waker = dummy_waker();
        let mut cx = Context::from_waker(&waker);
        Future::poll(self.future.as_mut(), &mut cx)
    }
}

fn dummy_raw_waker() -> RawWaker
{
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker
    {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(std::ptr::null::<()>(), vtable)
}

fn dummy_waker() -> Waker
{
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
