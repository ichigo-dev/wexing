/*

    Async Channel

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
//  Create a channel that can be used to send a single value. Sender is consumed
//  when it sends a value to the channel.
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
//  Create a synchronous, bounded channel.
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
    //  Consume self by sending a value to a channel and calls `wake()` a task
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
//  SyncSender
//------------------------------------------------------------------------------
#[derive(Clone)]
pub struct SyncSender<T: Send>
{
    sender: Option<std::sync::mpsc::SyncSender<T>>,
    inner: Arc<Mutex<Inner>>,
}

impl<T: Send + Clone> SyncSender<T>
{
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
    fn wake_receiver( &self )
    {
        let receiver_waker = self.inner.lock().unwrap().receiver_waker.take();
        if let Some(waker) = receiver_waker
        {
            waker.wake();
        }
    }

    fn wake_receiver_if_ok<E>( &self, result: Result<(), E> ) -> Result<(), E>
    {
        if result.is_ok()
        {
            self.wake_receiver();
        }
        result
    }

    pub fn send( &self, value: T ) -> Result<(), SendError<T>>
    {
        self.wake_receiver_if_ok(self.sender.as_ref().unwrap().send(value))
    }

    pub fn try_send( &self, value: T ) -> Result<(), TrySendError<T>>
    {
        self.wake_receiver_if_ok(self.sender.as_ref().unwrap().try_send(value))
    }
}

impl<T: Send> Drop for SyncSender<T>
{
    fn drop( &mut self )
    {
        let mut inner_guard = self.inner.lock().unwrap();
        self.sender.take();
        if Arc::strong_count(&self.inner) < 3
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
//  Reciever
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
    fn wake_senders( &self )
    {
        let wakers: Vec<Waker> = std::mem::take(&mut self.inner.lock().unwrap().sender_wakers);
        for waker in wakers
        {
            waker.wake();
        }
    }

    fn wake_senders_if_ok<E>( &self, result: Result<T, E> ) -> Result<T, E>
    {
        if result.is_ok()
        {
            self.wake_senders();
        }
        result
    }

    async fn async_recv( &mut self ) -> Result<T, std::sync::mpsc::RecvError>
    {
        self.await
    }

    pub fn recv( &self ) -> Result<T, std::sync::mpsc::RecvError>
    {
        self.wake_senders_if_ok(self.receiver.as_ref().unwrap().recv())
    }

    pub fn try_recv( &self ) -> Result<T, std::sync::mpsc::TryRecvError>
    {
        self.wake_senders_if_ok(self.receiver.as_ref().unwrap().try_recv())
    }

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

    pub fn iter( &self ) -> Iter<'_, T>
    {
        Iter { rx: self }
    }

    pub fn try_iter( &self ) -> TryIter<'_, T>
    {
        TryIter { rx: self }
    }
}

impl<T: Send> Drop for Receiver<T>
{
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
                let waker = cx.waker().clone();
                if Arc::strong_count(&self.inner) < 2
                {
                    Poll::Ready(Err(RecvError))
                }
                else
                {
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
//  Iterator
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
