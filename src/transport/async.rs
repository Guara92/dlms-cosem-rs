// We use `impl Future + Send` instead of `async fn` to be explicit about Send bounds.
// This is important for multi-threaded async runtimes to ensure futures can be sent between threads.
// Note: Glommio is an exception - it uses thread-per-core and is not Send.

#[cfg(feature = "std")]
use std::time::Duration;

use core::future::Future;

// ============================================================================
// Runtime Marker Traits - Conditional Send Bounds
// ============================================================================
//
// These marker traits implement the "MaybeSend" pattern used by Tokio and other
// runtime-agnostic libraries. They allow us to conditionally require Send based
// on the async runtime without duplicating the entire AsyncTransport trait.
//
// This approach is superior to trait duplication because:
// 1. Single source of truth for the AsyncTransport API
// 2. Runtime requirements are explicit via marker traits
// 3. Easy to extend for new runtime categories in the future
// 4. Zero breaking changes for library users

/// Marker trait that conditionally requires `Send` based on the async runtime category.
///
/// For multi-threaded runtimes (Tokio, Smol, Embassy), this trait requires `Send`
/// because futures may be moved between threads. For single-threaded runtimes
/// (Glommio, and future thread-per-core runtimes), this trait has no requirements.
///
/// # Why This Exists
///
/// Different async runtimes have different threading models:
/// - **Multi-threaded** (`rt-multi-thread`): Work-stealing schedulers that may
///   move tasks between threads. Requires `Send` for safety.
///   - Tokio, Smol, Embassy, tokio-uring
/// - **Single-threaded** (`rt-single-thread`): Tasks are pinned to a single thread
///   and never migrate. `Send` is not required and may not be implementable.
///   - Glommio, and future thread-per-core runtimes
///
/// This marker trait allows AsyncTransport to work with both models without
/// code duplication.
///
/// # Implementation Note
///
/// This trait uses umbrella features (`rt-multi-thread`, `rt-single-thread`) instead
/// of specific runtime features. This makes it easy to add new runtimes without
/// modifying the core trait definitions - just add the new runtime feature to the
/// appropriate category in Cargo.toml.
#[cfg(all(feature = "rt-multi-thread", not(feature = "rt-single-thread")))]
pub trait MaybeSend: Send {}

#[cfg(all(feature = "rt-multi-thread", not(feature = "rt-single-thread")))]
impl<T: Send> MaybeSend for T {}

/// Marker trait for single-threaded async runtimes.
///
/// This version has no `Send` requirement because single-threaded runtimes use
/// architectures where tasks are never moved between threads.
///
/// Activated by the `rt-single-thread` umbrella feature, which is automatically
/// enabled by runtime-specific features like `glommio`.
///
/// # Priority
///
/// When both `rt-multi-thread` and `rt-single-thread` are enabled (e.g., with
/// `--all-features`), `rt-single-thread` takes priority. This is the safer
/// choice as it doesn't require `Send`, making code compile in all scenarios.
#[cfg(feature = "rt-single-thread")]
pub trait MaybeSend {}

#[cfg(feature = "rt-single-thread")]
impl<T> MaybeSend for T {}

// ============================================================================
// AsyncTransport Trait
// ============================================================================

/// Trait representing the underlying async transport layer for DLMS/COSEM communication.
///
/// This trait allows the `AsyncDlmsClient` to be agnostic of the actual communication medium
/// (TCP, UDP, Serial, HDLC, etc.) and async runtime (Tokio, Smol, Embassy, Glommio).
///
/// Implementations should handle the low-level details of asynchronously sending and receiving bytes.
///
/// # Thread Safety
///
/// The trait uses the [`MaybeSend`] marker to conditionally require `Send`:
///
/// - **Multi-threaded runtimes** (Tokio, Smol, Embassy): Requires `Send` because futures
///   may be moved between threads by work-stealing schedulers.
/// - **Single-threaded runtimes** (Glommio): Does not require `Send` because tasks are
///   pinned to a single thread in a thread-per-core architecture.
///
/// This is handled automatically via feature flags - you don't need to worry about it
/// when implementing the trait.
///
/// # Supported Runtimes
///
/// | Runtime     | Threading Model    | Category           | Send Required | Platform       |
/// |-------------|-------------------|--------------------|---------------|----------------|
/// | Tokio       | Multi-threaded    | `rt-multi-thread`  | Yes           | Cross-platform |
/// | Smol        | Multi-threaded    | `rt-multi-thread`  | Yes           | Cross-platform |
/// | Embassy     | Single/Multi*     | `rt-multi-thread`  | Yes**         | Cross-platform |
/// | tokio-uring | Multi-threaded    | `rt-multi-thread`  | Yes           | Linux only     |
/// | Glommio     | Thread-per-core   | `rt-single-thread` | No            | Linux only     |
///
/// *Embassy can run in single or multi-threaded mode
/// **Embassy requires Send for std compatibility even in single-threaded mode
///
/// # Adding New Runtimes
///
/// To add support for a new runtime, simply add its feature to the appropriate
/// category in `Cargo.toml`:
///
/// - Multi-threaded: `new-runtime = ["async-client", "rt-multi-thread", "dep:new-runtime"]`
/// - Single-threaded: `new-runtime = ["async-client", "rt-single-thread", "dep:new-runtime"]`
///
/// No changes to this trait definition are needed!
///
/// # Examples
///
/// ## Multi-threaded Runtime (Tokio, Smol, Embassy, tokio-uring)
///
/// ```rust,no_run
/// # #[cfg(feature = "rt-multi-thread")]
/// # {
/// use dlms_cosem::transport::AsyncTransport;
///
/// #[derive(Debug)]
/// struct MyAsyncTransport;
///
/// #[derive(Debug)]
/// struct MyError;
///
/// // This implementation automatically gets Send requirement
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
///
///     #[cfg(feature = "std")]
///     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> {
///         self.recv(buffer).await
///     }
/// }
/// # }
/// ```
///
/// ## Single-threaded Runtime (Glommio, and future thread-per-core runtimes)
///
/// ```rust,no_run
/// # #[cfg(feature = "rt-single-thread")]
/// # {
/// use dlms_cosem::transport::AsyncTransport;
///
/// #[derive(Debug)]
/// struct MyGlommioTransport;
///
/// #[derive(Debug)]
/// struct MyError;
///
/// // This implementation does NOT require Send when using Glommio
/// impl AsyncTransport for MyGlommioTransport {
///     type Error = MyError;
///
///     async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
///         // Glommio-specific send logic (not Send)
///         Ok(())
///     }
///
///     async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
///         // Glommio-specific receive logic (not Send)
///         Ok(0)
///     }
///
///     #[cfg(feature = "std")]
///     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> {
///         self.recv(buffer).await
///     }
/// }
/// # }
/// ```
pub trait AsyncTransport: core::fmt::Debug + MaybeSend {
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
    /// # Implementation Note
    ///
    /// This method returns `impl Future + MaybeSend` to conditionally require `Send`
    /// based on the async runtime. For multi-threaded runtimes, the future must be
    /// `Send`. For Glommio, it does not need to be `Send`.
    fn send(&mut self, data: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend;

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
    /// # Implementation Note
    ///
    /// This method returns `impl Future + MaybeSend` to conditionally require `Send`
    /// based on the async runtime. For multi-threaded runtimes, the future must be
    /// `Send`. For Glommio, it does not need to be `Send`.
    fn recv(
        &mut self,
        buffer: &mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend;

    /// Asynchronously receives data from the remote device with a timeout.
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
    /// # Implementation Note
    ///
    /// This method returns `impl Future + MaybeSend` to conditionally require `Send`
    /// based on the async runtime. For multi-threaded runtimes, the future must be
    /// `Send`. For Glommio, it does not need to be `Send`.
    #[cfg(feature = "std")]
    fn recv_timeout(
        &mut self,
        buffer: &mut [u8],
        timeout: Duration,
    ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend;
}
