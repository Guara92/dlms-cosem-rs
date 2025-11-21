//! DLMS transport layer abstractions (sync and async).
//!
//! This module provides transport layer traits and implementations for both
//! synchronous and asynchronous I/O:
//!
//! # Transport Traits
//!
//! - [`sync::Transport`] - Synchronous transport trait for blocking I/O
//! - [`async::AsyncTransport`] - Asynchronous transport trait for non-blocking I/O
//!
//! # Transport Implementations
//!
//! ## TCP Transports
//!
//! - [`tcp::TcpTransport`] - Synchronous TCP transport (feature: `transport-tcp`)
//! - [`tcp::AsyncTcpTransport`] - Async TCP transport (feature: `transport-tcp-async`)
//!   - [`tcp::TokioTcpTransport`] - Tokio runtime (feature: `tokio`)
//!   - [`tcp::SmolTcpTransport`] - Smol runtime (feature: `smol`)
//!   - [`tcp::GlommioTcpTransport`] - Glommio runtime (feature: `glommio`)
//!
//! ## HDLC Wrappers
//!
//! - [`hdlc::HdlcTransport`] - Synchronous HDLC wrapper (feature: `transport-hdlc`)
//! - [`hdlc::AsyncHdlcTransport`] - Async HDLC wrapper (feature: `transport-hdlc-async`)
//!
//! # Examples
//!
//! ## Synchronous TCP Transport
//!
//! ```no_run
//! # #[cfg(feature = "transport-tcp")]
//! # {
//! use dlms_cosem::transport::tcp::TcpTransport;
//!
//! # fn example() -> std::io::Result<()> {
//! let mut transport = TcpTransport::connect("192.168.1.100:4059")?;
//! // Use with DlmsClient
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Async TCP Transport with Tokio
//!
//! ```no_run
//! # #[cfg(all(feature = "transport-tcp-async", feature = "tokio"))]
//! # {
//! use dlms_cosem::transport::tcp::AsyncTcpTransport;
//!
//! # async fn example() -> std::io::Result<()> {
//! let mut transport = AsyncTcpTransport::connect("192.168.1.100:4059").await?;
//! // Use with AsyncDlmsClient
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## TCP with HDLC Framing
//!
//! ```no_run
//! # #[cfg(all(feature = "transport-tcp", feature = "transport-hdlc"))]
//! # {
//! use dlms_cosem::transport::tcp::TcpTransport;
//! use dlms_cosem::transport::hdlc::HdlcTransport;
//!
//! # fn example() -> std::io::Result<()> {
//! let tcp = TcpTransport::connect("192.168.1.100:4059")?;
//! let mut hdlc = HdlcTransport::new(tcp, 0x01, 0x10);
//! // Use with DlmsClient
//! # Ok(())
//! # }
//! # }
//! ```

#[cfg(feature = "client")]
pub mod sync;

#[cfg(feature = "async-client")]
pub mod r#async;

// Transport implementations
#[cfg(any(feature = "transport-tcp", feature = "transport-tcp-async"))]
pub mod tcp;

#[cfg(any(feature = "transport-hdlc", feature = "transport-hdlc-async"))]
pub mod hdlc;

// Re-export commonly used types for convenience
#[cfg(feature = "client")]
pub use sync::Transport;

#[cfg(feature = "async-client")]
pub use r#async::AsyncTransport;

// Re-export TCP transports
#[cfg(feature = "transport-tcp")]
pub use tcp::TcpTransport;

// Only export AsyncTcpTransport when at least one std-based runtime is enabled
#[cfg(all(
    feature = "transport-tcp-async",
    any(feature = "tokio", feature = "smol", feature = "glommio", feature = "embassy")
))]
pub use tcp::AsyncTcpTransport;

// Re-export TCP constants (from parent module)
#[cfg(any(feature = "transport-tcp", feature = "transport-tcp-async"))]
pub use tcp::{DEFAULT_DLMS_TCP_PORT, DEFAULT_TCP_READ_TIMEOUT, DEFAULT_TCP_WRITE_TIMEOUT};

// Re-export HDLC wrappers
#[cfg(feature = "transport-hdlc")]
pub use hdlc::HdlcTransport;

#[cfg(feature = "transport-hdlc-async")]
pub use hdlc::AsyncHdlcTransport;

// Re-export HDLC types and constants (from parent module)
#[cfg(any(feature = "transport-hdlc", feature = "transport-hdlc-async"))]
pub use hdlc::{HDLC_FLAG, HdlcError, MAX_HDLC_FRAME_SIZE};
