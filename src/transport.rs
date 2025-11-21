//! DLMS transport layer abstractions (sync and async).
//!
//! This module provides transport layer traits for both synchronous and asynchronous I/O:
//! - [`sync::Transport`] - Synchronous transport trait for blocking I/O
//! - [`async::AsyncTransport`] - Asynchronous transport trait for non-blocking I/O
//!
//! Transport implementations handle the low-level sending and receiving of bytes
//! over physical media (TCP, serial, etc.), while the client layer handles DLMS
//! protocol logic.

#[cfg(feature = "client")]
pub mod sync;

#[cfg(feature = "async-client")]
pub mod r#async;

// Re-export commonly used types for convenience
#[cfg(feature = "client")]
pub use sync::Transport;

#[cfg(feature = "async-client")]
pub use r#async::AsyncTransport;
