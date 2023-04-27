/*

    Asynchronous support for standard library TcpStream.

*/

use std::io::{ ErrorKind, Read, Write };
use std::net::ToSocketAddrs;

use super::sleep;


//------------------------------------------------------------------------------
//  `std::net::TcpStream` wrapper with support for asynchronous accept.
//------------------------------------------------------------------------------
pub struct TcpStream
{
    std_stream: std::net::TcpStream,
}

impl TcpStream
{
    //--------------------------------------------------------------------------
    //  Wraps an existing stream.
    //--------------------------------------------------------------------------
    pub fn new
    (
        std_stream: std::net::TcpStream,
    ) -> Result<Self, std::io::Error>
    {
        std_stream.set_nonblocking(true)?;
        crate::timer::start_timer_thread();
        Ok(Self { std_stream })
    }

    //--------------------------------------------------------------------------
    //  Borrows the inner struct.
    //--------------------------------------------------------------------------
    pub fn inner( &self ) -> &std::net::TcpStream
    {
        &self.std_stream
    }

    pub fn inner_mut( &mut self ) -> &mut std::net::TcpStream
    {
        &mut self.std_stream
    }

    //--------------------------------------------------------------------------
    //  Converts to the inner struct.
    //--------------------------------------------------------------------------
    pub fn into_inner( self ) -> std::net::TcpStream
    {
        self.std_stream
    }

    //--------------------------------------------------------------------------
    //  Opens a TCP connection to `addr` .
    //--------------------------------------------------------------------------
    pub async fn connect<A: ToSocketAddrs + Send + 'static>
    (
        addr: A,
    ) -> Result<Self, std::io::Error>
    {
        crate::executor::schedule_blocking(move ||
        {
            TcpStream::new(std::net::TcpStream::connect(addr)?)
        })
        .async_recv()
        .await
        .map_err(|_|
        {
            std::io::Error::new(ErrorKind::Other, "connect thread panicked")
        })?
    }

    //--------------------------------------------------------------------------
    //  Reads some bytes from the socket and places them in `buf` . Returns the
    //  number of bytes read.
    //--------------------------------------------------------------------------
    pub async fn read
    (
        mut self,
        buf: &mut [u8],
    ) -> Result<usize, std::io::Error>
    {
        loop
        {
            match self.std_stream.read(buf)
            {
                Ok(num_read) => return Ok(num_read),
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut => { sleep().await; },
                Err(e) => return Err(e),
            }
        }
    }

    //--------------------------------------------------------------------------
    //  Reads all bytes until the socket is shutdown for reading. Appends the
    //  bytes to `buf` .
    //--------------------------------------------------------------------------
    pub async fn read_to_end
    (
        &mut self,
        buf: &mut Vec<u8>,
    ) -> Result<usize, std::io::Error>
    {
        let mut chunk: [u8; 128 * 1024] = [0; 128 * 1024];
        let mut total_read: usize = 0;
        loop
        {
            match self.std_stream.read(&mut chunk)
            {
                Ok(0) => return Ok(total_read),
                Ok(num_read) =>
                {
                    buf.extend_from_slice(&chunk[..num_read]);
                    total_read += num_read;
                },
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut => { sleep().await; },
                Err(e) if e.kind() == ErrorKind::Interrupted => {},
                Err(e) => return Err(e),
            }
        }
    }

    //--------------------------------------------------------------------------
    //  Reads all bytes until the socket is shutdown for reading. Interprets
    //  the bytes as a single UTF-8 string and appends it to `buf` .
    //--------------------------------------------------------------------------
    pub async fn read_to_string
    (
        &mut self,
        buf: &mut String,
    ) -> Result<usize, std::io::Error>
    {
        let mut bytes = Vec::new();
        self.read_to_end(&mut bytes).await?;
        let num_read = bytes.len();
        let mut result = String::from_utf8(bytes)
            .map_err(|e|
            {
                std::io::Error::new(ErrorKind::InvalidData, format!("{}", e))
            })?;
        core::mem::swap(buf, &mut result);
        Ok(num_read)
    }

    //--------------------------------------------------------------------------
    //  Reads the exact number of bytes required to fill `buf` .
    //--------------------------------------------------------------------------
    pub async fn read_exact
    (
        &mut self,
        buf: &mut [u8],
    ) -> Result<(), std::io::Error>
    {
        let mut dest = buf;
        while !dest.is_empty()
        {
            match self.std_stream.read(dest)
            {
                Ok(0) =>
                {
                    return Err
                    (
                        std::io::Error::new(ErrorKind::UnexpectedEof, "eof")
                    );
                },
                Ok(num_read) => { dest = &mut dest[num_read..]; },
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut => { sleep().await },
                Err(e) if e.kind() == ErrorKind::Interrupted => {},
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    //--------------------------------------------------------------------------
    //  Reads bytes into `bufs` , filling each buffer in order. The final
    //  buffer written to may be partially filled.
    //
    //  Returns the total number of bytes read.
    //--------------------------------------------------------------------------
    pub async fn read_vectored
    (
        &mut self,
        bufs: &mut [std::io::IoSliceMut<'_>],
    ) -> Result<usize, std::io::Error>
    {
        loop
        {
            match self.std_stream.read_vectored(bufs)
            {
                Ok(num_read) => return Ok(num_read),
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut => { sleep().await; },
                Err(e) => return Err(e),
            }
        }
    }

    //--------------------------------------------------------------------------
    //  Waits to receive some data on the socket, then copies it into `buf` .
    //
    //  Returns the number of bytes copied.
    //--------------------------------------------------------------------------
    pub async fn peek
    (
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, std::io::Error>
    {
        loop
        {
            match self.std_stream.peek(buf)
            {
                Ok(num_read) => return Ok(num_read),
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut => { sleep().await; },
                Err(e) => return Err(e),
            }
        }
    }

    //--------------------------------------------------------------------------
    //  Writes the bytes in `buf` to the socket.
    //
    //  Returns the number of bytes written.
    //--------------------------------------------------------------------------
    pub async fn write( &mut self, buf: &[u8] ) -> Result<usize, std::io::Error>
    {
        loop
        {
            match self.std_stream.write(buf)
            {
                Ok(num) => return Ok(num),
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut
                ||
                (
                    e.kind() == ErrorKind::Other && e.raw_os_error() == Some(41)
                ) => { sleep().await; },
                Err(e) => return Err(e),
            }
        }
    }

    //--------------------------------------------------------------------------
    //  Sends all buffered data that was previously written on this socket and
    //  waits for receipt confirmation by the remote machine.
    //--------------------------------------------------------------------------
    pub async fn flush( &mut self ) -> Result<(), std::io::Error>
    {
        loop
        {
            match self.std_stream.flush()
            {
                Ok(()) => return Ok(()),
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut
                ||
                (
                    e.kind() == ErrorKind::Other && e.raw_os_error() == Some(41)
                ) => { sleep().await; },
                Err(e) => return Err(e),
            }
        }
    }

    //--------------------------------------------------------------------------
    //  Writes all bytes in `buf` to the socket.
    //--------------------------------------------------------------------------
    pub async fn write_all
    (
        &mut self,
        mut buf: &[u8],
    ) -> Result<(), std::io::Error>
    {
        while !buf.is_empty()
        {
            match self.std_stream.write(buf)
            {
                Ok(0) => {},
                Ok(num_written) => { buf = &buf[num_written..]; },
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut
                ||
                (
                    e.kind() == ErrorKind::Other && e.raw_os_error() == Some(41)
                ) => { sleep().await; },
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    //--------------------------------------------------------------------------
    //  Writes data from a slice of buffers.
    //
    //  Takes data from each buffer in order. May partially read the last
    //  buffer read.
    //
    //  Returns the number of bytes written.
    //--------------------------------------------------------------------------
    pub async fn write_vectored
    (
        &mut self,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Result<usize, std::io::Error>
    {
        loop
        {
            match self.std_stream.write_vectored(bufs)
            {
                Ok(num) => return Ok(num),
                Err(e) if e.kind() == ErrorKind::WouldBlock
                || e.kind() == ErrorKind::TimedOut
                ||
                (
                    e.kind() == ErrorKind::Other && e.raw_os_error() == Some(41)
                ) => { sleep().await; },
                Err(e) => return Err(e),
            }
        }
    }
}
