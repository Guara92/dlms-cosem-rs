//! Embassy-Net TCP transport for no_std environments.
//!
//! This module provides true no_std async TCP support using embassy-net.
//! It requires an embassy-net Stack and works on bare-metal embedded systems.
//!
//! # Features
//!
//! - ✅ True no_std support (no standard library required)
//! - ✅ Minimal memory footprint (stack-allocated buffers)
//! - ✅ Zero heap allocations (compatible with heapless)
//! - ✅ Works on bare-metal MCUs (STM32, ESP32, nRF52, etc.)
//! - ✅ Compatible with embassy-executor
//! - ✅ Configurable timeouts
//!
//! # Platform Support
//!
//! This transport works on any platform with embassy-net support:
//! - STM32 microcontrollers (with embassy-stm32)
//! - ESP32 microcontrollers (with embassy-esp)
//! - nRF52 microcontrollers (with embassy-nrf)
//! - RP2040 (with embassy-rp)
//! - Any platform with embassy-net HAL support
//!
//! # Examples
//!
//! Basic usage with embassy-net:
//!
//! ```no_run
//! # #[cfg(all(feature = "embassy-net", feature = "async-client"))]
//! # {
//! use dlms_cosem::transport::tcp::EmbassyNetTcpTransport;
//! use embassy_net::{Stack, tcp::TcpSocket};
//! use embassy_time::Duration;
//!
//! # async fn example(stack: &'static Stack<embassy_net::driver::Driver>) -> Result<(), dlms_cosem::transport::tcp::EmbassyNetError> {
//! // Allocate RX/TX buffers (can use heapless arrays in no_std)
//! static mut RX_BUFFER: [u8; 2048] = [0; 2048];
//! static mut TX_BUFFER: [u8; 2048] = [0; 2048];
//!
//! // Create embassy-net TCP socket
//! let mut socket = TcpSocket::new(
//!     stack,
//!     unsafe { &mut RX_BUFFER },
//!     unsafe { &mut TX_BUFFER }
//! );
//!
//! // Connect to DLMS meter
//! socket.connect((embassy_net::Ipv4Address::new(192, 168, 1, 100), 4059))
//!     .await
//!     .map_err(|_| dlms_cosem::transport::tcp::EmbassyNetError::ConnectionClosed)?;
//!
//! // Create transport
//! let mut transport = EmbassyNetTcpTransport::new(socket);
//! transport.set_read_timeout(Some(Duration::from_secs(60)));
//!
//! // Use with AsyncDlmsClient...
//! # Ok(())
//! # }
//! # }
//! ```

use crate::transport::AsyncTransport;
use core::fmt;
use embassy_net::tcp::TcpSocket;
use embassy_time::{Duration, TimeoutError, with_timeout};
use embedded_io_async::Write;

/// Default read timeout for DLMS TCP connections (30 seconds)
const DEFAULT_TCP_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Default write timeout for DLMS TCP connections (30 seconds)
const DEFAULT_TCP_WRITE_TIMEOUT: Duration = Duration::from_secs(30);

/// Default TCP buffer size for embassy-net sockets (2KB is standard for DLMS)
///
/// This is the recommended buffer size for both RX and TX buffers when creating
/// embassy-net TCP sockets for DLMS communication.
///
/// # Memory Requirements
///
/// - RX buffer: 2048 bytes (static allocation)
/// - TX buffer: 2048 bytes (static allocation)
/// - Total: 4096 bytes per socket
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "embassy-net")]
/// # {
/// use dlms_cosem::transport::tcp::EMBASSY_NET_TCP_BUFFER_SIZE;
///
/// static mut RX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = [0; EMBASSY_NET_TCP_BUFFER_SIZE];
/// static mut TX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = [0; EMBASSY_NET_TCP_BUFFER_SIZE];
/// # }
/// ```
pub const EMBASSY_NET_TCP_BUFFER_SIZE: usize = 2048;

/// Asynchronous TCP transport using embassy-net (no_std).
///
/// This transport provides true no_std async TCP communication for DLMS/COSEM
/// using embassy-net. It works on bare-metal embedded systems (STM32, ESP32,
/// nRF52, etc.) with minimal memory footprint.
///
/// # Lifetime
///
/// The lifetime `'a` represents the lifetime of the embassy-net Stack and
/// the RX/TX buffers. These must outlive the transport.
///
/// # Memory Usage
///
/// - RX buffer: 2KB (stack-allocated)
/// - TX buffer: 2KB (stack-allocated)
/// - Transport overhead: <100 bytes
/// - Total: ~4KB RAM (no heap)
///
/// # Examples
///
/// ```no_run
/// # #[cfg(all(feature = "embassy-net", feature = "async-client"))]
/// # {
/// use dlms_cosem::transport::tcp::{EmbassyNetTcpTransport, EMBASSY_NET_TCP_BUFFER_SIZE};
/// use embassy_net::{Stack, tcp::TcpSocket};
/// use embassy_time::Duration;
///
/// # async fn example(stack: &'static Stack<embassy_net::driver::Driver>) -> Result<(), dlms_cosem::transport::tcp::EmbassyNetError> {
/// // Allocate buffers
/// static mut RX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = [0; EMBASSY_NET_TCP_BUFFER_SIZE];
/// static mut TX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = [0; EMBASSY_NET_TCP_BUFFER_SIZE];
///
/// // Create and connect socket
/// let mut socket = TcpSocket::new(stack, unsafe { &mut RX_BUFFER }, unsafe { &mut TX_BUFFER });
/// socket.connect((embassy_net::Ipv4Address::new(192, 168, 1, 100), 4059))
///     .await
///     .map_err(|_| dlms_cosem::transport::tcp::EmbassyNetError::ConnectionClosed)?;
///
/// // Create transport
/// let mut transport = EmbassyNetTcpTransport::new(socket);
/// transport.set_read_timeout(Some(Duration::from_secs(60)));
/// # Ok(())
/// # }
/// # }
/// ```
pub struct EmbassyNetTcpTransport<'a> {
    socket: TcpSocket<'a>,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl<'a> EmbassyNetTcpTransport<'a> {
    /// Creates a new embassy-net TCP transport from a connected socket.
    ///
    /// # Arguments
    ///
    /// * `socket` - A connected embassy-net TcpSocket
    ///
    /// # Note
    ///
    /// The socket must already be connected before creating the transport.
    /// Use `socket.connect()` before passing it to this constructor.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "embassy-net", feature = "async-client"))]
    /// # {
    /// use dlms_cosem::transport::tcp::{EmbassyNetTcpTransport, EMBASSY_NET_TCP_BUFFER_SIZE};
    /// use embassy_net::{Stack, tcp::TcpSocket, Ipv4Address};
    ///
    /// # async fn example(stack: &'static Stack<embassy_net::driver::Driver>) -> Result<(), dlms_cosem::transport::tcp::EmbassyNetError> {
    /// static mut RX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = [0; EMBASSY_NET_TCP_BUFFER_SIZE];
    /// static mut TX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = [0; EMBASSY_NET_TCP_BUFFER_SIZE];
    ///
    /// let mut socket = TcpSocket::new(stack, unsafe { &mut RX_BUFFER }, unsafe { &mut TX_BUFFER });
    /// socket.connect((Ipv4Address::new(192, 168, 1, 100), 4059))
    ///     .await
    ///     .map_err(|_| dlms_cosem::transport::tcp::EmbassyNetError::ConnectionClosed)?;
    ///
    /// let transport = EmbassyNetTcpTransport::new(socket);
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub fn new(socket: TcpSocket<'a>) -> Self {
        Self {
            socket,
            read_timeout: Some(DEFAULT_TCP_READ_TIMEOUT),
            write_timeout: Some(DEFAULT_TCP_WRITE_TIMEOUT),
        }
    }

    /// Sets the read timeout for the TCP socket.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The read timeout duration, or `None` to disable timeout
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "embassy-net", feature = "async-client"))]
    /// # {
    /// use dlms_cosem::transport::tcp::EmbassyNetTcpTransport;
    /// use embassy_net::tcp::TcpSocket;
    /// use embassy_time::Duration;
    ///
    /// # fn example(socket: TcpSocket<'static>) {
    /// let mut transport = EmbassyNetTcpTransport::new(socket);
    ///
    /// // Set custom timeout
    /// transport.set_read_timeout(Some(Duration::from_secs(60)));
    ///
    /// // Disable timeout
    /// transport.set_read_timeout(None);
    /// # }
    /// # }
    /// ```
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) {
        self.read_timeout = timeout;
    }

    /// Sets the write timeout for the TCP socket.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The write timeout duration, or `None` to disable timeout
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "embassy-net", feature = "async-client"))]
    /// # {
    /// use dlms_cosem::transport::tcp::EmbassyNetTcpTransport;
    /// use embassy_net::tcp::TcpSocket;
    /// use embassy_time::Duration;
    ///
    /// # fn example(socket: TcpSocket<'static>) {
    /// let mut transport = EmbassyNetTcpTransport::new(socket);
    ///
    /// // Set custom timeout
    /// transport.set_write_timeout(Some(Duration::from_secs(30)));
    ///
    /// // Disable timeout
    /// transport.set_write_timeout(None);
    /// # }
    /// # }
    /// ```
    pub fn set_write_timeout(&mut self, timeout: Option<Duration>) {
        self.write_timeout = timeout;
    }

    /// Returns whether the socket is connected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "embassy-net", feature = "async-client"))]
    /// # {
    /// use dlms_cosem::transport::tcp::EmbassyNetTcpTransport;
    /// use embassy_net::tcp::TcpSocket;
    ///
    /// # fn example(socket: TcpSocket<'static>) {
    /// let transport = EmbassyNetTcpTransport::new(socket);
    ///
    /// if transport.is_connected() {
    ///     // Socket is ready for communication
    /// }
    /// # }
    /// # }
    /// ```
    pub fn is_connected(&self) -> bool {
        self.socket.may_send()
    }
}

impl<'a> fmt::Debug for EmbassyNetTcpTransport<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmbassyNetTcpTransport")
            .field("read_timeout", &self.read_timeout)
            .field("write_timeout", &self.write_timeout)
            .field("connected", &self.is_connected())
            .finish()
    }
}

/// Error type for embassy-net TCP operations
///
/// This error type represents all possible errors that can occur during
/// embassy-net TCP operations. It is compatible with both std and no_std
/// environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbassyNetError {
    /// Operation timed out
    ///
    /// The requested operation did not complete within the specified timeout
    /// duration. This can occur during read or write operations.
    Timeout,

    /// Connection closed
    ///
    /// The TCP connection was closed by the remote peer or due to a network
    /// error. No further operations can be performed on this socket.
    ConnectionClosed,

    /// Write buffer full
    ///
    /// The socket's write buffer is full and cannot accept more data.
    /// This is a temporary condition; retry the operation after some data
    /// has been sent.
    WriteBufferFull,
}

impl fmt::Display for EmbassyNetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(f, "operation timed out"),
            Self::ConnectionClosed => write!(f, "connection closed"),
            Self::WriteBufferFull => write!(f, "write buffer full"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EmbassyNetError {}

impl<'a> AsyncTransport for EmbassyNetTcpTransport<'a> {
    type Error = EmbassyNetError;

    async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let write_fut = async {
            self.socket.write_all(data).await.map_err(|_| EmbassyNetError::WriteBufferFull)
        };

        if let Some(timeout_duration) = self.write_timeout {
            with_timeout(timeout_duration, write_fut).await.map_err(|_| EmbassyNetError::Timeout)?
        } else {
            write_fut.await
        }
    }

    async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let read_fut =
            async { self.socket.read(buffer).await.map_err(|_| EmbassyNetError::ConnectionClosed) };

        if let Some(timeout_duration) = self.read_timeout {
            with_timeout(timeout_duration, read_fut).await.map_err(|_| EmbassyNetError::Timeout)?
        } else {
            read_fut.await
        }
    }

    async fn recv_timeout(
        &mut self,
        buffer: &mut [u8],
        duration: core::time::Duration,
    ) -> Result<usize, Self::Error> {
        // Convert std Duration to embassy Duration
        let embassy_duration = Duration::from_millis(duration.as_millis() as u64);

        with_timeout(embassy_duration, self.socket.read(buffer))
            .await
            .map_err(|_: TimeoutError| EmbassyNetError::Timeout)?
            .map_err(|_| EmbassyNetError::ConnectionClosed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Verify timeout constants are set correctly
        assert_eq!(DEFAULT_TCP_READ_TIMEOUT, Duration::from_secs(30));
        assert_eq!(DEFAULT_TCP_WRITE_TIMEOUT, Duration::from_secs(30));
    }

    #[test]
    fn test_buffer_size_constant() {
        // Verify buffer size is appropriate for DLMS (2KB is standard)
        assert_eq!(EMBASSY_NET_TCP_BUFFER_SIZE, 2048);
    }

    #[test]
    fn test_error_display() {
        // Test error message formatting
        assert_eq!(EmbassyNetError::Timeout.to_string(), "operation timed out");
        assert_eq!(EmbassyNetError::ConnectionClosed.to_string(), "connection closed");
        assert_eq!(EmbassyNetError::WriteBufferFull.to_string(), "write buffer full");
    }

    #[test]
    fn test_error_equality() {
        // Test error equality and inequality
        assert_eq!(EmbassyNetError::Timeout, EmbassyNetError::Timeout);
        assert_ne!(EmbassyNetError::Timeout, EmbassyNetError::ConnectionClosed);
        assert_ne!(EmbassyNetError::ConnectionClosed, EmbassyNetError::WriteBufferFull);
    }

    #[test]
    fn test_error_clone() {
        // Verify errors are cloneable
        let err1 = EmbassyNetError::Timeout;
        let err2 = err1;
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_error_debug() {
        // Verify debug formatting works
        let err = EmbassyNetError::Timeout;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Timeout"));
    }

    // Note: Integration tests with actual embassy-net Stack require hardware setup
    // See examples/tcp_transport_embassy_net_nostd.rs for working patterns
}
