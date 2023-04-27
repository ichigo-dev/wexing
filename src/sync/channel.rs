/*

    Asynchronous support for standard library channel.

*/

use core::future::Future;
use core::pin::Pin;
use core::task::{ Context, Poll };
use std::any::type_name;
use std::cell::Cell;
use std::fmt::{ Debug, Formatter };
use std::sync::mpsc::{ RecvError, SendError, TryRecvError, TrySendError };
use std::sync::{ Arc, Mutex };
use std::task::Waker;


//------------------------------------------------------------------------------
//  Creates a channel that can be used to send a single value. Sender is
//  consumed when it sends a value to the channel.
//------------------------------------------------------------------------------
#[must_use]
pub fn oneshot<T>() -> (OneSender<T>, Receiver<T>)
where
    T: Send
{
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let inner = Arc::new(Mutex::new(Inner
    {
        sender_wakers: Vec::new(),
        receiver_waker: None,
    }));

    (
        OneSender
        {
            sender: Some(sender),
            inner: inner.clone(),
        },
        Receiver
        {
            receiver: Some(receiver),
            inner
        },
    )
}

//------------------------------------------------------------------------------
//  Creates a synchronous, bounded channel.
//------------------------------------------------------------------------------
#[must_use]
pub fn sync_channel<T>( bound: usize ) -> (SyncSender<T>, Receiver<T>)
where
    T: Send
{
    assert!(bound > 0, "bound must be greater than zero");
    let (sender, receiver) = std::sync::mpsc::sync_channel(bound);
    let inner = Arc::new(Mutex::new(Inner
    {
        sender_wakers: Vec::new(),
        receiver_waker: None,
    }));

    (
        SyncSender
        {
            sender: Some(sender),
            inner: inner.clone(),
        },
        Receiver
        {
            receiver: Some(receiver),
            inner,
        }
    )
}


//------------------------------------------------------------------------------
//  Data for internal processing of channel.
//
//  The receiver wakes up when it is ready to receive a value.
//
//  Senders are suspended from sending when the queue is full, and woken when
//  there is space in the queue.
//------------------------------------------------------------------------------
struct Inner
{
    sender_wakers: Vec<Waker>,
    receiver_waker: Option<Waker>,
}


//------------------------------------------------------------------------------
//  A OneSender is consumed by sending a single value to the channel.
//------------------------------------------------------------------------------
pub struct OneSender<T: Send>
{
    sender: Option<std::sync::mpsc::SyncSender<T>>,
    inner: Arc<Mutex<Inner>>,
}

impl<T: Send> OneSender<T>
{
    //--------------------------------------------------------------------------
    //  Consumes self by sending a value to a channel and calls `wake()` a task
    //  waiting for the `Receiver` to receive the value.
    //--------------------------------------------------------------------------
    pub fn send( mut self, value: T ) -> Result<(), SendError<T>>
    {
        self.sender.take().unwrap().send(value)
    }
}

impl<T: Send> Drop for OneSender<T>
{
    //--------------------------------------------------------------------------
    //  When the `OneSender` dropped, it calles `wake()` of a task waiting for
    //  the `Receiver` to receive a value.
    //--------------------------------------------------------------------------
    fn drop( &mut self )
    {
        let mut inner_guard = self.inner.lock().unwrap();
        self.sender.take();
        let receiver_waker = inner_guard.receiver_waker.take();
        drop(inner_guard);
        if let Some(waker) = receiver_waker
        {
            waker.wake();
        }
    }
}

impl<T: Send> PartialEq for OneSender<T>
{
    fn eq( &self, _other: &Self ) -> bool
    {
        false
    }
}

impl<T: Send> Eq for OneSender<T> {}


//------------------------------------------------------------------------------
//  `std::sync::mpsc::SyncSender` wrapper with support for asynchronous send.
//------------------------------------------------------------------------------
#[derive(Clone)]
pub struct SyncSender<T: Send>
{
    sender: Option<std::sync::mpsc::SyncSender<T>>,
    inner: Arc<Mutex<Inner>>,
}

impl<T: Send + Clone> SyncSender<T>
{
    //--------------------------------------------------------------------------
    //  Attempts to send a message and reschedules the task if the channel
    //  queue is full.
    //--------------------------------------------------------------------------
    pub async fn async_send( &self, value: T ) -> Result<(), SendError<T>>
    {
        self.wake_receiver_if_ok
        (
            SendFuture
            {
                sender: self.sender.as_ref().unwrap().clone(),
                inner: self.inner.clone(),
                value: Cell::new(Some(value)),
            }
            .await
        )
    }
}

impl<T: Send> SyncSender<T>
{
    //--------------------------------------------------------------------------
    //  Wakes the receiver.
    //--------------------------------------------------------------------------
    fn wake_receiver( &self )
    {
        let receiver_waker = self.inner.lock().unwrap().receiver_waker.take();
        if let Some(waker) = receiver_waker
        {
            waker.wake();
        }
    }

    //--------------------------------------------------------------------------
    //  Wakes the receiver if the result is `Ok` .
    //--------------------------------------------------------------------------
    fn wake_receiver_if_ok<E>( &self, result: Result<(), E> ) -> Result<(), E>
    {
        if result.is_ok()
        {
            self.wake_receiver();
        }
        result
    }

    //--------------------------------------------------------------------------
    //  Sends a message to the channel queue.
    //--------------------------------------------------------------------------
    pub fn send( &self, value: T ) -> Result<(), SendError<T>>
    {
        self.wake_receiver_if_ok(self.sender.as_ref().unwrap().send(value))
    }

    //--------------------------------------------------------------------------
    //  Attempts to send a message to the channel queue.
    //--------------------------------------------------------------------------
    pub fn try_send( &self, value: T ) -> Result<(), TrySendError<T>>
    {
        self.wake_receiver_if_ok(self.sender.as_ref().unwrap().try_send(value))
    }
}

impl<T: Send> Drop for SyncSender<T>
{
    //--------------------------------------------------------------------------
    //  If only this sender and the receiver refer to `Inner` (This means the
    //  last sender will be dropped), wakes up the receiver.
    //--------------------------------------------------------------------------
    fn drop( &mut self )
    {
        let mut inner_guard = self.inner.lock().unwrap();
        self.sender.take();
        if Arc::strong_count(&self.inner) <= 2
        {
            let receiver_waker = inner_guard.receiver_waker.take();
            drop(inner_guard);
            if let Some(waker) = receiver_waker
            {
                waker.wake();
            }
        }
    }
}

impl<T: Send> Debug for SyncSender<T>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result
    {
        write!(f, "SyncSender<{}>", type_name::<T>())
    }
}

impl<T: Send> PartialEq for SyncSender<T>
{
    fn eq( &self, other: &Self ) -> bool
    {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T:Send> Eq for SyncSender<T> {}


//------------------------------------------------------------------------------
//  Future to create `SyncSender`.
//------------------------------------------------------------------------------
pub struct SendFuture<T: Send>
{
    sender: std::sync::mpsc::SyncSender<T>,
    inner: Arc<Mutex<Inner>>,
    value: Cell<Option<T>>,
}

impl<T: Send> Future for SendFuture<T>
{
    type Output = Result<(), SendError<T>>;

    //--------------------------------------------------------------------------
    //  Attempts to send a message and reschedules if the queue is full.
    //--------------------------------------------------------------------------
    fn poll( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Self::Output>
    {
        let value = self.value.take().unwrap();
        let mut inner_guard = self.inner.lock().unwrap();
        match self.sender.try_send(value)
        {
            Ok(()) => Poll::Ready(Ok(())),
            Err(TrySendError::Disconnected(value)) =>
            {
                Poll::Ready(Err(SendError(value)))
            },
            Err(TrySendError::Full(value)) =>
            {
                self.value.set(Some(value));
                inner_guard.sender_wakers.push(cx.waker().clone());
                Poll::Pending
            },
        }
    }
}


//------------------------------------------------------------------------------
//  `std::sync::mpsc::Receiver` wrapper with support for asynchronous receive.
//------------------------------------------------------------------------------
pub struct Receiver<T>
where
    T: Send,
{
    receiver: Option<std::sync::mpsc::Receiver<T>>,
    inner: Arc<Mutex<Inner>>,
}

impl<T: Send> Receiver<T>
{
    //--------------------------------------------------------------------------
    //  Wakes senders.
    //--------------------------------------------------------------------------
    fn wake_senders( &self )
    {
        let wakers: Vec<Waker> = std::mem::take
        (
            &mut self.inner.lock().unwrap().sender_wakers
        );

        for waker in wakers
        {
            waker.wake();
        }
    }

    //--------------------------------------------------------------------------
    //  Wakes senders if the result is `Ok` .
    //--------------------------------------------------------------------------
    fn wake_senders_if_ok<E>( &self, result: Result<T, E> ) -> Result<T, E>
    {
        if result.is_ok()
        {
            self.wake_senders();
        }
        result
    }

    //--------------------------------------------------------------------------
    //  Attempts to receive a message and reschedules the task if the channel
    //  queue is empty.
    //--------------------------------------------------------------------------
    async fn async_recv( &mut self ) -> Result<T, std::sync::mpsc::RecvError>
    {
        self.await
    }

    //--------------------------------------------------------------------------
    //  Receives a message from the channel queue.
    //--------------------------------------------------------------------------
    pub fn recv( &self ) -> Result<T, std::sync::mpsc::RecvError>
    {
        self.wake_senders_if_ok(self.receiver.as_ref().unwrap().recv())
    }

    //--------------------------------------------------------------------------
    //  Attempts to receive a message to the channel queue.
    //--------------------------------------------------------------------------
    pub fn try_recv( &self ) -> Result<T, std::sync::mpsc::TryRecvError>
    {
        self.wake_senders_if_ok(self.receiver.as_ref().unwrap().try_recv())
    }

    //--------------------------------------------------------------------------
    //  Attempts to wait for a value on this receiver, returning an error if the
    //  corresponding channel has hung up, or if it waits more than timeout.
    //
    //  This function will always block the current thread if threre is no data
    //  available and it's possible for more data to be send (at least one
    //  sender still exists). Once a mesage is sent to the corresponding
    //  `Sender` (or `SyncSender` ), this receiver will wake up and return that
    //  message.
    //
    //  If the corresponding `Sender` has disconnected, or it disconnects while
    //  this call is blocking, this call will wake up and return `Err` to
    //  indicate that no more messages can ever be received on this channel.
    //  However, since channels are buffered, messages sent before the
    //  disconnect will still be properly received.
    //--------------------------------------------------------------------------
    pub fn recv_timeout
    (
        &self,
        timeout: core::time::Duration,
    ) -> Result<T, std::sync::mpsc::RecvTimeoutError>
    {
        self.wake_senders_if_ok
        (
            self.receiver.as_ref().unwrap().recv_timeout(timeout)
        )
    }

    //--------------------------------------------------------------------------
    //  Attempts to wait for a value on this receiver, returning an error if the
    //  corresponding channel has hung up, or if deadline is reached.
    //
    //  This function will always block the current thread if there is no data
    //  available and it’s possible for more data to be sent. Once a message is
    //  sent to the corresponding Sender (or SyncSender), then this receiver
    //  will wake up and return that message.
    //
    //  If the corresponding Sender has disconnected, or it disconnects while
    //  this call is blocking, this call will wake up and return Err to indicate
    //  that no more messages can ever be received on this channel. However,
    //  since channels are buffered, messages sent before the disconnect will
    //  still be properly received.
    //--------------------------------------------------------------------------
    #[cfg(unstable)]
    pub fn recv_deadline
    (
        &self,
        deadline: std::time::Instant,
    ) -> Result<T, std::sync::mpsc::RecvTimeoutError>
    {
        self.wake_senders_if_ok
        (
            self.receiver.as_ref().unwrap().recv_deadline(deadline)
        )
    }

    //--------------------------------------------------------------------------
    //  Creates an iterator that retrieves messages that can be received.
    //--------------------------------------------------------------------------
    pub fn iter( &self ) -> Iter<'_, T>
    {
        Iter { rx: self }
    }

    //--------------------------------------------------------------------------
    //  Attempts to create an iterator that retrieves messages that can be
    //  received.
    //--------------------------------------------------------------------------
    pub fn try_iter( &self ) -> TryIter<'_, T>
    {
        TryIter { rx: self }
    }
}

impl<T: Send> Drop for Receiver<T>
{
    //--------------------------------------------------------------------------
    //  Wakes senders when dropped.
    //--------------------------------------------------------------------------
    fn drop( &mut self )
    {
        let mut inner_guard = self.inner.lock().unwrap();
        self.receiver.take();
        let receiver_waker = inner_guard.receiver_waker.take();
        let sender_wakers = std::mem::take(&mut inner_guard.sender_wakers);
        drop(inner_guard);
        drop(receiver_waker);
        for waker in sender_wakers
        {
            waker.wake();
        }
    }
}

impl<T: Send> Future for Receiver<T>
{
    type Output = Result<T, std::sync::mpsc::RecvError>;

    //--------------------------------------------------------------------------
    //  If it is possible to receive from the channel queue, receives the value
    //  and wakes the sender.
    //
    //  If the channel queue is empty, reschedules the task.
    //--------------------------------------------------------------------------
    fn poll( self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        let mut inner_guard = self.inner.lock().unwrap();
        match self.receiver.as_ref().unwrap().try_recv()
        {
            Ok(value) =>
            {
                drop(inner_guard);
                self.wake_senders();
                Poll::Ready(Ok(value))
            },
            Err(TryRecvError::Disconnected) => Poll::Ready(Err(RecvError)),
            Err(TryRecvError::Empty) =>
            {
                //  Return `Err` If there is no sender already.
                if Arc::strong_count(&self.inner) < 2
                {
                    Poll::Ready(Err(RecvError))
                }
                else
                {
                    let waker = cx.waker().clone();
                    let receiver_waker = inner_guard.receiver_waker.replace(waker);
                    drop(inner_guard);
                    drop(receiver_waker);
                    Poll::Pending
                }
            },
        }
    }
}

impl<T: Send> Debug for Receiver<T>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> std::fmt::Result
    {
        write!(f, "Receiver<{}>", type_name::<T>())
    }
}

impl<T: Send> PartialEq for Receiver<T>
{
    fn eq( &self, _other: &Self ) -> bool
    {
        false
    }
}

impl<T: Send> Eq for Receiver<T> {}

impl<T: Send> IntoIterator for Receiver<T>
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter( self ) -> IntoIter<T>
    {
        IntoIter { rx: self }
    }
}

impl<'a, T: Send> IntoIterator for &'a Receiver<T>
{
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter( self ) -> Iter<'a, T>
    {
        self.iter()
    }
}


//------------------------------------------------------------------------------
//  Iterator related implementation.
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct Iter<'a, T: 'a + Send>
{
    rx: &'a Receiver<T>,
}

impl<'a, T: Send> Iterator for Iter<'a, T>
{
    type Item = T;

    fn next( &mut self ) -> Option<T>
    {
        self.rx.recv().ok()
    }
}

#[derive(Debug)]
pub struct IntoIter<T: Send>
{
    rx: Receiver<T>,
}

impl<T: Send> Iterator for IntoIter<T>
{
    type Item = T;

    fn next( &mut self ) -> Option<T>
    {
        self.rx.recv().ok()
    }
}

pub struct TryIter<'a,T: 'a + Send>
{
    rx: &'a Receiver<T>,
}

impl<'a, T: Send> Iterator for TryIter<'a, T>
{
    type Item = T;

    fn next( &mut self ) -> Option<T>
    {
        self.rx.try_recv().ok()
    }
}
