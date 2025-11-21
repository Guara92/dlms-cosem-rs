#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::time::Duration;

/// Trait representing the underlying transport layer for DLMS/COSEM communication.
///
/// This trait allows the `DlmsClient` to be agnostic of the actual communication medium
/// (TCP, UDP, Serial, HDLC, etc.).
///
/// Implementations should handle the low-level details of sending and receiving bytes.
pub trait Transport: core::fmt::Debug {
    /// The error type returned by transport operations.
    type Error: core::fmt::Debug;

    /// Sends data to the remote device.
    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Receives data from the remote device.
    ///
    /// This function should populate the provided buffer with received data.
    /// Returns the number of bytes read.
    fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;

    /// Receives data from the remote device with a timeout.
    ///
    /// This function should populate the provided buffer with received data.
    /// Returns the number of bytes read, or an error if the timeout expires.
    ///
    /// Default implementation calls `recv` (no timeout support).
    /// Override this method to provide actual timeout functionality.
    #[cfg(feature = "std")]
    fn recv_timeout(
        &mut self,
        buffer: &mut [u8],
        _timeout: Duration,
    ) -> Result<usize, Self::Error> {
        self.recv(buffer)
    }
}
