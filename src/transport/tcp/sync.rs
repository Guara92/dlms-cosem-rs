//! Synchronous TCP transport implementation for DLMS/COSEM.
//!
//! This module provides a TCP transport implementation that can be used with the
//! synchronous DLMS client. It wraps a standard TCP stream and implements the
//! `Transport` trait.
//!
//! See parent module [`crate::transport::tcp`] for detailed documentation,
//! shared constants, and examples.

use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::transport::sync::Transport;

// Import shared constants from parent module
use super::{DEFAULT_TCP_READ_TIMEOUT, DEFAULT_TCP_WRITE_TIMEOUT};

/// Synchronous TCP transport for DLMS/COSEM communication.
///
/// This transport wraps a standard `TcpStream` and implements the `Transport` trait,
/// allowing it to be used with the synchronous DLMS client.
///
/// # Connection Management
///
/// The transport maintains a persistent TCP connection. If the connection is lost,
/// the transport will return an error, and a new transport instance must be created.
///
/// # Timeouts
///
/// The transport supports configurable read and write timeouts. By default:
/// - Read timeout: 30 seconds
/// - Write timeout: 30 seconds
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "transport-tcp")]
/// # {
/// use dlms_cosem::transport::tcp::TcpTransport;
/// use std::time::Duration;
///
/// // Connect to a DLMS server on default port
/// let mut transport = TcpTransport::connect("192.168.1.100:4059")
///     .expect("Failed to connect");
///
/// // Configure timeouts
/// transport.set_read_timeout(Some(Duration::from_secs(60)))
///     .expect("Failed to set read timeout");
/// transport.set_write_timeout(Some(Duration::from_secs(60)))
///     .expect("Failed to set write timeout");
/// # }
/// ```
#[derive(Debug)]
pub struct TcpTransport {
    /// The underlying TCP stream.
    stream: TcpStream,
}

impl TcpTransport {
    /// Creates a new TCP transport by connecting to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to connect to (e.g., "192.168.1.100:4059")
    ///
    /// # Returns
    ///
    /// * `Ok(TcpTransport)` - Successfully connected transport
    /// * `Err(io::Error)` - Connection failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "transport-tcp")]
    /// # {
    /// use dlms_cosem::transport::tcp::TcpTransport;
    ///
    /// // Connect using host:port
    /// let transport = TcpTransport::connect("192.168.1.100:4059")?;
    ///
    /// // Connect using default DLMS port
    /// let transport = TcpTransport::connect(("192.168.1.100", 4059))?;
    /// # Ok::<(), std::io::Error>(())
    /// # }
    /// ```
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;

        // Set default timeouts
        stream.set_read_timeout(Some(DEFAULT_TCP_READ_TIMEOUT))?;
        stream.set_write_timeout(Some(DEFAULT_TCP_WRITE_TIMEOUT))?;

        // Disable Nagle's algorithm for lower latency
        stream.set_nodelay(true)?;

        Ok(Self { stream })
    }

    /// Sets the read timeout for the TCP stream.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The read timeout duration, or `None` to disable timeout
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "transport-tcp")]
    /// # {
    /// use dlms_cosem::transport::tcp::TcpTransport;
    /// use std::time::Duration;
    ///
    /// let mut transport = TcpTransport::connect("192.168.1.100:4059")?;
    /// transport.set_read_timeout(Some(Duration::from_secs(60)))?;
    /// # Ok::<(), std::io::Error>(())
    /// # }
    /// ```
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(timeout)
    }

    /// Sets the write timeout for the TCP stream.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The write timeout duration, or `None` to disable timeout
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "transport-tcp")]
    /// # {
    /// use dlms_cosem::transport::tcp::TcpTransport;
    /// use std::time::Duration;
    ///
    /// let mut transport = TcpTransport::connect("192.168.1.100:4059")?;
    /// transport.set_write_timeout(Some(Duration::from_secs(60)))?;
    /// # Ok::<(), std::io::Error>(())
    /// # }
    /// ```
    pub fn set_write_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.stream.set_write_timeout(timeout)
    }

    /// Returns the local socket address of the TCP connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "transport-tcp")]
    /// # {
    /// use dlms_cosem::transport::tcp::TcpTransport;
    ///
    /// let transport = TcpTransport::connect("192.168.1.100:4059")?;
    /// let local_addr = transport.local_addr()?;
    /// println!("Local address: {}", local_addr);
    /// # Ok::<(), std::io::Error>(())
    /// # }
    /// ```
    pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.stream.local_addr()
    }

    /// Returns the remote socket address of the TCP connection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "transport-tcp")]
    /// # {
    /// use dlms_cosem::transport::tcp::TcpTransport;
    ///
    /// let transport = TcpTransport::connect("192.168.1.100:4059")?;
    /// let peer_addr = transport.peer_addr()?;
    /// println!("Connected to: {}", peer_addr);
    /// # Ok::<(), std::io::Error>(())
    /// # }
    /// ```
    pub fn peer_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.stream.peer_addr()
    }

    /// Shuts down the read, write, or both halves of the TCP connection.
    ///
    /// # Arguments
    ///
    /// * `how` - Specifies which parts of the connection to shut down
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "transport-tcp")]
    /// # {
    /// use dlms_cosem::transport::tcp::TcpTransport;
    /// use std::net::Shutdown;
    ///
    /// let mut transport = TcpTransport::connect("192.168.1.100:4059")?;
    /// // ... use transport ...
    /// transport.shutdown(Shutdown::Both)?;
    /// # Ok::<(), std::io::Error>(())
    /// # }
    /// ```
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        self.stream.shutdown(how)
    }
}

impl Transport for TcpTransport {
    type Error = io::Error;

    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.stream.write_all(data)?;
        self.stream.flush()?;
        Ok(())
    }

    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        self.stream.read(buffer)
    }

    fn recv_timeout(&mut self, buffer: &mut [u8], timeout: Duration) -> Result<usize, Self::Error> {
        let original_timeout = self.stream.read_timeout()?;
        self.stream.set_read_timeout(Some(timeout))?;

        let result = self.stream.read(buffer);

        // Restore original timeout
        if let Err(restore_err) = self.stream.set_read_timeout(original_timeout) {
            // If we failed to restore the timeout but succeeded reading,
            // we should still report the restore error
            if result.is_ok() {
                return Err(restore_err);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_accessible() {
        // Verify we can access shared constants from parent module
        use super::super::DEFAULT_DLMS_TCP_PORT;
        assert_eq!(DEFAULT_DLMS_TCP_PORT, 4059);
        assert_eq!(DEFAULT_TCP_READ_TIMEOUT, Duration::from_secs(30));
        assert_eq!(DEFAULT_TCP_WRITE_TIMEOUT, Duration::from_secs(30));
    }

    // Note: Integration tests with actual TCP connections are in tests/tcp_transport.rs
}
