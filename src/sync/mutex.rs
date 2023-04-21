/*

    Async Mutex

*/

use core::future::Future;
use core::ops::{ Deref, DerefMut };
use core::pin::Pin;
use core::task::{ Context, Poll };
use std::collections::VecDeque;
use std::sync::TryLockError;
use std::task::Waker;


//------------------------------------------------------------------------------
//  Data for internal processing of `Mutex`.
//
//  If another task tries to lock an already locked `Mutex`, it is addes to the
//  internal wakers and `wake()` is called when the lock is released.
//------------------------------------------------------------------------------
struct Inner
{
    wakers: VecDeque<Waker>,
    locked: bool,
}


//------------------------------------------------------------------------------
//  `std::sync::Mutex` wrapper with support for asynchronous lock.
//------------------------------------------------------------------------------
pub struct Mutex<T>
{
    inner: std::sync::Mutex<Inner>,
    value: std::sync::Mutex<T>,
}

impl<T> Mutex<T>
{
    //--------------------------------------------------------------------------
    //  Create a new `Mutex`.
    //--------------------------------------------------------------------------
    pub fn new( value: T ) -> Mutex<T>
    {
        Self
        {
            inner: std::sync::Mutex::new
            (
                Inner
                {
                    wakers: VecDeque::new(),
                    locked: false,
                }
            ),
            value: std::sync::Mutex::new(value),
        }
    }

    //--------------------------------------------------------------------------
    //  Lock the value and get `MutexGuard`.
    //--------------------------------------------------------------------------
    pub async fn lock( &self ) -> MutexGuard<'_, T>
    {
        LockFuture { mutex: self }.await
    }
}


//------------------------------------------------------------------------------
//  An RAII implementation of a "scoped lock" of a mutex. When this structure is
//  dropped (falls out of scope), the lock will be unlocked.
//
//  The data protected by the mutex can be accessed through this guard via its
//  `Deref` and `DerefMut` implementations.
//------------------------------------------------------------------------------
pub struct MutexGuard<'a, T>
{
    mutex: &'a Mutex<T>,
    value_guard: Option<std::sync::MutexGuard<'a, T>>,
}

impl<'a, T> MutexGuard<'a, T>
{
    //--------------------------------------------------------------------------
    //  Create a new `MutexGuard`.
    //--------------------------------------------------------------------------
    fn new( mutex: &'a Mutex<T>, value_guard: std::sync::MutexGuard<'a, T> )
        -> MutexGuard<'a, T>
    {
        let mut inner_guard = mutex.inner.lock().unwrap();
        assert!(!inner_guard.locked);
        inner_guard.locked = true;
        MutexGuard
        {
            mutex,
            value_guard: Some(value_guard),
        }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T>
{
    //--------------------------------------------------------------------------
    //  When `MutexGuard` is dropped, call `wake()` on any other tasks that
    //  tried to get the lock.
    //--------------------------------------------------------------------------
    fn drop( &mut self )
    {
        let mut wakers = VecDeque::new();
        {
            let mut inner_guard = self.mutex.inner.lock().unwrap();
            assert!(inner_guard.locked);
            inner_guard.locked = false;
            std::mem::swap(&mut inner_guard.wakers, &mut wakers);
        }
        self.value_guard.take();
        for waker in wakers
        {
            waker.wake();
        }
    }
}

impl<'a, T> Deref for MutexGuard<'a, T>
{
    type Target = T;

    //--------------------------------------------------------------------------
    //  Access the inner value.
    //--------------------------------------------------------------------------
    fn deref( &self ) -> &Self::Target
    {
        &*self.value_guard.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T>
{
    //--------------------------------------------------------------------------
    //  Access the inner value.
    //--------------------------------------------------------------------------
    fn deref_mut( &mut self ) -> &mut Self::Target
    {
        &mut *self.value_guard.as_mut().unwrap()
    }
}


//------------------------------------------------------------------------------
//  Future to create `MutexGuard`.
//------------------------------------------------------------------------------
pub struct LockFuture<'a, T>
{
    mutex: &'a Mutex<T>,
}

impl<'a, T> Future for LockFuture<'a, T>
{
    type Output = MutexGuard<'a, T>;

    //--------------------------------------------------------------------------
    //  Attempt to acquire `MutexGuard` and re-polling if the value is already
    //  locked.
    //--------------------------------------------------------------------------
    fn poll( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Self::Output>
    {
        loop
        {
            match self.mutex.value.try_lock()
            {
                Ok(guard) =>
                {
                    return Poll::Ready(MutexGuard::new(self.mutex, guard));
                },
                Err(TryLockError::Poisoned(e)) => panic!("{}", e),
                Err(TryLockError::WouldBlock) => {},
            }

            //  If already locked, register a waker for this task and `wake()`
            //  when unlocked.
            let mut guard = self.mutex.inner.lock().unwrap();
            if guard.locked == true
            {
                guard.wakers.push_back(cx.waker().clone());
                return Poll::Pending;
            }
        }
    }
}
