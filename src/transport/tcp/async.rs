//! Asynchronous TCP transport implementation for DLMS/COSEM.
//!
//! This module provides async TCP transport implementations that can be used with the
//! async DLMS client. It supports multiple async runtimes through feature flags.
//!
//! See parent module [`crate::transport::tcp`] for detailed documentation,
//! shared constants, and examples.

// Re-export runtime-specific transports
#[cfg(feature = "tokio")]
pub use tokio_impl::TokioTcpTransport;

#[cfg(feature = "smol")]
pub use smol_impl::SmolTcpTransport;

#[cfg(feature = "glommio")]
pub use glommio_impl::GlommioTcpTransport;

#[cfg(feature = "embassy")]
pub use embassy_impl::EmbassyTcpTransport;

// Re-export the default async transport for the enabled runtime
#[cfg(feature = "tokio")]
pub use TokioTcpTransport as AsyncTcpTransport;

#[cfg(all(feature = "smol", not(feature = "tokio")))]
pub use SmolTcpTransport as AsyncTcpTransport;

#[cfg(all(feature = "glommio", not(any(feature = "tokio", feature = "smol"))))]
pub use GlommioTcpTransport as AsyncTcpTransport;

#[cfg(all(
    feature = "embassy",
    not(any(feature = "tokio", feature = "smol", feature = "glommio"))
))]
pub use EmbassyTcpTransport as AsyncTcpTransport;

// ============================================================================
// Tokio Implementation
// ============================================================================

#[cfg(feature = "tokio")]
mod tokio_impl {
    use super::super::{DEFAULT_TCP_READ_TIMEOUT, DEFAULT_TCP_WRITE_TIMEOUT};
    use crate::transport::r#async::AsyncTransport;
    use std::io;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::timeout;

    /// Asynchronous TCP transport using Tokio runtime.
    ///
    /// This transport wraps a Tokio `TcpStream` and implements the `AsyncTransport` trait,
    /// allowing it to be used with the async DLMS client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "tokio")]
    /// # {
    /// use dlms_cosem::transport::tcp::TokioTcpTransport;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let mut transport = TokioTcpTransport::connect("192.168.1.100:4059").await?;
    /// transport.set_read_timeout(Some(Duration::from_secs(60)));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    #[derive(Debug)]
    pub struct TokioTcpTransport {
        stream: TcpStream,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    }

    impl TokioTcpTransport {
        /// Creates a new Tokio TCP transport by connecting to the specified address.
        ///
        /// # Arguments
        ///
        /// * `addr` - The address to connect to (e.g., "192.168.1.100:4059")
        ///
        /// # Returns
        ///
        /// * `Ok(TokioTcpTransport)` - Successfully connected transport
        /// * `Err(io::Error)` - Connection failed
        pub async fn connect<A: tokio::net::ToSocketAddrs>(addr: A) -> io::Result<Self> {
            let stream = TcpStream::connect(addr).await?;

            // Disable Nagle's algorithm for lower latency
            stream.set_nodelay(true)?;

            Ok(Self {
                stream,
                read_timeout: Some(DEFAULT_TCP_READ_TIMEOUT),
                write_timeout: Some(DEFAULT_TCP_WRITE_TIMEOUT),
            })
        }

        /// Sets the read timeout for the TCP stream.
        ///
        /// # Arguments
        ///
        /// * `timeout` - The read timeout duration, or `None` to disable timeout
        pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
            self.read_timeout = timeout;
        }

        /// Sets the write timeout for the TCP stream.
        ///
        /// # Arguments
        ///
        /// * `timeout` - The write timeout duration, or `None` to disable timeout
        pub fn set_write_timeout(&mut self, timeout: Option<Duration>) {
            self.write_timeout = timeout;
        }

        /// Returns the local socket address of the TCP connection.
        pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
            self.stream.local_addr()
        }

        /// Returns the remote socket address of the TCP connection.
        pub fn peer_addr(&self) -> io::Result<std::net::SocketAddr> {
            self.stream.peer_addr()
        }
    }

    impl AsyncTransport for TokioTcpTransport {
        type Error = io::Error;

        async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            let write_op = async {
                self.stream.write_all(data).await?;
                self.stream.flush().await?;
                Ok::<(), io::Error>(())
            };

            if let Some(duration) = self.write_timeout {
                timeout(duration, write_op)
                    .await
                    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "write timeout"))?
            } else {
                write_op.await
            }
        }

        async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
            let read_op = self.stream.read(buffer);

            if let Some(duration) = self.read_timeout {
                timeout(duration, read_op)
                    .await
                    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "read timeout"))?
            } else {
                read_op.await
            }
        }

        async fn recv_timeout(
            &mut self,
            buffer: &mut [u8],
            duration: Duration,
        ) -> Result<usize, Self::Error> {
            timeout(duration, self.stream.read(buffer))
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "read timeout"))?
        }
    }
}

// ============================================================================
// Smol Implementation
// ============================================================================

#[cfg(feature = "smol")]
mod smol_impl {
    use super::super::{DEFAULT_TCP_READ_TIMEOUT, DEFAULT_TCP_WRITE_TIMEOUT};
    use crate::transport::r#async::AsyncTransport;
    use smol::io::{AsyncReadExt, AsyncWriteExt};
    use smol::net::TcpStream;
    use std::io;
    use std::time::Duration;

    /// Asynchronous TCP transport using Smol runtime.
    ///
    /// This transport wraps a Smol `TcpStream` and implements the `AsyncTransport` trait,
    /// allowing it to be used with the async DLMS client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "smol")]
    /// # {
    /// use dlms_cosem::transport::tcp::SmolTcpTransport;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let mut transport = SmolTcpTransport::connect("192.168.1.100:4059").await?;
    /// transport.set_read_timeout(Some(Duration::from_secs(60)));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    #[derive(Debug)]
    pub struct SmolTcpTransport {
        stream: TcpStream,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    }

    impl SmolTcpTransport {
        /// Creates a new Smol TCP transport by connecting to the specified address.
        pub async fn connect<A: smol::net::AsyncToSocketAddrs>(addr: A) -> io::Result<Self> {
            let stream = TcpStream::connect(addr).await?;

            // Disable Nagle's algorithm for lower latency
            stream.set_nodelay(true)?;

            Ok(Self {
                stream,
                read_timeout: Some(DEFAULT_TCP_READ_TIMEOUT),
                write_timeout: Some(DEFAULT_TCP_WRITE_TIMEOUT),
            })
        }

        /// Sets the read timeout for the TCP stream.
        pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
            self.read_timeout = timeout;
        }

        /// Sets the write timeout for the TCP stream.
        pub fn set_write_timeout(&mut self, timeout: Option<Duration>) {
            self.write_timeout = timeout;
        }

        /// Returns the local socket address of the TCP connection.
        pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
            self.stream.local_addr()
        }

        /// Returns the remote socket address of the TCP connection.
        pub fn peer_addr(&self) -> io::Result<std::net::SocketAddr> {
            self.stream.peer_addr()
        }
    }

    impl AsyncTransport for SmolTcpTransport {
        type Error = io::Error;

        async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            let write_op = async {
                self.stream.write_all(data).await?;
                self.stream.flush().await?;
                Ok::<(), io::Error>(())
            };

            if let Some(duration) = self.write_timeout {
                smol::future::or(
                    async {
                        smol::Timer::after(duration).await;
                        Err(io::Error::new(io::ErrorKind::TimedOut, "write timeout"))
                    },
                    write_op,
                )
                .await
            } else {
                write_op.await
            }
        }

        async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
            let read_op = self.stream.read(buffer);

            if let Some(duration) = self.read_timeout {
                smol::future::or(
                    async {
                        smol::Timer::after(duration).await;
                        Err(io::Error::new(io::ErrorKind::TimedOut, "read timeout"))
                    },
                    read_op,
                )
                .await
            } else {
                read_op.await
            }
        }

        async fn recv_timeout(
            &mut self,
            buffer: &mut [u8],
            duration: Duration,
        ) -> Result<usize, Self::Error> {
            smol::future::or(
                async {
                    smol::Timer::after(duration).await;
                    Err(io::Error::new(io::ErrorKind::TimedOut, "read timeout"))
                },
                self.stream.read(buffer),
            )
            .await
        }
    }
}

// ============================================================================
// Glommio Implementation (Linux only, io_uring)
// ============================================================================

#[cfg(feature = "glommio")]
mod glommio_impl {
    use super::super::{DEFAULT_TCP_READ_TIMEOUT, DEFAULT_TCP_WRITE_TIMEOUT};
    use crate::transport::r#async::AsyncTransport;
    use futures_lite::io::{AsyncReadExt, AsyncWriteExt};
    use glommio::net::TcpStream;
    use std::io;
    use std::time::Duration;

    /// Asynchronous TCP transport using Glommio runtime.
    ///
    /// This transport wraps a Glommio `TcpStream` and implements the `AsyncTransport` trait,
    /// allowing it to be used with the async DLMS client.
    ///
    /// Glommio is a thread-per-core runtime for Linux that uses io_uring for high-performance I/O.
    /// It is optimized for low-latency, high-throughput applications.
    ///
    /// # Platform Support
    ///
    /// Glommio only works on Linux systems with io_uring support (kernel 5.8+).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "glommio")]
    /// # {
    /// use dlms_cosem::transport::tcp::GlommioTcpTransport;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let mut transport = GlommioTcpTransport::connect("192.168.1.100:4059").await?;
    /// transport.set_read_timeout(Some(Duration::from_secs(60)));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    #[derive(Debug)]
    pub struct GlommioTcpTransport {
        stream: TcpStream,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    }

    impl GlommioTcpTransport {
        /// Creates a new Glommio TCP transport by connecting to the specified address.
        ///
        /// # Arguments
        ///
        /// * `addr` - The address to connect to (e.g., "192.168.1.100:4059")
        ///
        /// # Returns
        ///
        /// * `Ok(GlommioTcpTransport)` - Successfully connected transport
        /// * `Err(io::Error)` - Connection failed
        ///
        /// # Platform Requirements
        ///
        /// This function requires Linux with io_uring support (kernel 5.8+).
        pub async fn connect<A: std::net::ToSocketAddrs>(addr: A) -> io::Result<Self> {
            let stream = TcpStream::connect(addr).await?;

            // Disable Nagle's algorithm for lower latency
            stream.set_nodelay(true)?;

            Ok(Self {
                stream,
                read_timeout: Some(DEFAULT_TCP_READ_TIMEOUT),
                write_timeout: Some(DEFAULT_TCP_WRITE_TIMEOUT),
            })
        }

        /// Sets the read timeout for the TCP stream.
        ///
        /// # Arguments
        ///
        /// * `timeout` - The read timeout duration, or `None` to disable timeout
        pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
            self.read_timeout = timeout;
        }

        /// Sets the write timeout for the TCP stream.
        ///
        /// # Arguments
        ///
        /// * `timeout` - The write timeout duration, or `None` to disable timeout
        pub fn set_write_timeout(&mut self, timeout: Option<Duration>) {
            self.write_timeout = timeout;
        }

        /// Returns the local socket address of the TCP connection.
        pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
            self.stream.local_addr().map_err(|e| match e {
                glommio::GlommioError::IoError(io_err) => io_err,
                _ => io::Error::other("failed to get local address"),
            })
        }

        /// Returns the remote socket address of the TCP connection.
        pub fn peer_addr(&self) -> io::Result<std::net::SocketAddr> {
            self.stream.peer_addr().map_err(|e| match e {
                glommio::GlommioError::IoError(io_err) => io_err,
                _ => io::Error::other("failed to get peer address"),
            })
        }
    }

    impl AsyncTransport for GlommioTcpTransport {
        type Error = io::Error;

        async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            let write_op = async {
                self.stream.write_all(data).await?;
                self.stream.flush().await?;
                Ok::<(), io::Error>(())
            };

            if let Some(duration) = self.write_timeout {
                glommio::timer::timeout(duration, async {
                    write_op.await.map_err(glommio::GlommioError::IoError)
                })
                .await
                .map_err(|e| match e {
                    glommio::GlommioError::IoError(io_err) => io_err,
                    _ => io::Error::new(io::ErrorKind::TimedOut, "write timeout"),
                })
            } else {
                write_op.await
            }
        }

        async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
            if let Some(duration) = self.read_timeout {
                glommio::timer::timeout(duration, async {
                    self.stream.read(buffer).await.map_err(glommio::GlommioError::IoError)
                })
                .await
                .map_err(|e| match e {
                    glommio::GlommioError::IoError(io_err) => io_err,
                    _ => io::Error::new(io::ErrorKind::TimedOut, "read timeout"),
                })
            } else {
                self.stream.read(buffer).await
            }
        }

        async fn recv_timeout(
            &mut self,
            buffer: &mut [u8],
            duration: Duration,
        ) -> Result<usize, Self::Error> {
            glommio::timer::timeout(duration, async {
                self.stream.read(buffer).await.map_err(glommio::GlommioError::IoError)
            })
            .await
            .map_err(|e| match e {
                glommio::GlommioError::IoError(io_err) => io_err,
                _ => io::Error::new(io::ErrorKind::TimedOut, "read timeout"),
            })
        }
    }
}

// ============================================================================
// Embassy Implementation
// ============================================================================

#[cfg(feature = "embassy")]
mod embassy_impl {
    use super::super::{DEFAULT_TCP_READ_TIMEOUT, DEFAULT_TCP_WRITE_TIMEOUT};
    use crate::transport::r#async::AsyncTransport;
    use embassy_futures::select::{Either, select};
    use std::io;
    use std::time::Duration;

    /// Asynchronous TCP transport using Embassy runtime.
    ///
    /// This transport provides Embassy-compatible async TCP communication for DLMS/COSEM.
    /// Embassy is designed for embedded systems but can also be used in std environments.
    ///
    /// This implementation uses `std::net::TcpStream` in non-blocking mode with Embassy's
    /// async primitives for compatibility across different Embassy executors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "embassy")]
    /// # {
    /// use dlms_cosem::transport::tcp::EmbassyTcpTransport;
    /// use std::time::Duration;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let mut transport = EmbassyTcpTransport::connect("192.168.1.100:4059").await?;
    /// transport.set_read_timeout(Some(Duration::from_secs(60)));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    #[derive(Debug)]
    pub struct EmbassyTcpTransport {
        stream: std::net::TcpStream,
        read_timeout: Option<Duration>,
        write_timeout: Option<Duration>,
    }

    impl EmbassyTcpTransport {
        /// Creates a new Embassy TCP transport by connecting to the specified address.
        ///
        /// # Arguments
        ///
        /// * `addr` - The address to connect to (e.g., "192.168.1.100:4059")
        ///
        /// # Returns
        ///
        /// * `Ok(EmbassyTcpTransport)` - Successfully connected transport
        /// * `Err(io::Error)` - Connection failed
        pub async fn connect<A: std::net::ToSocketAddrs>(addr: A) -> io::Result<Self> {
            // Connect synchronously for now (Embassy executors vary)
            // In a real embedded environment, this would use embassy-net
            let stream = std::net::TcpStream::connect(addr)?;

            // Set non-blocking mode for async operations
            stream.set_nonblocking(true)?;

            // Disable Nagle's algorithm for lower latency
            stream.set_nodelay(true)?;

            Ok(Self {
                stream,
                read_timeout: Some(DEFAULT_TCP_READ_TIMEOUT),
                write_timeout: Some(DEFAULT_TCP_WRITE_TIMEOUT),
            })
        }

        /// Sets the read timeout for the TCP stream.
        ///
        /// # Arguments
        ///
        /// * `timeout` - The read timeout duration, or `None` to disable timeout
        pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
            self.read_timeout = timeout;
        }

        /// Sets the write timeout for the TCP stream.
        ///
        /// # Arguments
        ///
        /// * `timeout` - The write timeout duration, or `None` to disable timeout
        pub fn set_write_timeout(&mut self, timeout: Option<Duration>) {
            self.write_timeout = timeout;
        }

        /// Returns the local socket address of the TCP connection.
        pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
            self.stream.local_addr()
        }

        /// Returns the remote socket address of the TCP connection.
        pub fn peer_addr(&self) -> io::Result<std::net::SocketAddr> {
            self.stream.peer_addr()
        }

        /// Async write using non-blocking I/O
        async fn write_async(&mut self, data: &[u8]) -> io::Result<()> {
            use std::io::Write;

            let mut written = 0;
            while written < data.len() {
                match self.stream.write(&data[written..]) {
                    Ok(n) => {
                        written += n;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // Yield and retry
                        embassy_futures::yield_now().await;
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }

            // Flush the stream
            loop {
                match self.stream.flush() {
                    Ok(_) => break,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        embassy_futures::yield_now().await;
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }

            Ok(())
        }

        /// Async read using non-blocking I/O
        async fn read_async(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            use std::io::Read;

            loop {
                match self.stream.read(buffer) {
                    Ok(n) => return Ok(n),
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // Yield and retry
                        embassy_futures::yield_now().await;
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }

    impl AsyncTransport for EmbassyTcpTransport {
        type Error = io::Error;

        async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            let write_timeout = self.write_timeout;
            let write_fut = self.write_async(data);

            if let Some(timeout_duration) = write_timeout {
                let timer = async {
                    let start = std::time::Instant::now();
                    loop {
                        if start.elapsed() >= timeout_duration {
                            break;
                        }
                        embassy_futures::yield_now().await;
                    }
                };

                match select(write_fut, timer).await {
                    Either::First(result) => result,
                    Either::Second(_) => {
                        Err(io::Error::new(io::ErrorKind::TimedOut, "operation timeout"))
                    }
                }
            } else {
                write_fut.await
            }
        }

        async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
            let read_timeout = self.read_timeout;
            let read_fut = self.read_async(buffer);

            if let Some(timeout_duration) = read_timeout {
                let timer = async {
                    let start = std::time::Instant::now();
                    loop {
                        if start.elapsed() >= timeout_duration {
                            break;
                        }
                        embassy_futures::yield_now().await;
                    }
                };

                match select(read_fut, timer).await {
                    Either::First(result) => result,
                    Either::Second(_) => {
                        Err(io::Error::new(io::ErrorKind::TimedOut, "operation timeout"))
                    }
                }
            } else {
                read_fut.await
            }
        }

        async fn recv_timeout(
            &mut self,
            buffer: &mut [u8],
            duration: Duration,
        ) -> Result<usize, Self::Error> {
            let read_fut = self.read_async(buffer);

            let timer = async {
                let start = std::time::Instant::now();
                loop {
                    if start.elapsed() >= duration {
                        break;
                    }
                    embassy_futures::yield_now().await;
                }
            };

            match select(read_fut, timer).await {
                Either::First(result) => result,
                Either::Second(_) => {
                    Err(io::Error::new(io::ErrorKind::TimedOut, "operation timeout"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        DEFAULT_DLMS_TCP_PORT, DEFAULT_TCP_READ_TIMEOUT, DEFAULT_TCP_WRITE_TIMEOUT,
    };
    use std::time::Duration;

    #[test]
    fn test_constants_accessible() {
        // Verify we can access shared constants from parent module
        assert_eq!(DEFAULT_DLMS_TCP_PORT, 4059);
        assert_eq!(DEFAULT_TCP_READ_TIMEOUT, Duration::from_secs(30));
        assert_eq!(DEFAULT_TCP_WRITE_TIMEOUT, Duration::from_secs(30));
    }

    #[test]
    fn test_runtime_type_exports() {
        // Verify that runtime-specific types are properly exported
        #[cfg(feature = "tokio")]
        {
            let _ = std::any::type_name::<super::TokioTcpTransport>();
        }

        #[cfg(feature = "smol")]
        {
            let _ = std::any::type_name::<super::SmolTcpTransport>();
        }

        #[cfg(feature = "glommio")]
        {
            let _ = std::any::type_name::<super::GlommioTcpTransport>();
        }

        #[cfg(feature = "embassy")]
        {
            let _ = std::any::type_name::<super::EmbassyTcpTransport>();
        }
    }

    // Note: Integration tests with actual TCP connections are in examples/
}
