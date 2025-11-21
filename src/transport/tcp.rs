//! TCP transport implementations for DLMS/COSEM.
//!
//! This module provides TCP transport implementations for both synchronous and
//! asynchronous DLMS clients.
//!
//! # Features
//!
//! - `transport-tcp` - Synchronous TCP transport (requires `std`)
//! - `transport-tcp-async` - Asynchronous TCP transport (requires `async-client`)
//!   - `tokio` - Tokio runtime support
//!   - `smol` - Smol runtime support
//!   - `glommio` - Glommio runtime support (Linux only, io_uring)
//!   - `embassy` - Embassy runtime support (embedded-first async, std-compatible)
//!   - `embassy-net` - Embassy-net runtime support (true no_std, bare-metal)
//!
//! # Default Port and Timeouts
//!
//! The default DLMS/COSEM TCP port is **4059** as specified in IEC 62056-47.
//! Default timeouts are set to 30 seconds for both read and write operations.
//!
//! # TCP Configuration
//!
//! All TCP transports disable Nagle's algorithm (`TCP_NODELAY`) by default for
//! lower latency, which is important for real-time meter communication.
//!
//! # Examples
//!
//! ## Synchronous TCP Transport
//!
//! ```no_run
//! # #[cfg(feature = "transport-tcp")]
//! # {
//! use dlms_cosem::transport::tcp::TcpTransport;
//! use std::time::Duration;
//!
//! # fn example() -> std::io::Result<()> {
//! let mut transport = TcpTransport::connect("192.168.1.100:4059")?;
//! transport.set_read_timeout(Some(Duration::from_secs(30)))?;
//!
//! // Use with DlmsClient
//! // let client = ClientBuilder::new(transport, settings).build_with_heap(2048);
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Asynchronous TCP Transport (Tokio)
//!
//! ```no_run
//! # #[cfg(all(feature = "transport-tcp-async", feature = "tokio"))]
//! # {
//! use dlms_cosem::transport::tcp::AsyncTcpTransport;
//! use std::time::Duration;
//!
//! # async fn example() -> std::io::Result<()> {
//! let mut transport = AsyncTcpTransport::connect("192.168.1.100:4059").await?;
//! transport.set_read_timeout(Some(Duration::from_secs(30)));
//!
//! // Use with AsyncDlmsClient
//! // let client = AsyncDlmsClient::new(transport, settings);
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Using Specific Runtime
//!
//! ```no_run
//! # #[cfg(all(feature = "transport-tcp-async", feature = "tokio"))]
//! # {
//! use dlms_cosem::transport::tcp::TokioTcpTransport;
//!
//! # async fn example() -> std::io::Result<()> {
//! let transport = TokioTcpTransport::connect("192.168.1.100:4059").await?;
//! // Use with AsyncDlmsClient
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Embassy-Net Transport (no_std)
//!
//! ```no_run
//! # #[cfg(all(feature = "transport-tcp-async", feature = "embassy-net"))]
//! # {
//! use dlms_cosem::transport::tcp::{EmbassyNetTcpTransport, EMBASSY_NET_TCP_BUFFER_SIZE};
//! use embassy_net::{Stack, tcp::TcpSocket};
//!
//! # async fn example(stack: &'static Stack<embassy_net::driver::Driver>) -> Result<(), dlms_cosem::transport::tcp::EmbassyNetError> {
//! // Allocate static buffers for no_std
//! static mut RX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = [0; EMBASSY_NET_TCP_BUFFER_SIZE];
//! static mut TX_BUFFER: [u8; EMBASSY_NET_TCP_BUFFER_SIZE] = [0; EMBASSY_NET_TCP_BUFFER_SIZE];
//!
//! // Create and connect socket
//! let mut socket = TcpSocket::new(stack, unsafe { &mut RX_BUFFER }, unsafe { &mut TX_BUFFER });
//! socket.connect((embassy_net::Ipv4Address::new(192, 168, 1, 100), 4059))
//!     .await
//!     .map_err(|_| dlms_cosem::transport::tcp::EmbassyNetError::ConnectionClosed)?;
//!
//! let transport = EmbassyNetTcpTransport::new(socket);
//! // Use with AsyncDlmsClient
//! # Ok(())
//! # }
//! # }
//! ```

// Synchronous TCP transport
#[cfg(all(feature = "std", feature = "transport-tcp"))]
pub mod sync;

#[cfg(all(feature = "std", feature = "transport-tcp"))]
pub use sync::TcpTransport;

// Asynchronous TCP transport
#[cfg(all(feature = "std", feature = "transport-tcp-async"))]
pub mod r#async;

// Only export AsyncTcpTransport when at least one std-based runtime is enabled
#[cfg(all(
    feature = "std",
    feature = "transport-tcp-async",
    any(feature = "tokio", feature = "smol", feature = "glommio", feature = "embassy")
))]
pub use r#async::AsyncTcpTransport;

// Re-export runtime-specific types when available
#[cfg(all(feature = "std", feature = "transport-tcp-async", feature = "tokio"))]
pub use r#async::TokioTcpTransport;

#[cfg(all(feature = "std", feature = "transport-tcp-async", feature = "smol"))]
pub use r#async::SmolTcpTransport;

#[cfg(all(feature = "std", feature = "transport-tcp-async", feature = "glommio"))]
pub use r#async::GlommioTcpTransport;

#[cfg(all(feature = "std", feature = "transport-tcp-async", feature = "embassy"))]
pub use r#async::EmbassyTcpTransport;

// Embassy-net transport (no_std)
// Only compile when embassy-net is enabled AND std is disabled
// Embassy-net is specifically for no_std environments
#[cfg(all(feature = "embassy-net", not(feature = "std")))]
pub mod async_embassy_net;

#[cfg(all(feature = "embassy-net", not(feature = "std")))]
pub use async_embassy_net::{EMBASSY_NET_TCP_BUFFER_SIZE, EmbassyNetError, EmbassyNetTcpTransport};

// ============================================================================
// Shared Constants (used by both sync and async implementations)
// ============================================================================

#[cfg(feature = "std")]
use std::time::Duration;

/// Default TCP port for DLMS/COSEM communication (as per IEC 62056-47).
pub const DEFAULT_DLMS_TCP_PORT: u16 = 4059;

/// Default read timeout for TCP connections (30 seconds).
pub const DEFAULT_TCP_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Default write timeout for TCP connections (30 seconds).
pub const DEFAULT_TCP_WRITE_TIMEOUT: Duration = Duration::from_secs(30);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_DLMS_TCP_PORT, 4059);
        assert_eq!(DEFAULT_TCP_READ_TIMEOUT, Duration::from_secs(30));
        assert_eq!(DEFAULT_TCP_WRITE_TIMEOUT, Duration::from_secs(30));
    }
}
