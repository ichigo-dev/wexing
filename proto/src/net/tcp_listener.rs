/*

    Asynchronous support for standard library TcpListener.

*/

use super::{ sleep, TcpStream };
use std::io::ErrorKind;
use std::net::{ SocketAddr, ToSocketAddrs };


//------------------------------------------------------------------------------
//  `std::net::TcpListener` wrapper with support for asynchronous accept.
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct TcpListener
{
    std_listener: std::net::TcpListener,
}

impl TcpListener
{
    //--------------------------------------------------------------------------
    //  Wraps an existing listener socket.
    //--------------------------------------------------------------------------
    pub fn new
    (
        std_listener: std::net::TcpListener,
    ) -> Result<Self, std::io::Error>
    {
        std_listener.set_nonblocking(true)?;
        crate::timer::start_timer_thread();
        Ok(Self { std_listener })
    }

    //--------------------------------------------------------------------------
    //  Returns a TCP listener socket, bound to `addr` , that is ready to
    //  accept connections.
    //--------------------------------------------------------------------------
    pub fn bind<A: ToSocketAddrs>
    (
        addr: A,
    ) -> Result<Self, std::io::Error>
    {
        Self::new(std::net::TcpListener::bind(addr)?)
    }

    //--------------------------------------------------------------------------
    //  Borrows the inner struct.
    //--------------------------------------------------------------------------
    #[must_use]
    pub fn inner( &self ) -> &std::net::TcpListener
    {
        &self.std_listener
    }

    //--------------------------------------------------------------------------
    //  Converts to the inner struct.
    //--------------------------------------------------------------------------
    #[must_use]
    pub fn into_inner( self ) -> std::net::TcpListener
    {
        self.std_listener
    }

    //--------------------------------------------------------------------------
    //  Makes a new handle to this socket.
    //--------------------------------------------------------------------------
    pub fn try_clone( &self ) -> Result<TcpListener, std::io::Error>
    {
        Ok(Self
        {
            std_listener: self.std_listener.try_clone()?,
        })
    }

    //--------------------------------------------------------------------------
    //  Waits for a new connection and then accepts it. Returns the address of
    //  the remote side of the connection and a stream for reading and writing
    //  the connection.
    //--------------------------------------------------------------------------
    pub async fn accept( &self )
        -> Result<(TcpStream, SocketAddr), std::io::Error>
    {
        loop
        {
            match self.std_listener.accept()
            {
                Ok((std_stream, addr)) =>
                {
                    return Ok((TcpStream::new(std_stream)?, addr));
                },
                Err(e) if e.kind() == ErrorKind::WouldBlock =>
                {
                    sleep().await;
                },
                Err(e) => return Err(e),
            }
        }
    }
}
