
extern crate alloc;

use crate::association::{AareApdu, AssociationResult, ReleaseResponseApdu, ReleaseResponseReason};
use crate::client::sync::{Buffer, ClientSettings, DlmsSession, SessionState};
use crate::client::{
    CLOCK_CLASS_ID, CLOCK_TIME_ATTRIBUTE_ID, PROFILE_GENERIC_BUFFER_ATTRIBUTE_ID,
    PROFILE_GENERIC_CLASS_ID,
};
use crate::transport::r#async::AsyncTransport;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::fmt::Debug;

#[cfg(feature = "heapless-buffer")]
use heapless::Vec as HeaplessVec;

#[cfg(feature = "encode")]
use crate::action::ActionRequest;
#[cfg(feature = "encode")]
use crate::get::{AccessSelector, AttributeDescriptor, GetRequest, GetRequestWithList};
#[cfg(feature = "encode")]
use crate::set::{SetRequest, SetRequestWithList};

#[cfg(feature = "parse")]
use crate::action::ActionResponse;
#[cfg(feature = "parse")]
use crate::get::{DataAccessResult, GetDataResult, GetResponse};
#[cfg(feature = "parse")]
use crate::set::SetResponse;

use crate::data::Data;
use crate::obis_code::ObisCode;

#[cfg(all(feature = "encode", feature = "parse"))]
use crate::data::DateTime;
#[cfg(all(feature = "encode", feature = "parse"))]
use crate::selective_access::{CaptureObjectDefinition, RangeDescriptor};

/// Errors that can occur during async client operations.
#[derive(Debug)]
pub enum AsyncClientError<E> {
    /// Error from the underlying async transport layer.
    TransportError(E),
    /// The connection was closed by the remote peer.
    ConnectionClosed,
    /// Error parsing the received data.
    ParseError,
    /// Error encoding the request.
    EncodeError,
    /// Association was rejected by the server.
    AssociationFailed(AssociationResult),
    /// Received a Release Response with a reason other than Normal.
    ReleaseRejected(ReleaseResponseReason),
    /// Client is not associated.
    NotAssociated,
    /// Data access error from the server.
    #[cfg(feature = "parse")]
    DataAccessError(DataAccessResult),
    /// Action error from the server.
    #[cfg(feature = "parse")]
    ActionError(crate::action::ActionResult),
    /// Unexpected response type.
    UnexpectedResponse,
    /// Invoke ID mismatch between request and response.
    InvokeIdMismatch,
    /// Invalid response data format.
    #[cfg(feature = "parse")]
    InvalidResponseData,
}

impl<E> From<E> for AsyncClientError<E> {
    fn from(e: E) -> Self {
        AsyncClientError::TransportError(e)
    }
}

impl<E: fmt::Display> fmt::Display for AsyncClientError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsyncClientError::TransportError(e) => write!(f, "Transport error: {}", e),
            AsyncClientError::ConnectionClosed => write!(f, "Connection closed"),
            AsyncClientError::ParseError => write!(f, "Parse error"),
            AsyncClientError::EncodeError => write!(f, "Encode error"),
            AsyncClientError::AssociationFailed(result) => {
                write!(f, "Association failed: {:?}", result)
            }
            AsyncClientError::ReleaseRejected(reason) => {
                write!(f, "Release rejected: {:?}", reason)
            }
            AsyncClientError::NotAssociated => write!(f, "Not associated"),
            #[cfg(feature = "parse")]
            AsyncClientError::DataAccessError(err) => write!(f, "Data access error: {:?}", err),
            #[cfg(feature = "parse")]
            AsyncClientError::ActionError(err) => write!(f, "Action error: {:?}", err),
            AsyncClientError::UnexpectedResponse => write!(f, "Unexpected response"),
            AsyncClientError::InvokeIdMismatch => write!(f, "Invoke ID mismatch"),
            #[cfg(feature = "parse")]
            AsyncClientError::InvalidResponseData => write!(f, "Invalid response data"),
        }
    }
}

#[cfg(feature = "std")]
impl<E: fmt::Debug + fmt::Display> std::error::Error for AsyncClientError<E> {}

/// Async DLMS Client for communicating with DLMS/COSEM devices.
///
/// This client provides async/await APIs for all DLMS operations, supporting
/// multiple async runtimes (Tokio, Glommio, Smol, Embassy) through the `AsyncTransport` trait.
///
/// The client reuses all protocol logic from `DlmsSession` to ensure consistency
/// with the synchronous client while providing a fully asynchronous interface.
///
/// # Type Parameters
///
/// * `T` - The async transport implementation (must implement `AsyncTransport`)
/// * `B` - The buffer type (`Vec<u8>` for heap allocation, `heapless::Vec<u8, N>` for stack)
///
/// # Examples
///
/// ## Using heap-allocated buffer (std):
/// ```rust,no_run
/// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
/// # use dlms_cosem::transport::r#async::AsyncTransport;
/// # use dlms_cosem::client::ClientSettings;
/// # #[derive(Debug)]
/// # struct TokioTcpTransport;
/// # impl AsyncTransport for TokioTcpTransport {
/// #     type Error = std::io::Error;
/// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
/// #     #[cfg(feature = "std")]
/// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
/// # }
/// let settings = ClientSettings::default();
/// let transport = TokioTcpTransport;
/// let mut client = AsyncClientBuilder::new(transport, settings)
///     .build_with_heap(2048);
/// ```
///
/// ## Using stack-allocated buffer (no_std):
/// ```rust,no_run
/// # #[cfg(feature = "heapless-buffer")]
/// # {
/// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
/// # use dlms_cosem::transport::r#async::AsyncTransport;
/// # use dlms_cosem::client::ClientSettings;
/// # #[derive(Debug)]
/// # struct EmbeddedTransport;
/// # impl AsyncTransport for EmbeddedTransport {
/// #     type Error = ();
/// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
/// #     #[cfg(feature = "std")]
/// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
/// # }
/// let settings = ClientSettings::default();
/// let transport = EmbeddedTransport;
/// let mut client = AsyncClientBuilder::new(transport, settings)
///     .build_with_heapless::<2048>();
/// # }
/// ```
#[derive(Debug)]
pub struct AsyncDlmsClient<T: AsyncTransport, B: Buffer> {
    transport: T,
    session: DlmsSession,
    buffer: B,
}

/// Builder for constructing an `AsyncDlmsClient` with flexible buffer allocation strategy.
///
/// This builder allows explicit choice between heap-allocated (`Vec<u8>`) and
/// stack-allocated (`heapless::Vec<u8, N>`) buffers, making the memory allocation
/// strategy clear at the call site.
///
/// # Examples
///
/// ## Heap-allocated buffer (std, runtime size):
/// ```no_run
/// use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
/// use dlms_cosem::transport::r#async::AsyncTransport;
/// use dlms_cosem::client::ClientSettings;
/// # #[derive(Debug)]
/// # struct MyTransport;
/// # impl AsyncTransport for MyTransport {
/// #     type Error = std::io::Error;
/// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
/// #     #[cfg(feature = "std")]
/// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
/// # }
///
/// let transport = MyTransport;
/// let settings = ClientSettings::default();
/// let client = AsyncClientBuilder::new(transport, settings)
///     .build_with_heap(4096);  // Runtime size decision
/// ```
///
/// ## Stack-allocated buffer (no_std, compile-time size):
/// ```no_run
/// # #[cfg(feature = "heapless-buffer")]
/// # {
/// use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
/// use dlms_cosem::transport::r#async::AsyncTransport;
/// use dlms_cosem::client::ClientSettings;
/// # #[derive(Debug)]
/// # struct MyTransport;
/// # impl AsyncTransport for MyTransport {
/// #     type Error = ();
/// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
/// #     #[cfg(feature = "std")]
/// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
/// # }
///
/// let transport = MyTransport;
/// let settings = ClientSettings::default();
/// let client = AsyncClientBuilder::new(transport, settings)
///     .build_with_heapless::<2048>();  // Compile-time size
/// # }
/// ```
#[derive(Debug)]
pub struct AsyncClientBuilder<T: AsyncTransport> {
    transport: T,
    settings: ClientSettings,
}

impl<T: AsyncTransport> AsyncClientBuilder<T> {
    /// Creates a new async client builder with the given transport and settings.
    pub fn new(transport: T, settings: ClientSettings) -> Self {
        Self { transport, settings }
    }

    /// Builds a client with a heap-allocated buffer of the specified runtime size.
    ///
    /// This is suitable for `std` environments where dynamic memory allocation is available.
    ///
    /// # Arguments
    /// * `buffer_size` - The size of the receive buffer in bytes (determined at runtime)
    ///
    /// # Recommended Sizes
    /// - **Minimal**: 256 bytes (simple read/write only)
    /// - **Standard**: 2048 bytes (handles most use cases)
    /// - **Load Profiles**: 4096-8192 bytes (block transfers)
    /// - **Maximum**: 65635 bytes (max PDU + overhead)
    pub fn build_with_heap(self, buffer_size: usize) -> AsyncDlmsClient<T, Vec<u8>> {
        AsyncDlmsClient {
            transport: self.transport,
            session: DlmsSession::new(self.settings),
            buffer: vec![0u8; buffer_size],
        }
    }

    /// Builds a client with a stack-allocated heapless buffer of compile-time size N.
    ///
    /// This is suitable for `no_std` embedded environments without a heap allocator.
    /// The buffer size N must be known at compile-time and will be allocated on the stack.
    ///
    /// # Type Parameters
    /// * `N` - The buffer size in bytes (const generic, determined at compile-time)
    ///
    /// # Panics
    /// Panics if N < 256 (minimum practical DLMS buffer size).
    ///
    /// # Note
    /// Large buffer sizes (>1024 bytes) may cause stack overflow on embedded systems.
    /// Consider using heap allocation for larger buffers if possible.
    #[cfg(feature = "heapless-buffer")]
    pub fn build_with_heapless<const N: usize>(self) -> AsyncDlmsClient<T, HeaplessVec<u8, N>> {
        assert!(N >= 256, "Buffer size must be at least 256 bytes for DLMS communication");

        let mut buffer = HeaplessVec::new();
        buffer.resize(N, 0).expect("Buffer initialization failed");

        AsyncDlmsClient {
            transport: self.transport,
            session: DlmsSession::new(self.settings),
            buffer,
        }
    }
}

impl<T: AsyncTransport, B: Buffer> AsyncDlmsClient<T, B> {
    /// Returns a reference to the underlying transport.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Returns a mutable reference to the underlying transport.
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Returns a reference to the current client settings.
    pub fn settings(&self) -> &ClientSettings {
        self.session.settings()
    }

    /// Returns a reference to the current session state.
    pub fn state(&self) -> &SessionState {
        self.session.state()
    }

    /// Returns a reference to the session.
    pub fn session(&self) -> &DlmsSession {
        &self.session
    }

    /// Establishes an association with the remote DLMS server.
    ///
    /// This method sends an AARQ (Association Request) and waits for an AARE
    /// (Association Response). If the association is accepted, the client state
    /// is updated to reflect the active connection.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if association was successful.
    /// * `Err(AsyncClientError)` if the association failed or a transport error occurred.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    ///     .build_with_heap(2048);
    /// client.connect().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn connect(&mut self) -> Result<(), AsyncClientError<T::Error>> {
        // Generate AARQ
        let aarq = self.session.generate_aarq();

        // Encode AARQ
        let request_buf = aarq.encode();

        // Send AARQ
        self.transport.send(&request_buf).await?;

        // Receive AARE
        let n = self.transport.recv(self.buffer.as_mut()).await?;
        if n == 0 {
            return Err(AsyncClientError::ConnectionClosed);
        }

        // Parse AARE
        let (_rem, aare) = AareApdu::parse(&self.buffer.as_ref()[..n])
            .map_err(|_| AsyncClientError::ParseError)?;

        // Handle AARE
        self.session.handle_aare(&aare).map_err(AsyncClientError::AssociationFailed)?;

        Ok(())
    }

    /// Closes the association with the remote DLMS server.
    ///
    /// This method sends an RLRQ (Release Request) and waits for an RLRE
    /// (Release Response). The client state is updated to reflect the disconnection.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the release was successful.
    /// * `Err(AsyncClientError)` if the release failed or a transport error occurred.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// client.disconnect().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn disconnect(&mut self) -> Result<(), AsyncClientError<T::Error>> {
        // Generate RLRQ
        let rlrq = self.session.generate_release_request();

        // Encode RLRQ
        let request_buf = rlrq.encode();

        // Send RLRQ
        self.transport.send(&request_buf).await?;

        // Receive RLRE
        let n = self.transport.recv(self.buffer.as_mut()).await?;
        if n == 0 {
            return Err(AsyncClientError::ConnectionClosed);
        }

        // Parse RLRE
        let (_rem, rlre) = ReleaseResponseApdu::parse(&self.buffer.as_ref()[..n])
            .map_err(|_| AsyncClientError::ParseError)?;

        // Handle RLRE
        self.session.handle_release_response(&rlre);

        // Check release reason
        if let Some(reason) = rlre.reason {
            if reason != ReleaseResponseReason::Normal {
                return Err(AsyncClientError::ReleaseRejected(reason));
            }
        }

        Ok(())
    }

    /// Reads an attribute from a COSEM object.
    ///
    /// This is the fundamental read operation in DLMS. It sends a GET-Request-Normal
    /// and processes the GET-Response.
    ///
    /// # Arguments
    ///
    /// * `class_id` - COSEM interface class ID (e.g., 3 for Register, 7 for ProfileGeneric).
    /// * `obis_code` - OBIS code identifying the object instance.
    /// * `attribute_id` - Attribute ID to read (e.g., 2 for value).
    /// * `access_selection` - Optional selective access parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(Data)` - The attribute value on success.
    /// * `Err(AsyncClientError)` - On transport, encoding, parsing, or DLMS errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::ObisCode;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// // Read active energy (1.0.1.8.0.255) attribute 2 (value)
    /// let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
    /// let data = client.read(3, obis, 2, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn read(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        attribute_id: i8,
        access_selection: Option<AccessSelector>,
    ) -> Result<Data, AsyncClientError<T::Error>> {
        if !self.session.state().associated {
            return Err(AsyncClientError::NotAssociated);
        }

        // Generate GET request
        let request =
            self.session.generate_get_request(class_id, obis_code, attribute_id, access_selection);
        let invoke_id = match &request {
            GetRequest::Normal(req) => req.invoke_id,
            _ => return Err(AsyncClientError::UnexpectedResponse),
        };

        // Encode request
        let request_buf = request.encode();

        // Send request
        self.transport.send(&request_buf).await?;

        // Receive response
        let n = self.transport.recv(self.buffer.as_mut()).await?;
        if n == 0 {
            return Err(AsyncClientError::ConnectionClosed);
        }

        // Parse response
        let (_rem, response) = GetResponse::parse(&self.buffer.as_ref()[..n])
            .map_err(|_| AsyncClientError::ParseError)?;

        // Handle response
        self.session
            .handle_get_response(response, invoke_id)
            .map_err(AsyncClientError::DataAccessError)
    }

    /// Writes a value to an attribute of a COSEM object.
    ///
    /// This sends a SET-Request-Normal and processes the SET-Response.
    ///
    /// # Arguments
    ///
    /// * `class_id` - COSEM interface class ID.
    /// * `obis_code` - OBIS code identifying the object instance.
    /// * `attribute_id` - Attribute ID to write.
    /// * `value` - The data value to write.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the write was successful.
    /// * `Err(AsyncClientError)` - On transport, encoding, parsing, or DLMS errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::ObisCode;
    /// # use dlms_cosem::Data;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// let obis = ObisCode::new(0, 0, 96, 1, 0, 255);
    /// let value = Data::Unsigned(42);
    /// client.write(1, obis, 2, value).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn write(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        attribute_id: i8,
        value: Data,
    ) -> Result<(), AsyncClientError<T::Error>> {
        if !self.session.state().associated {
            return Err(AsyncClientError::NotAssociated);
        }

        // Generate SET request
        let request =
            self.session.generate_set_request(class_id, obis_code, attribute_id, value, None);
        let invoke_id = match &request {
            SetRequest::Normal(req) => req.invoke_id,
            _ => return Err(AsyncClientError::UnexpectedResponse),
        };

        // Encode request
        let request_buf = request.encode();

        // Send request
        self.transport.send(&request_buf).await?;

        // Receive response
        let n = self.transport.recv(self.buffer.as_mut()).await?;
        if n == 0 {
            return Err(AsyncClientError::ConnectionClosed);
        }

        // Parse response
        let (_rem, response) = SetResponse::parse(&self.buffer.as_ref()[..n])
            .map_err(|_| AsyncClientError::ParseError)?;

        // Handle response
        self.session
            .handle_set_response(response, invoke_id)
            .map_err(AsyncClientError::DataAccessError)
    }

    /// Invokes a method on a COSEM object.
    ///
    /// This sends an ACTION-Request-Normal and processes the ACTION-Response.
    ///
    /// # Arguments
    ///
    /// * `class_id` - COSEM interface class ID.
    /// * `obis_code` - OBIS code identifying the object instance.
    /// * `method_id` - Method ID to invoke.
    /// * `parameters` - Optional method parameters.
    ///
    /// # Returns
    ///
    /// * `Ok(Option<Data>)` - Method return value (if any) on success.
    /// * `Err(AsyncClientError)` - On transport, encoding, parsing, or DLMS errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::ObisCode;
    /// # use dlms_cosem::Data;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// // Invoke method 1 (reset) on a register
    /// let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
    /// let result = client.method(3, obis, 1, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn method(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        method_id: i8,
        parameters: Option<Data>,
    ) -> Result<Option<Data>, AsyncClientError<T::Error>> {
        if !self.session.state().associated {
            return Err(AsyncClientError::NotAssociated);
        }

        // Generate ACTION request
        let request =
            self.session.generate_action_request(class_id, obis_code, method_id, parameters);
        let invoke_id = match &request {
            ActionRequest::Normal(req) => req.invoke_id,
            _ => return Err(AsyncClientError::UnexpectedResponse),
        };

        // Encode request
        let request_buf = request.encode();

        // Send request
        self.transport.send(&request_buf).await?;

        // Receive response
        let n = self.transport.recv(self.buffer.as_mut()).await?;
        if n == 0 {
            return Err(AsyncClientError::ConnectionClosed);
        }

        // Parse response
        let (_rem, response) = ActionResponse::parse(&self.buffer.as_ref()[..n])
            .map_err(|_| AsyncClientError::ParseError)?;

        // Handle response
        self.session
            .handle_action_response(response, invoke_id)
            .map_err(AsyncClientError::ActionError)
    }

    /// Reads multiple attributes in a single request using GET-Request-With-List.
    ///
    /// This is more efficient than multiple individual read operations when reading
    /// several attributes from the same or different objects.
    ///
    /// # Arguments
    ///
    /// * `descriptors` - Vector of attribute descriptors to read.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Result<Data, DataAccessResult>>)` - Results for each attribute.
    /// * `Err(AsyncClientError)` - On transport, encoding, or parsing errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::ObisCode;
    /// # use dlms_cosem::get::AttributeDescriptor;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// let descriptors = vec![
    ///     AttributeDescriptor {
    ///         class_id: 3,
    ///         instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
    ///         attribute_id: 2,
    ///
    ///     },
    ///     AttributeDescriptor {
    ///         class_id: 3,
    ///         instance_id: ObisCode::new(1, 0, 2, 8, 0, 255),
    ///         attribute_id: 2,
    ///
    ///     },
    /// ];
    /// let results = client.read_multiple(&descriptors).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn read_multiple(
        &mut self,
        descriptors: &[AttributeDescriptor],
    ) -> Result<Vec<Result<Data, DataAccessResult>>, AsyncClientError<T::Error>> {
        if !self.session.state().associated {
            return Err(AsyncClientError::NotAssociated);
        }

        if descriptors.is_empty() {
            return Ok(Vec::new());
        }

        let invoke_id = self.session.next_invoke_id();

        // Build GET-Request-With-List
        let attribute_descriptor_list: Vec<AttributeDescriptor> = descriptors.to_vec();
        let request =
            GetRequest::WithList(GetRequestWithList { invoke_id, attribute_descriptor_list });

        // Encode request
        let request_buf = request.encode();

        // Send request
        self.transport.send(&request_buf).await?;

        // Receive response
        let n = self.transport.recv(self.buffer.as_mut()).await?;
        if n == 0 {
            return Err(AsyncClientError::ConnectionClosed);
        }

        // Parse response
        let (_rem, response) = GetResponse::parse(&self.buffer.as_ref()[..n])
            .map_err(|_| AsyncClientError::ParseError)?;

        // Handle response
        match response {
            GetResponse::WithList(list_response) => {
                if list_response.invoke_id != invoke_id {
                    return Err(AsyncClientError::InvokeIdMismatch);
                }

                // Convert GetDataResult to Result<Data, DataAccessResult>
                let results: Vec<Result<Data, DataAccessResult>> = list_response
                    .results
                    .into_iter()
                    .map(|result| match result {
                        GetDataResult::Data(data) => Ok(data),
                        GetDataResult::DataAccessError(err) => Err(err),
                    })
                    .collect();
                Ok(results)
            }
            _ => Err(AsyncClientError::UnexpectedResponse),
        }
    }

    /// Writes multiple attributes in a single request using SET-Request-With-List.
    ///
    /// # Arguments
    ///
    /// * `writes` - Vector of (descriptor, value) pairs to write.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<DataAccessResult>)` - Write results for each attribute.
    /// * `Err(AsyncClientError)` - On transport, encoding, or parsing errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::ObisCode;
    /// # use dlms_cosem::Data;
    /// # use dlms_cosem::get::AttributeDescriptor;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// let writes = vec![
    ///     (
    ///         AttributeDescriptor {
    ///             class_id: 1,
    ///             instance_id: ObisCode::new(0, 0, 96, 1, 0, 255),
    ///             attribute_id: 2,
    ///
    ///         },
    ///         Data::Unsigned(42),
    ///     ),
    /// ];
    /// let results = client.write_multiple(&writes).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn write_multiple(
        &mut self,
        writes: &[(AttributeDescriptor, Data)],
    ) -> Result<Vec<DataAccessResult>, AsyncClientError<T::Error>> {
        if !self.session.state().associated {
            return Err(AsyncClientError::NotAssociated);
        }

        if writes.is_empty() {
            return Ok(Vec::new());
        }

        let invoke_id = self.session.next_invoke_id();

        // Build attribute descriptor list and data list
        let mut attribute_descriptor_list = Vec::new();
        let mut data_list = Vec::new();

        for (descriptor, value) in writes {
            attribute_descriptor_list.push(descriptor.clone());
            data_list.push(value.clone());
        }

        let request = SetRequest::WithList(SetRequestWithList {
            invoke_id,
            attribute_descriptor_list,
            value_list: data_list,
        });

        // Encode request
        let request_buf = request.encode();

        // Send request
        self.transport.send(&request_buf).await?;

        // Receive response
        let n = self.transport.recv(self.buffer.as_mut()).await?;
        if n == 0 {
            return Err(AsyncClientError::ConnectionClosed);
        }

        // Parse response
        let (_rem, response) = SetResponse::parse(&self.buffer.as_ref()[..n])
            .map_err(|_| AsyncClientError::ParseError)?;

        // Handle response
        match response {
            SetResponse::WithList(list_response) => {
                if list_response.invoke_id != invoke_id {
                    return Err(AsyncClientError::InvokeIdMismatch);
                }
                Ok(list_response.results)
            }
            _ => Err(AsyncClientError::UnexpectedResponse),
        }
    }

    /// Reads multiple attributes with automatic chunking.
    ///
    /// This method splits large read requests into smaller chunks based on
    /// `max_attributes_per_request` setting, making multiple requests as needed.
    ///
    /// # Arguments
    ///
    /// * `descriptors` - Vector of attribute descriptors to read.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Result<Data, DataAccessResult>>)` - Results for all attributes.
    /// * `Err(AsyncClientError)` - On the first error encountered.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::ObisCode;
    /// # use dlms_cosem::get::AttributeDescriptor;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// // Read 25 attributes - will be split into chunks of 10 (default)
    /// let mut descriptors = Vec::new();
    /// for i in 0..25 {
    ///     descriptors.push(AttributeDescriptor {
    ///         class_id: 3,
    ///         instance_id: ObisCode::new(1, 0, i as u8, 8, 0, 255),
    ///         attribute_id: 2,
    ///
    ///     });
    /// }
    /// let results = client.read_multiple_chunked(&descriptors).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn read_multiple_chunked(
        &mut self,
        descriptors: &[AttributeDescriptor],
    ) -> Result<Vec<Result<Data, DataAccessResult>>, AsyncClientError<T::Error>> {
        let max_per_request = self.session.settings().max_attributes_per_request;

        let mut all_results = Vec::new();

        if let Some(max) = max_per_request {
            // Chunked mode
            for chunk in descriptors.chunks(max) {
                let chunk_results = self.read_multiple(chunk).await?;
                all_results.extend(chunk_results);
            }
        } else {
            // No chunking
            all_results = self.read_multiple(descriptors).await?;
        }

        Ok(all_results)
    }

    /// Writes multiple attributes with automatic chunking.
    ///
    /// This method splits large write requests into smaller chunks based on
    /// `max_attributes_per_request` setting, making multiple requests as needed.
    ///
    /// # Arguments
    ///
    /// * `writes` - Vector of (descriptor, value) pairs to write.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<DataAccessResult>)` - Write results for all attributes.
    /// * `Err(AsyncClientError)` - On the first error encountered.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::ObisCode;
    /// # use dlms_cosem::Data;
    /// # use dlms_cosem::get::AttributeDescriptor;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// let mut writes = Vec::new();
    /// for i in 0..15 {
    ///     writes.push((
    ///         AttributeDescriptor {
    ///             class_id: 1,
    ///             instance_id: ObisCode::new(0, 0, i as u8, 1, 0, 255),
    ///             attribute_id: 2,
    ///
    ///         },
    ///         Data::DoubleLongUnsigned(i as u32),
    ///     ));
    /// }
    /// let results = client.write_multiple_chunked(&writes).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn write_multiple_chunked(
        &mut self,
        writes: &[(AttributeDescriptor, Data)],
    ) -> Result<Vec<DataAccessResult>, AsyncClientError<T::Error>> {
        let max_per_request = self.session.settings().max_attributes_per_request;

        let mut all_results = Vec::new();

        if let Some(max) = max_per_request {
            // Chunked mode
            for chunk in writes.chunks(max) {
                let chunk_results = self.write_multiple(chunk).await?;
                all_results.extend(chunk_results);
            }
        } else {
            // No chunking
            all_results = self.write_multiple(writes).await?;
        }

        Ok(all_results)
    }

    /// Reads a load profile (ProfileGeneric buffer) with optional date/time filtering.
    ///
    /// This is a convenience method for reading ProfileGeneric objects with
    /// automatic RangeDescriptor construction.
    ///
    /// # Arguments
    ///
    /// * `obis_code` - OBIS code of the ProfileGeneric object.
    /// * `from` - Optional start date/time.
    /// * `to` - Optional end date/time.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Vec<Data>>)` - Rows and columns of profile data.
    /// * `Err(AsyncClientError)` - On any error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::ObisCode;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// let obis = ObisCode::new(1, 0, 99, 1, 0, 255);
    /// let profile = client.read_load_profile(obis, None, None).await?;
    /// for row in profile {
    ///     println!("Row: {:?}", row);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn read_load_profile(
        &mut self,
        obis_code: ObisCode,
        from: Option<DateTime>,
        to: Option<DateTime>,
    ) -> Result<Vec<Vec<Data>>, AsyncClientError<T::Error>> {
        let access_selection = if from.is_some() || to.is_some() {
            let restricting_object = CaptureObjectDefinition {
                class_id: CLOCK_CLASS_ID,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: CLOCK_TIME_ATTRIBUTE_ID,
                data_index: 0,
            };

            let range_descriptor = RangeDescriptor {
                restricting_object,
                from_value: from.map(Data::DateTime).unwrap_or(Data::Null),
                to_value: to.map(Data::DateTime).unwrap_or(Data::Null),
                selected_values: Vec::new(), // All columns
            };

            Some(AccessSelector { selector: 1, parameters: range_descriptor.encode() })
        } else {
            None
        };

        let data = self
            .read(
                PROFILE_GENERIC_CLASS_ID,
                obis_code,
                PROFILE_GENERIC_BUFFER_ATTRIBUTE_ID,
                access_selection,
            )
            .await?;

        // Parse response - ProfileGeneric buffer is encoded as compact-array
        // which is parsed as Structure(Vec<Data>) where each element is Structure(row)
        match data {
            Data::Structure(rows) => {
                let mut result = Vec::new();
                for row in rows {
                    match row {
                        Data::Structure(columns) => result.push(columns),
                        _ => return Err(AsyncClientError::InvalidResponseData),
                    }
                }
                Ok(result)
            }
            _ => Err(AsyncClientError::InvalidResponseData),
        }
    }

    /// Reads the current time from a Clock object.
    ///
    /// This is a convenience method for reading the standard Clock object (0.0.1.0.0.255).
    ///
    /// # Returns
    ///
    /// * `Ok(DateTime)` - The current device time.
    /// * `Err(AsyncClientError)` - On any error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// let time = client.read_clock().await?;
    /// println!("Device time: {:?}", time);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn read_clock(&mut self) -> Result<DateTime, AsyncClientError<T::Error>> {
        let obis = ObisCode::new(0, 0, 1, 0, 0, 255);
        let data = self.read(CLOCK_CLASS_ID, obis, CLOCK_TIME_ATTRIBUTE_ID, None).await?;

        match data {
            Data::DateTime(dt) => Ok(dt),
            Data::OctetString(bytes) if bytes.len() == 12 => {
                let (_rem, dt) =
                    DateTime::parse(&bytes).map_err(|_| AsyncClientError::InvalidResponseData)?;
                Ok(dt)
            }
            _ => Err(AsyncClientError::InvalidResponseData),
        }
    }

    /// Sets the time on a Clock object.
    ///
    /// This is a convenience method for writing to the standard Clock object (0.0.1.0.0.255).
    ///
    /// # Arguments
    ///
    /// * `time` - The DateTime to set on the device.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the time was set successfully.
    /// * `Err(AsyncClientError)` - On any error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use dlms_cosem::async_client::{AsyncClientBuilder, AsyncClientError};
    /// # use dlms_cosem::transport::r#async::AsyncTransport;
    /// # use dlms_cosem::client::ClientSettings;
    /// # use dlms_cosem::{DateTime, Date, Time, ObisCode};
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl AsyncTransport for MyTransport {
    /// #     type Error = std::io::Error;
    /// #     async fn send(&mut self, _data: &[u8]) -> Result<(), Self::Error> { Ok(()) }
    /// #     async fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, Self::Error> { Ok(0) }
    /// #     #[cfg(feature = "std")]
    /// #     async fn recv_timeout(&mut self, buffer: &mut [u8], _timeout: std::time::Duration) -> Result<usize, Self::Error> { self.recv(buffer).await }
    /// # }
    /// # async fn example() -> Result<(), AsyncClientError<std::io::Error>> {
    /// # let mut client = AsyncClientBuilder::new(MyTransport, ClientSettings::default())
    /// #     .build_with_heap(2048);
    /// let date = Date::new(2025, 1, 30, 4);
    /// let time = Time::new(Some(14), Some(30), Some(0), Some(0));
    /// let datetime = DateTime::new(date, time, None, None);
    /// client.set_clock(datetime).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub async fn set_clock(&mut self, time: DateTime) -> Result<(), AsyncClientError<T::Error>> {
        let obis = ObisCode::new(0, 0, 1, 0, 0, 255);
        self.write(CLOCK_CLASS_ID, obis, CLOCK_TIME_ATTRIBUTE_ID, Data::DateTime(time)).await
    }
}

#[cfg(test)]
#[allow(clippy::manual_async_fn)]
mod tests {
    use super::*;

    use crate::association::{AareApdu, ApplicationContextName, AssociationResult};
    use crate::get::{
        AttributeDescriptor, DataAccessResult, GetDataResult, GetResponse, GetResponseNormal,
        GetResponseWithList,
    };
    use crate::set::{SetResponse, SetResponseNormal};
    use crate::transport::r#async::MaybeSend;
    use alloc::vec::Vec;
    use core::fmt;

    /// Mock async transport for testing
    #[derive(Debug)]
    struct MockAsyncTransport {
        response_queue: Vec<Vec<u8>>,
        current_response: usize,
    }

    #[derive(Debug)]
    struct MockTransportError;

    impl fmt::Display for MockTransportError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Mock transport error")
        }
    }

    impl MockAsyncTransport {
        fn new() -> Self {
            Self { response_queue: Vec::new(), current_response: 0 }
        }

        fn add_response(&mut self, response: Vec<u8>) {
            self.response_queue.push(response);
        }
    }

    // Unified AsyncTransport implementation using MaybeSend marker
    // This works for all runtimes (Tokio, Smol, Embassy, Glommio) without duplication
    impl AsyncTransport for MockAsyncTransport {
        type Error = MockTransportError;

        fn send(
            &mut self,
            _data: &[u8],
        ) -> impl Future<Output = Result<(), Self::Error>> + MaybeSend {
            async { Ok(()) }
        }

        fn recv(
            &mut self,
            buffer: &mut [u8],
        ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend {
            async move {
                if self.current_response >= self.response_queue.len() {
                    return Ok(0);
                }

                let response = &self.response_queue[self.current_response];
                let len = response.len().min(buffer.len());
                buffer[..len].copy_from_slice(&response[..len]);
                self.current_response += 1;
                Ok(len)
            }
        }

        #[cfg(feature = "std")]
        fn recv_timeout(
            &mut self,
            buffer: &mut [u8],
            _timeout: std::time::Duration,
        ) -> impl Future<Output = Result<usize, Self::Error>> + MaybeSend {
            // For mock transport, just call recv (ignore timeout)
            self.recv(buffer)
        }
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[tokio::test]
    async fn test_async_client_builder_heap() {
        let transport = MockAsyncTransport::new();
        let settings = ClientSettings::default();
        let _client = AsyncClientBuilder::new(transport, settings).build_with_heap(2048);
    }

    #[cfg(all(feature = "encode", feature = "parse", feature = "heapless-buffer"))]
    #[tokio::test]
    async fn test_async_client_builder_heapless() {
        let transport = MockAsyncTransport::new();
        let settings = ClientSettings::default();
        let _client = AsyncClientBuilder::new(transport, settings).build_with_heapless::<2048>();
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[tokio::test]
    async fn test_async_client_connect_success() {
        let mut transport = MockAsyncTransport::new();
        let settings = ClientSettings::default();

        let aare = AareApdu {
            protocol_version: 0,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: crate::association::AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };

        transport.add_response(aare.encode());
        let mut client = AsyncClientBuilder::new(transport, settings).build_with_heap(2048);
        let result = client.connect().await;

        assert!(result.is_ok());
        assert!(client.state().associated);
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[tokio::test]
    async fn test_async_client_read_success() {
        let mut transport = MockAsyncTransport::new();
        let settings = ClientSettings::default();

        let aare = AareApdu {
            protocol_version: 0,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: crate::association::AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.add_response(aare.encode());

        let get_response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0,
            result: GetDataResult::Data(Data::DoubleLongUnsigned(12345)),
        });
        transport.add_response(get_response.encode());

        let mut client = AsyncClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().await.unwrap();

        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let result = client.read(3, obis, 2, None).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Data::DoubleLongUnsigned(12345));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[tokio::test]
    async fn test_async_client_read_not_associated() {
        let transport = MockAsyncTransport::new();
        let settings = ClientSettings::default();
        let mut client = AsyncClientBuilder::new(transport, settings).build_with_heap(2048);

        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let result = client.read(3, obis, 2, None).await;

        assert!(matches!(result, Err(AsyncClientError::NotAssociated)));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[tokio::test]
    async fn test_async_client_write_success() {
        let mut transport = MockAsyncTransport::new();
        let settings = ClientSettings::default();

        let aare = AareApdu {
            protocol_version: 0,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: crate::association::AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.add_response(aare.encode());

        let set_response = SetResponse::Normal(SetResponseNormal {
            invoke_id: 0,
            result: DataAccessResult::Success,
        });
        transport.add_response(set_response.encode());

        let mut client = AsyncClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().await.unwrap();

        let obis = ObisCode::new(0, 0, 96, 1, 0, 255);
        let result = client.write(1, obis, 2, Data::Unsigned(42)).await;

        assert!(result.is_ok());
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[tokio::test]
    async fn test_async_client_read_multiple_success() {
        let mut transport = MockAsyncTransport::new();
        let settings = ClientSettings::default();

        let aare = AareApdu {
            protocol_version: 0,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: crate::association::AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.add_response(aare.encode());

        let get_response = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: alloc::vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(100)),
                GetDataResult::Data(Data::DoubleLongUnsigned(200)),
            ],
        });
        transport.add_response(get_response.encode());

        let mut client = AsyncClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().await.unwrap();

        let descriptors = alloc::vec![
            AttributeDescriptor {
                class_id: 3,
                instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
                attribute_id: 2,
            },
            AttributeDescriptor {
                class_id: 3,
                instance_id: ObisCode::new(1, 0, 2, 8, 0, 255),
                attribute_id: 2,
            },
        ];

        let result = client.read_multiple(&descriptors).await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], Ok(Data::DoubleLongUnsigned(100)));
        assert_eq!(results[1], Ok(Data::DoubleLongUnsigned(200)));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[tokio::test]
    async fn test_async_client_read_multiple_chunked() {
        let mut transport = MockAsyncTransport::new();
        let settings = ClientSettings { max_attributes_per_request: Some(2), ..Default::default() };

        let aare = AareApdu {
            protocol_version: 0,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: crate::association::AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.add_response(aare.encode());

        let response1 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: alloc::vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(100)),
                GetDataResult::Data(Data::DoubleLongUnsigned(200)),
            ],
        });
        transport.add_response(response1.encode());

        let response2 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 1,
            results: alloc::vec![GetDataResult::Data(Data::DoubleLongUnsigned(300))],
        });
        transport.add_response(response2.encode());

        let mut client = AsyncClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().await.unwrap();

        let descriptors = alloc::vec![
            AttributeDescriptor {
                class_id: 3,
                instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
                attribute_id: 2,
            },
            AttributeDescriptor {
                class_id: 3,
                instance_id: ObisCode::new(1, 0, 2, 8, 0, 255),
                attribute_id: 2,
            },
            AttributeDescriptor {
                class_id: 3,
                instance_id: ObisCode::new(1, 0, 3, 8, 0, 255),
                attribute_id: 2,
            },
        ];

        let result = client.read_multiple_chunked(&descriptors).await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], Ok(Data::DoubleLongUnsigned(100)));
        assert_eq!(results[1], Ok(Data::DoubleLongUnsigned(200)));
        assert_eq!(results[2], Ok(Data::DoubleLongUnsigned(300)));
    }
}
