//! DLMS client implementations (sync and async).
//!
//! This module provides both synchronous and asynchronous DLMS client implementations:
//! - [`sync::DlmsClient`] - Synchronous client using [`crate::transport::sync::Transport`]
//! - [`async::AsyncDlmsClient`] - Asynchronous client using [`crate::transport::async::AsyncTransport`]
//!
//! Both clients share common constants and error types defined in this module.

// ============================================================================
// Shared Constants
// ============================================================================

/// Class ID for Clock object (COSEM interface class 8)
pub const CLOCK_CLASS_ID: u16 = 8;

/// Attribute ID for Clock.time (attribute 2)
pub const CLOCK_TIME_ATTRIBUTE_ID: i8 = 2;

/// Class ID for ProfileGeneric object (COSEM interface class 7)
pub const PROFILE_GENERIC_CLASS_ID: u16 = 7;

/// Attribute ID for ProfileGeneric.buffer (attribute 2)
pub const PROFILE_GENERIC_BUFFER_ATTRIBUTE_ID: i8 = 2;

/// Default maximum attributes per request (Gurux compatibility)
///
/// This default value matches Gurux DLMS.c behavior for maximum
/// attributes in a single GET-Request-With-List or SET-Request-With-List.
pub const DEFAULT_MAX_ATTRIBUTES_PER_REQUEST: usize = 10;

/// Size of the receive buffer for APDU responses
///
/// This buffer size is sufficient for most DLMS responses while
/// keeping memory usage reasonable.
pub const RECV_BUFFER_SIZE: usize = 4096;

// ============================================================================
// Submodules
// ============================================================================

#[cfg(feature = "client")]
pub mod sync;

#[cfg(feature = "async-client")]
pub mod r#async;

// Re-export commonly used types for convenience

// Common types (available with either client or async-client)
#[cfg(any(feature = "client", feature = "async-client"))]
pub use sync::{ClientSettings, SessionState};

// Sync client types
#[cfg(feature = "client")]
pub use sync::{Buffer, ClientBuilder, ClientError, DlmsClient, DlmsSession};

// Async client types
#[cfg(feature = "async-client")]
pub use r#async::{AsyncClientBuilder, AsyncClientError, AsyncDlmsClient};
