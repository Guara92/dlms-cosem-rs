#![cfg_attr(not(feature = "std"), no_std)]
// We use `impl Future + Send` instead of `async fn` to be explicit about Send bounds.
// This is important for multi-threaded async runtimes to ensure futures can be sent between threads.

#[cfg(feature = "std")]
use std::time::Duration;

use core::future::Future;

/// Trait representing the underlying async transport layer for DLMS/COSEM communication.
///
/// This trait allows the `AsyncDlmsClient` to be agnostic of the actual communication medium
/// (TCP, UDP, Serial, HDLC, etc.) and async runtime (Tokio, Glommio, Smol, Embassy, etc.).
///
/// Implementations should handle the low-level details of asynchronously sending and receiving bytes.
///
/// # Examples
///
/// ```rust,no_run
/// use dlms_cosem::async_transport::AsyncTransport;
///
/// #[derive(Debug)]
/// struct MyAsyncTransport;
///
/// #[derive(Debug)]
/// struct MyError;
///
/// impl AsyncTransport for MyAsyncTransport {
///     type Error = MyError;
///
///     async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
///         // Implementation-specific send logic
///         Ok(())
///     }
///
///     async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
///         // Implementation-specific receive logic
///         Ok(0)
///     }
/// }
/// ```
pub trait AsyncTransport: core::fmt::Debug + Send {
    /// The error type returned by async transport operations.
    type Error: core::fmt::Debug;

    /// Asynchronously sends data to the remote device.
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to send to the remote device.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the data was successfully sent.
    /// * `Err(Self::Error)` if an error occurred during transmission.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_transport::AsyncTransport;
    /// # async fn example<T: AsyncTransport>(transport: &mut T) -> Result<(), T::Error> {
    /// let data = &[0x01, 0x02, 0x03];
    /// transport.send(data).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Implementation Note
    ///
    /// This method returns `impl Future + Send` instead of being an `async fn` to
    /// explicitly require the returned future to be `Send`. This ensures the transport
    /// can be used safely across thread boundaries in multi-threaded async runtimes.
    fn send(&mut self, data: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Asynchronously receives data from the remote device.
    ///
    /// This function should populate the provided buffer with received data.
    /// Returns the number of bytes read.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A mutable buffer to store the received bytes.
    ///
    /// # Returns
    ///
    /// * `Ok(n)` where `n` is the number of bytes read into the buffer.
    /// * `Err(Self::Error)` if an error occurred during reception.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_transport::AsyncTransport;
    /// # async fn example<T: AsyncTransport>(transport: &mut T) -> Result<(), T::Error> {
    /// let mut buffer = [0u8; 1024];
    /// let n = transport.recv(&mut buffer).await?;
    /// println!("Received {} bytes", n);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Implementation Note
    ///
    /// This method returns `impl Future + Send` instead of being an `async fn` to
    /// explicitly require the returned future to be `Send`. This ensures the transport
    /// can be used safely across thread boundaries in multi-threaded async runtimes.
    fn recv(
        &mut self,
        buffer: &mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + Send;

    /// Asynchronously receives data from the remote device with a timeout.
    ///
    /// This function should populate the provided buffer with received data.
    /// Returns the number of bytes read, or an error if the timeout expires.
    ///
    /// Default implementation calls `recv` (no timeout support).
    /// Override this method to provide actual timeout functionality.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A mutable buffer to store the received bytes.
    /// * `timeout` - Maximum duration to wait for data.
    ///
    /// # Returns
    ///
    /// * `Ok(n)` where `n` is the number of bytes read into the buffer.
    /// * `Err(Self::Error)` if an error occurred or the timeout expired.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "std")]
    /// # use std::time::Duration;
    /// # use dlms_cosem::async_transport::AsyncTransport;
    /// # async fn example<T: AsyncTransport>(transport: &mut T) -> Result<(), T::Error> {
    /// let mut buffer = [0u8; 1024];
    /// let timeout = Duration::from_secs(5);
    /// let n = transport.recv_timeout(&mut buffer, timeout).await?;
    /// println!("Received {} bytes within timeout", n);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "std")]
    fn recv_timeout(
        &mut self,
        buffer: &mut [u8],
        _timeout: Duration,
    ) -> impl Future<Output = Result<usize, Self::Error>> + Send
    where
        Self: Send,
    {
        // Default implementation ignores timeout and calls recv
        // Override this method to provide actual timeout functionality
        self.recv(buffer)
    }
}
