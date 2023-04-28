/*

    Select the earliest completed Future among multiple Futures.

*/

mod options;
pub use options::*;

use core::future::Future;
use core::pin::Pin;
use core::task::{ Context, Poll };


//------------------------------------------------------------------------------
//  A dummy Future that always returns `Poll::Pending` .
//------------------------------------------------------------------------------
struct PendingFuture;

impl Unpin for PendingFuture {}

impl Future for PendingFuture
{
    type Output = ();

    //--------------------------------------------------------------------------
    //  Always returns `Poll::Pending` , so this Future will never be selected.
    //--------------------------------------------------------------------------
    fn poll( self: Pin<&mut Self>, _cx: &mut Context<'_> ) -> Poll<Self::Output>
    {
        Poll::Pending
    }
}


//------------------------------------------------------------------------------
//  A Future that determines which of the `Future` it contains reaches
//  `Poll::Ready` first.
//------------------------------------------------------------------------------
pub struct SelectFuture<A, B, C, D, E, FutA, FutB, FutC, FutD, FutE>
where
    FutA: Future<Output = A> + Send + Unpin + 'static,
    FutB: Future<Output = B> + Send + Unpin + 'static,
    FutC: Future<Output = C> + Send + Unpin + 'static,
    FutD: Future<Output = D> + Send + Unpin + 'static,
    FutE: Future<Output = E> + Send + Unpin + 'static,
{
    a: FutA,
    b: FutB,
    c: FutC,
    d: FutD,
    e: FutE,
}

impl<A, B, C, D, E, FutA, FutB, FutC, FutD, FutE>
    SelectFuture<A, B, C, D, E, FutA, FutB, FutC, FutD, FutE>
where
    FutA: Future<Output = A> + Send + Unpin + 'static,
    FutB: Future<Output = B> + Send + Unpin + 'static,
    FutC: Future<Output = C> + Send + Unpin + 'static,
    FutD: Future<Output = D> + Send + Unpin + 'static,
    FutE: Future<Output = E> + Send + Unpin + 'static,
{
    //--------------------------------------------------------------------------
    //  Creates a SelectFuture.
    //--------------------------------------------------------------------------
    pub fn new
    (
        a: FutA,
        b: FutB,
        c: FutC,
        d: FutD,
        e: FutE,
    ) -> SelectFuture<A, B, C, D, E, FutA, FutB, FutC, FutD, FutE>
    {
        SelectFuture { a, b, c, d, e }
    }
}

impl<A, B, C, D, E, FutA, FutB, FutC, FutD, FutE> Future
    for SelectFuture<A, B, C, D, E, FutA, FutB, FutC, FutD, FutE>
where
    FutA: Future<Output = A> + Send + Unpin + 'static,
    FutB: Future<Output = B> + Send + Unpin + 'static,
    FutC: Future<Output = C> + Send + Unpin + 'static,
    FutD: Future<Output = D> + Send + Unpin + 'static,
    FutE: Future<Output = E> + Send + Unpin + 'static,
{
    type Output = OptionAbcde<A, B, C, D, E>;

    //--------------------------------------------------------------------------
    //  If there is already a `Poll::Ready` Future, returns it, otherwise
    //  returns `Poll::Pending` .
    //--------------------------------------------------------------------------
    fn poll( self: Pin<&mut Self>, cx: &mut Context<'_> ) -> Poll<Self::Output>
    {
        let mut_self = self.get_mut();

        match Pin::new(&mut mut_self.a).poll(cx)
        {
            Poll::Ready(value) => return Poll::Ready(OptionAbcde::A(value)),
            Poll::Pending => {}
        }
        match Pin::new(&mut mut_self.b).poll(cx)
        {
            Poll::Ready(value) => return Poll::Ready(OptionAbcde::B(value)),
            Poll::Pending => {}
        }
        match Pin::new(&mut mut_self.c).poll(cx)
        {
            Poll::Ready(value) => return Poll::Ready(OptionAbcde::C(value)),
            Poll::Pending => {}
        }
        match Pin::new(&mut mut_self.d).poll(cx)
        {
            Poll::Ready(value) => return Poll::Ready(OptionAbcde::D(value)),
            Poll::Pending => {}
        }
        match Pin::new(&mut mut_self.e).poll(cx)
        {
            Poll::Ready(value) => return Poll::Ready(OptionAbcde::E(value)),
            Poll::Pending => {}
        }
        Poll::Pending
    }
}


//------------------------------------------------------------------------------
//  Awaits both futures and returns the value from the one that compoletes
//  first.
//------------------------------------------------------------------------------
pub async fn select_ab<A, B, FutA, FutB>( a: FutA, b: FutB ) -> OptionAb<A, B>
where
    FutA: Future<Output = A> + Send + Unpin + 'static,
    FutB: Future<Output = B> + Send + Unpin + 'static,
{
    match SelectFuture::new
    (
        Box::pin(a),
        Box::pin(b),
        PendingFuture {},
        PendingFuture {},
        PendingFuture {},
    )
    .await
    {
        OptionAbcde::A(value) => OptionAb::A(value),
        OptionAbcde::B(value) => OptionAb::B(value),
        _ => unreachable!(),
    }
}


//------------------------------------------------------------------------------
//  Awaits the futures and returns the value from the one that compoletes first.
//------------------------------------------------------------------------------
pub async fn select_abc<A, B, C, FutA, FutB, FutC>
(
    a: FutA,
    b: FutB,
    c: FutC,
) -> OptionAbc<A, B, C>
where
    FutA: Future<Output = A> + Send + Unpin + 'static,
    FutB: Future<Output = B> + Send + Unpin + 'static,
    FutC: Future<Output = C> + Send + Unpin + 'static,
{
    match SelectFuture::new
    (
        Box::pin(a),
        Box::pin(b),
        Box::pin(c),
        PendingFuture {},
        PendingFuture {},
    )
    .await
    {
        OptionAbcde::A(value) => OptionAbc::A(value),
        OptionAbcde::B(value) => OptionAbc::B(value),
        OptionAbcde::C(value) => OptionAbc::C(value),
        _ => unreachable!(),
    }
}


//------------------------------------------------------------------------------
//  Awaits the futures and returns the value from the one that compoletes first.
//------------------------------------------------------------------------------
pub async fn select_abcd<A, B, C, D, FutA, FutB, FutC, FutD>
(
    a: FutA,
    b: FutB,
    c: FutC,
    d: FutD,
) -> OptionAbcd<A, B, C, D>
where
    FutA: Future<Output = A> + Send + Unpin + 'static,
    FutB: Future<Output = B> + Send + Unpin + 'static,
    FutC: Future<Output = C> + Send + Unpin + 'static,
    FutD: Future<Output = D> + Send + Unpin + 'static,
{
    match SelectFuture::new
    (
        Box::pin(a),
        Box::pin(b),
        Box::pin(c),
        Box::pin(d),
        PendingFuture {},
    )
    .await
    {
        OptionAbcde::A(value) => OptionAbcd::A(value),
        OptionAbcde::B(value) => OptionAbcd::B(value),
        OptionAbcde::C(value) => OptionAbcd::C(value),
        OptionAbcde::D(value) => OptionAbcd::D(value),
        OptionAbcde::E(_) => unreachable!(),
    }
}


//------------------------------------------------------------------------------
//  Awaits the futures and returns the value from the one that compoletes first.
//------------------------------------------------------------------------------
pub async fn select_abcde<A, B, C, D, E, FutA, FutB, FutC, FutD, FutE>
(
    a: FutA,
    b: FutB,
    c: FutC,
    d: FutD,
    e: FutE,
) -> OptionAbcde<A, B, C, D, E>
where
    FutA: Future<Output = A> + Send + Unpin + 'static,
    FutB: Future<Output = B> + Send + Unpin + 'static,
    FutC: Future<Output = C> + Send + Unpin + 'static,
    FutD: Future<Output = D> + Send + Unpin + 'static,
    FutE: Future<Output = E> + Send + Unpin + 'static,
{
    SelectFuture::new
    (
        Box::pin(a),
        Box::pin(b),
        Box::pin(c),
        Box::pin(d),
        Box::pin(e),
    )
    .await
}
