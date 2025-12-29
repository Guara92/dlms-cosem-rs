
extern crate alloc;

use crate::association::{
    AareApdu, AarqApdu, ApplicationContextName, AssociationResult, AuthenticationValue,
    MechanismName, ReleaseRequestApdu, ReleaseRequestReason, ReleaseResponseApdu,
    ReleaseResponseReason,
};
use crate::client::{
    CLOCK_CLASS_ID, CLOCK_TIME_ATTRIBUTE_ID, DEFAULT_MAX_ATTRIBUTES_PER_REQUEST,
    PROFILE_GENERIC_BUFFER_ATTRIBUTE_ID, PROFILE_GENERIC_CLASS_ID,
};
use crate::transport::sync::Transport;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use core::fmt::Debug;

#[cfg(feature = "encode")]
use crate::action::{ActionRequest, ActionRequestNormal};
#[cfg(feature = "encode")]
use crate::get::{
    AccessSelector, AttributeDescriptor, GetRequest, GetRequestNormal, GetRequestWithList,
};
#[cfg(feature = "encode")]
use crate::set::{SetRequest, SetRequestNormal, SetRequestWithList};

#[cfg(feature = "parse")]
use crate::action::{ActionResponse, ActionResponseNormal};
#[cfg(feature = "parse")]
use crate::get::{DataAccessResult, GetDataResult, GetResponse, GetResponseNormal};
#[cfg(feature = "parse")]
use crate::set::{SetResponse, SetResponseNormal};

use crate::data::Data;
use crate::obis_code::ObisCode;

#[cfg(all(feature = "encode", feature = "parse"))]
use crate::data::DateTime;
#[cfg(all(feature = "encode", feature = "parse"))]
use crate::selective_access::{CaptureObjectDefinition, RangeDescriptor};

#[cfg(feature = "heapless-buffer")]
use heapless::Vec as HeaplessVec;

/// Errors that can occur during client operations.
#[derive(Debug)]
pub enum ClientError<E> {
    /// Error from the underlying transport layer.
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

impl<E> From<E> for ClientError<E> {
    fn from(e: E) -> Self {
        ClientError::TransportError(e)
    }
}

impl<E: fmt::Display> fmt::Display for ClientError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::TransportError(e) => write!(f, "Transport error: {}", e),
            ClientError::ConnectionClosed => write!(f, "Connection closed"),
            ClientError::ParseError => write!(f, "Parse error"),
            ClientError::EncodeError => write!(f, "Encode error"),
            ClientError::AssociationFailed(result) => write!(f, "Association failed: {:?}", result),
            ClientError::ReleaseRejected(reason) => write!(f, "Release rejected: {:?}", reason),
            ClientError::NotAssociated => write!(f, "Not associated"),
            #[cfg(feature = "parse")]
            ClientError::DataAccessError(err) => write!(f, "Data access error: {:?}", err),
            #[cfg(feature = "parse")]
            ClientError::ActionError(err) => write!(f, "Action error: {:?}", err),
            ClientError::UnexpectedResponse => write!(f, "Unexpected response"),
            ClientError::InvokeIdMismatch => write!(f, "Invoke ID mismatch"),
            #[cfg(feature = "parse")]
            ClientError::InvalidResponseData => write!(f, "Invalid response data"),
        }
    }
}

#[cfg(feature = "std")]
impl<E: fmt::Debug + fmt::Display> std::error::Error for ClientError<E> {}

/// Settings for the DLMS client.
#[derive(Debug, Clone)]
pub struct ClientSettings {
    /// Client SAP (Service Access Point).
    /// Default: 16 (Public Client).
    pub client_address: u8,
    /// Server SAP (Service Access Point).
    /// Default: 1 (Management Logical Device).
    pub server_address: u16,
    /// Authentication mechanism to use.
    /// Default: LowestLevelSecurity (No security).
    pub authentication_mechanism: MechanismName,
    /// Authentication secret (password or key).
    /// Default: None.
    pub authentication_value: Option<Vec<u8>>,
    /// Application Context Name.
    /// Default: LogicalNameReferencing.
    pub application_context_name: ApplicationContextName,
    /// Maximum PDU size the client can receive.
    /// Default: 0xFFFF (65535).
    pub max_pdu_size: u16,
    /// Maximum number of attributes per request for bulk operations.
    /// Used by chunked read/write methods to split large requests.
    /// Default: Some(10) for Gurux compatibility.
    /// Set to None for no limit.
    pub max_attributes_per_request: Option<usize>,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            client_address: 16, // Public Client
            server_address: 1,  // Management Logical Device
            authentication_mechanism: MechanismName::LowestLevelSecurity,
            authentication_value: None,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            max_pdu_size: 0xFFFF,
            max_attributes_per_request: Some(DEFAULT_MAX_ATTRIBUTES_PER_REQUEST),
        }
    }
}

/// Internal state of the client session.
#[derive(Debug, Clone, Default)]
pub struct SessionState {
    /// Whether the client is currently associated with the server.
    pub associated: bool,
    /// Negotiated maximum PDU size.
    pub negotiated_max_pdu_size: u16,
    /// Negotiated conformance block.
    pub negotiated_conformance: Option<Vec<u8>>,
    /// Association Result from the last AARE.
    pub association_result: Option<AssociationResult>,
    /// Association Diagnostic from the last AARE.
    pub association_diagnostic: Option<u8>,
}

/// The logic core of the DLMS Client.
///
/// This struct handles the state machine and PDU generation/processing.
/// It is decoupled from the transport layer to allow usage in both sync and async contexts.
#[derive(Debug)]
pub struct DlmsSession {
    settings: ClientSettings,
    state: SessionState,
    invoke_id: u8,
}

impl DlmsSession {
    /// Creates a new session with the given settings.
    pub fn new(settings: ClientSettings) -> Self {
        Self { settings, state: SessionState::default(), invoke_id: 0 }
    }

    /// Returns the current settings.
    pub fn settings(&self) -> &ClientSettings {
        &self.settings
    }

    /// Returns the current session state.
    pub fn state(&self) -> &SessionState {
        &self.state
    }

    /// Generates an AARQ APDU for association.
    pub fn generate_aarq(&self) -> AarqApdu {
        let calling_auth_value = self
            .settings
            .authentication_value
            .as_ref()
            .map(|secret| AuthenticationValue::CharString(secret.clone()));

        // Basic AARQ construction
        let mut aarq = AarqApdu::new_simple_ln(self.settings.max_pdu_size);
        aarq.application_context_name = self.settings.application_context_name;
        aarq.mechanism_name = Some(self.settings.authentication_mechanism);
        aarq.calling_authentication_value = calling_auth_value;
        aarq
    }

    /// Processes an AARE APDU and updates the session state.
    pub fn handle_aare(&mut self, aare: &AareApdu) -> Result<(), AssociationResult> {
        self.state.association_result = Some(aare.result);
        self.state.association_diagnostic = Some(aare.result_source_diagnostic.as_u8());

        if aare.result == AssociationResult::Accepted {
            self.state.associated = true;

            if let Some(user_info) = &aare.user_information {
                self.state.negotiated_max_pdu_size = user_info.server_max_receive_pdu_size;
                self.state.negotiated_conformance =
                    Some(user_info.negotiated_conformance.to_bytes().to_vec());
            }

            Ok(())
        } else {
            self.state.associated = false;
            Err(aare.result)
        }
    }

    /// Generates a Release Request APDU.
    pub fn generate_release_request(&self) -> ReleaseRequestApdu {
        ReleaseRequestApdu { reason: Some(ReleaseRequestReason::Normal), user_information: None }
    }

    /// Processes a Release Response APDU.
    pub fn handle_release_response(&mut self, _rlre: &ReleaseResponseApdu) {
        self.state.associated = false;
    }

    /// Generates the next invoke ID.
    pub fn next_invoke_id(&mut self) -> u8 {
        let id = self.invoke_id;
        self.invoke_id = self.invoke_id.wrapping_add(1);
        id
    }

    /// Generates a GET-Request-Normal APDU.
    ///
    /// This is the low-level request generation. Use `DlmsClient::read()` for the complete workflow.
    #[cfg(feature = "encode")]
    pub fn generate_get_request(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        attribute_id: i8,
        access_selection: Option<AccessSelector>,
    ) -> GetRequest {
        let invoke_id = self.next_invoke_id();
        GetRequest::Normal(GetRequestNormal {
            invoke_id,
            class_id,
            instance_id: obis_code,
            attribute_id,
            access_selection,
        })
    }

    /// Processes a GET-Response and extracts the data.
    #[cfg(feature = "parse")]
    pub fn handle_get_response(
        &self,
        response: GetResponse,
        expected_invoke_id: u8,
    ) -> Result<Data, DataAccessResult> {
        match response {
            GetResponse::Normal(GetResponseNormal { invoke_id, result }) => {
                if invoke_id != expected_invoke_id {
                    // In a real implementation, we'd return InvokeIdMismatch error
                    // For now, we proceed but this should be validated
                }
                match result {
                    GetDataResult::Data(data) => Ok(data),
                    GetDataResult::DataAccessError(err) => Err(err),
                }
            }
            _ => {
                // Block transfer not yet supported
                Err(DataAccessResult::OtherReason)
            }
        }
    }

    /// Generates a SET-Request-Normal APDU.
    ///
    /// This is the low-level request generation. Use `DlmsClient::write()` for the complete workflow.
    #[cfg(feature = "encode")]
    pub fn generate_set_request(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        attribute_id: i8,
        value: Data,
        access_selection: Option<AccessSelector>,
    ) -> SetRequest {
        let invoke_id = self.next_invoke_id();
        SetRequest::Normal(SetRequestNormal {
            invoke_id,
            class_id,
            instance_id: obis_code,
            attribute_id,
            access_selection,
            value,
        })
    }

    /// Processes a SET-Response and extracts the result.
    #[cfg(feature = "parse")]
    pub fn handle_set_response(
        &self,
        response: SetResponse,
        expected_invoke_id: u8,
    ) -> Result<(), DataAccessResult> {
        match response {
            SetResponse::Normal(SetResponseNormal { invoke_id, result }) => {
                if invoke_id != expected_invoke_id {
                    // Should validate invoke_id match
                }
                match result {
                    DataAccessResult::Success => Ok(()),
                    err => Err(err),
                }
            }
            _ => {
                // Block transfer not yet supported
                Err(DataAccessResult::OtherReason)
            }
        }
    }

    /// Generates an ACTION-Request-Normal APDU.
    ///
    /// This is the low-level request generation. Use `DlmsClient::method()` for the complete workflow.
    #[cfg(feature = "encode")]
    pub fn generate_action_request(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        method_id: i8,
        parameters: Option<Data>,
    ) -> ActionRequest {
        let invoke_id = self.next_invoke_id();
        ActionRequest::Normal(ActionRequestNormal {
            invoke_id,
            class_id,
            instance_id: obis_code,
            method_id,
            method_invocation_parameters: parameters,
        })
    }

    /// Processes an ACTION-Response and extracts the result.
    #[cfg(feature = "parse")]
    pub fn handle_action_response(
        &self,
        response: ActionResponse,
        expected_invoke_id: u8,
    ) -> Result<Option<Data>, crate::action::ActionResult> {
        match response {
            ActionResponse::Normal(ActionResponseNormal { invoke_id, result }) => {
                if invoke_id != expected_invoke_id {
                    // Should validate invoke_id match
                }
                match result {
                    crate::action::ActionResult::Success(opt_result) => {
                        // Extract Data from GetDataResult
                        match opt_result {
                            Some(crate::action::GetDataResult::Data(data)) => Ok(Some(data)),
                            Some(crate::action::GetDataResult::DataAccessError(_err)) => {
                                // Convert DataAccessError to ActionResult error
                                Err(crate::action::ActionResult::OtherReason)
                            }
                            None => Ok(None),
                        }
                    }
                    err => Err(err),
                }
            }
            _ => {
                // Block transfer not yet supported
                Err(crate::action::ActionResult::OtherReason)
            }
        }
    }
}

/// Trait for types that can be used as buffers in the DLMS client.
///
/// This trait allows the client to work with different buffer implementations
/// (heap-allocated `Vec`, stack-allocated `heapless::Vec`, etc.).
pub trait Buffer: AsMut<[u8]> + AsRef<[u8]> + Debug {
    /// Returns the capacity of the buffer.
    fn capacity(&self) -> usize;

    /// Returns the current length of valid data in the buffer.
    fn len(&self) -> usize;

    /// Returns true if the buffer contains no valid data.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resizes the buffer to the specified length, filling with the given value.
    fn resize(&mut self, new_len: usize, value: u8);
}

/// Buffer implementation for heap-allocated `Vec<u8>`.
impl Buffer for Vec<u8> {
    fn capacity(&self) -> usize {
        Vec::capacity(self)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn resize(&mut self, new_len: usize, value: u8) {
        Vec::resize(self, new_len, value)
    }
}

/// Buffer implementation for stack-allocated `heapless::Vec<u8, N>`.
#[cfg(feature = "heapless-buffer")]
impl<const N: usize> Buffer for HeaplessVec<u8, N> {
    fn capacity(&self) -> usize {
        N
    }

    fn len(&self) -> usize {
        self.as_slice().len()
    }

    fn resize(&mut self, new_len: usize, value: u8) {
        heapless::Vec::resize(self, new_len, value)
            .expect("Buffer resize failed - capacity exceeded")
    }
}

/// The DLMS Client.
///
/// This struct manages the connection and communication with a DLMS server.
/// It is generic over the transport layer and the buffer type, allowing flexibility
/// in both I/O implementation and memory allocation strategy.
///
/// # Type Parameters
/// - `T`: The transport implementation (TCP, Serial, HDLC, etc.)
/// - `B`: The buffer type (`Vec<u8>` for heap allocation, `heapless::Vec<u8, N>` for stack)
///
/// # Examples
///
/// ## Using heap-allocated buffer (std):
/// ```no_run
/// use dlms_cosem::client::{ClientBuilder, ClientSettings};
/// # #[derive(Debug)]
/// # struct MyTransport;
/// # impl dlms_cosem::transport::Transport for MyTransport {
/// #     type Error = ();
/// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
/// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
/// # }
///
/// let transport = MyTransport;
/// let settings = ClientSettings::default();
/// let mut client = ClientBuilder::new(transport, settings)
///     .build_with_heap(2048);
/// ```
///
/// ## Using stack-allocated buffer (no_std):
/// ```no_run
/// # #[cfg(feature = "heapless-buffer")]
/// # {
/// use dlms_cosem::client::{ClientBuilder, ClientSettings};
/// # #[derive(Debug)]
/// # struct MyTransport;
/// # impl dlms_cosem::transport::Transport for MyTransport {
/// #     type Error = ();
/// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
/// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
/// # }
///
/// let transport = MyTransport;
/// let settings = ClientSettings::default();
/// let mut client = ClientBuilder::new(transport, settings)
///     .build_with_heapless::<2048>();
/// # }
/// ```
#[derive(Debug)]
pub struct DlmsClient<T: Transport, B: Buffer> {
    transport: T,
    session: DlmsSession,
    buffer: B,
}

/// Builder for constructing a `DlmsClient` with flexible buffer allocation strategy.
///
/// This builder allows explicit choice between heap-allocated (`Vec<u8>`) and
/// stack-allocated (`heapless::Vec<u8, N>`) buffers, making the memory allocation
/// strategy clear at the call site.
///
/// # Examples
///
/// ## Heap-allocated buffer (std, runtime size):
/// ```no_run
/// use dlms_cosem::client::{ClientBuilder, ClientSettings};
/// # #[derive(Debug)]
/// # struct MyTransport;
/// # impl dlms_cosem::transport::Transport for MyTransport {
/// #     type Error = ();
/// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
/// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
/// # }
///
/// let transport = MyTransport;
/// let settings = ClientSettings::default();
/// let client = ClientBuilder::new(transport, settings)
///     .build_with_heap(4096);  // Runtime size decision
/// ```
///
/// ## Stack-allocated buffer (no_std, compile-time size):
/// ```no_run
/// # #[cfg(feature = "heapless-buffer")]
/// # {
/// use dlms_cosem::client::{ClientBuilder, ClientSettings};
/// # #[derive(Debug)]
/// # struct MyTransport;
/// # impl dlms_cosem::transport::Transport for MyTransport {
/// #     type Error = ();
/// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
/// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
/// # }
///
/// let transport = MyTransport;
/// let settings = ClientSettings::default();
/// let client = ClientBuilder::new(transport, settings)
///     .build_with_heapless::<2048>();  // Compile-time size
/// # }
/// ```
#[derive(Debug)]
pub struct ClientBuilder<T: Transport> {
    transport: T,
    settings: ClientSettings,
}

impl<T: Transport> ClientBuilder<T> {
    /// Creates a new client builder with the given transport and settings.
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
    pub fn build_with_heap(self, buffer_size: usize) -> DlmsClient<T, Vec<u8>> {
        DlmsClient {
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
    pub fn build_with_heapless<const N: usize>(self) -> DlmsClient<T, HeaplessVec<u8, N>> {
        assert!(N >= 256, "Buffer size must be at least 256 bytes for DLMS communication");

        let mut buffer = HeaplessVec::new();
        buffer.resize(N, 0).expect("Buffer initialization failed");

        DlmsClient { transport: self.transport, session: DlmsSession::new(self.settings), buffer }
    }
}

impl<T: Transport, B: Buffer> DlmsClient<T, B> {
    /// Returns a reference to the underlying transport.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Returns a mutable reference to the underlying transport.
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Returns the current session logic handler.
    pub fn session(&self) -> &DlmsSession {
        &self.session
    }

    /// Connects to the DLMS server (Association).
    ///
    /// Sends an AARQ and expects an AARE.
    pub fn connect(&mut self) -> Result<(), ClientError<T::Error>> {
        let aarq = self.session.generate_aarq();
        let encoded = aarq.encode();

        self.transport.send(&encoded).map_err(ClientError::TransportError)?;

        let bytes_read =
            self.transport.recv(self.buffer.as_mut()).map_err(ClientError::TransportError)?;
        if bytes_read == 0 {
            return Err(ClientError::ConnectionClosed);
        }

        let (_rem, aare) = AareApdu::parse(&self.buffer.as_ref()[..bytes_read])
            .map_err(|_| ClientError::ParseError)?;

        self.session.handle_aare(&aare).map_err(ClientError::AssociationFailed)?;

        Ok(())
    }

    /// Disconnects from the DLMS server.
    ///
    /// Sends a Release Request and expects a Release Response.
    pub fn disconnect(&mut self) -> Result<(), ClientError<T::Error>> {
        if !self.session.state.associated {
            return Ok(());
        }

        let rlrq = self.session.generate_release_request();
        let encoded = rlrq.encode();

        self.transport.send(&encoded).map_err(ClientError::TransportError)?;

        let bytes_read =
            self.transport.recv(self.buffer.as_mut()).map_err(ClientError::TransportError)?;
        if bytes_read == 0 {
            // If connection closed, we are disconnected anyway.
            self.session.state.associated = false;
            return Ok(());
        }

        let (_rem, rlre) = ReleaseResponseApdu::parse(&self.buffer.as_ref()[..bytes_read])
            .map_err(|_| ClientError::ParseError)?;

        self.session.handle_release_response(&rlre);

        Ok(())
    }

    /// Reads a single COSEM attribute (GET service).
    ///
    /// This is the high-level wrapper for GET-Request-Normal.
    /// Equivalent to Gurux `cl_read()`.
    ///
    /// # Parameters
    /// - `class_id`: COSEM interface class ID
    /// - `obis_code`: Logical name (OBIS code)
    /// - `attribute_id`: Attribute index (1-based)
    /// - `access_selection`: Optional selective access descriptor
    ///
    /// # Returns
    /// The attribute data on success, or an error.
    ///
    /// # Examples
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::ObisCode;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// // Read Register value (class 3, attribute 2)
    /// let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
    /// let value = client.read(3, obis, 2, None);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn read(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        attribute_id: i8,
        access_selection: Option<AccessSelector>,
    ) -> Result<Data, ClientError<T::Error>> {
        if !self.session.state.associated {
            return Err(ClientError::NotAssociated);
        }

        let request =
            self.session.generate_get_request(class_id, obis_code, attribute_id, access_selection);
        let invoke_id = match &request {
            GetRequest::Normal(n) => n.invoke_id,
            _ => 0,
        };

        let encoded = request.encode();
        self.transport.send(&encoded).map_err(ClientError::TransportError)?;

        let bytes_read =
            self.transport.recv(self.buffer.as_mut()).map_err(ClientError::TransportError)?;
        if bytes_read == 0 {
            return Err(ClientError::ConnectionClosed);
        }

        let (_rem, response) = GetResponse::parse(&self.buffer.as_ref()[..bytes_read])
            .map_err(|_| ClientError::ParseError)?;

        self.session.handle_get_response(response, invoke_id).map_err(ClientError::DataAccessError)
    }

    /// Writes a single COSEM attribute (SET service).
    ///
    /// This is the high-level wrapper for SET-Request-Normal.
    /// Equivalent to Gurux `cl_write()`.
    ///
    /// # Parameters
    /// - `class_id`: COSEM interface class ID
    /// - `obis_code`: Logical name (OBIS code)
    /// - `attribute_id`: Attribute index (1-based)
    /// - `value`: Data to write
    /// - `access_selection`: Optional selective access descriptor
    ///
    /// # Returns
    /// `Ok(())` on success, or an error.
    ///
    /// # Examples
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::{ObisCode, Data};
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// // Write Register value (class 3, attribute 2)
    /// let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
    /// let value = Data::DoubleLongUnsigned(12345);
    /// let result = client.write(3, obis, 2, value, None);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn write(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        attribute_id: i8,
        value: Data,
        access_selection: Option<AccessSelector>,
    ) -> Result<(), ClientError<T::Error>> {
        if !self.session.state.associated {
            return Err(ClientError::NotAssociated);
        }

        let request = self.session.generate_set_request(
            class_id,
            obis_code,
            attribute_id,
            value,
            access_selection,
        );
        let invoke_id = match &request {
            SetRequest::Normal(n) => n.invoke_id,
            _ => 0,
        };

        let encoded = request.encode();
        self.transport.send(&encoded).map_err(ClientError::TransportError)?;

        let bytes_read =
            self.transport.recv(self.buffer.as_mut()).map_err(ClientError::TransportError)?;
        if bytes_read == 0 {
            return Err(ClientError::ConnectionClosed);
        }

        let (_rem, response) = SetResponse::parse(&self.buffer.as_ref()[..bytes_read])
            .map_err(|_| ClientError::ParseError)?;

        self.session.handle_set_response(response, invoke_id).map_err(ClientError::DataAccessError)
    }

    /// Invokes a COSEM method (ACTION service).
    ///
    /// This is the high-level wrapper for ACTION-Request-Normal.
    /// Equivalent to Gurux `cl_method()`.
    ///
    /// # Parameters
    /// - `class_id`: COSEM interface class ID
    /// - `obis_code`: Logical name (OBIS code)
    /// - `method_id`: Method index (1-based)
    /// - `parameters`: Optional method invocation parameters
    ///
    /// # Returns
    /// Optional return data on success, or an error.
    ///
    /// # Examples
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::{ObisCode, Data};
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// // Invoke Clock.adjust_to_quarter (class 8, method 1)
    /// let obis = ObisCode::new(0, 0, 1, 0, 0, 255);
    /// let result = client.method(8, obis, 1, None);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn method(
        &mut self,
        class_id: u16,
        obis_code: ObisCode,
        method_id: i8,
        parameters: Option<Data>,
    ) -> Result<Option<Data>, ClientError<T::Error>> {
        if !self.session.state.associated {
            return Err(ClientError::NotAssociated);
        }

        let request =
            self.session.generate_action_request(class_id, obis_code, method_id, parameters);
        let invoke_id = match &request {
            ActionRequest::Normal(n) => n.invoke_id,
            _ => 0,
        };

        let encoded = request.encode();
        self.transport.send(&encoded).map_err(ClientError::TransportError)?;

        let bytes_read =
            self.transport.recv(self.buffer.as_mut()).map_err(ClientError::TransportError)?;
        if bytes_read == 0 {
            return Err(ClientError::ConnectionClosed);
        }

        let (_rem, response) = ActionResponse::parse(&self.buffer.as_ref()[..bytes_read])
            .map_err(|_| ClientError::ParseError)?;

        self.session.handle_action_response(response, invoke_id).map_err(ClientError::ActionError)
    }

    /// Read multiple attributes in a single request (GET-Request-With-List).
    ///
    /// This method allows efficient bulk reading of multiple COSEM attributes.
    /// All attributes are read in a single request/response exchange.
    ///
    /// # Arguments
    ///
    /// * `requests` - Slice of tuples (class_id, obis_code, attribute_id)
    ///
    /// # Returns
    ///
    /// Vector of results, one per request. Each element is either:
    /// - `Ok(Data)` - Successfully read attribute value
    /// - `Err(DataAccessResult)` - Error accessing that specific attribute
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if:
    /// - Not associated with the server
    /// - Transport error occurs
    /// - Response cannot be parsed
    /// - Invoke ID mismatch
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::ObisCode;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// // Read multiple register values
    /// let requests = [
    ///     (3, ObisCode::new(1, 0, 1, 8, 0, 255), 2), // Active energy
    ///     (3, ObisCode::new(1, 0, 2, 8, 0, 255), 2), // Reactive energy
    ///     (3, ObisCode::new(1, 0, 3, 8, 0, 255), 2), // Apparent energy
    /// ];
    /// let results = client.read_multiple(&requests);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn read_multiple(
        &mut self,
        requests: &[(u16, ObisCode, i8)],
    ) -> Result<Vec<Result<Data, DataAccessResult>>, ClientError<T::Error>> {
        if !self.session.state.associated {
            return Err(ClientError::NotAssociated);
        }

        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let invoke_id = self.session.next_invoke_id();

        // Build attribute descriptor list
        let attribute_descriptor_list: Vec<AttributeDescriptor> = requests
            .iter()
            .map(|(class_id, obis_code, attribute_id)| AttributeDescriptor {
                class_id: *class_id,
                instance_id: *obis_code,
                attribute_id: *attribute_id,
            })
            .collect();

        let request =
            GetRequest::WithList(GetRequestWithList { invoke_id, attribute_descriptor_list });

        let encoded = request.encode();
        self.transport.send(&encoded).map_err(ClientError::TransportError)?;

        let bytes_read =
            self.transport.recv(self.buffer.as_mut()).map_err(ClientError::TransportError)?;
        if bytes_read == 0 {
            return Err(ClientError::ConnectionClosed);
        }

        let (_rem, response) = GetResponse::parse(&self.buffer.as_ref()[..bytes_read])
            .map_err(|_| ClientError::ParseError)?;

        match response {
            GetResponse::WithList(list_response) => {
                if list_response.invoke_id != invoke_id {
                    return Err(ClientError::InvokeIdMismatch);
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
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Write multiple attributes in a single request (SET-Request-With-List).
    ///
    /// This method allows efficient bulk writing of multiple COSEM attributes.
    /// All attributes are written in a single request/response exchange.
    ///
    /// # Arguments
    ///
    /// * `requests` - Slice of tuples (class_id, obis_code, attribute_id, value)
    ///
    /// # Returns
    ///
    /// Vector of `DataAccessResult`, one per request. Each element indicates
    /// success or the specific error for that write operation.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if:
    /// - Not associated with the server
    /// - Transport error occurs
    /// - Response cannot be parsed
    /// - Invoke ID mismatch
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::{ObisCode, Data};
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// // Write multiple data objects
    /// let requests = [
    ///     (1, ObisCode::new(0, 0, 96, 1, 0, 255), 2, Data::Unsigned(10)),
    ///     (1, ObisCode::new(0, 0, 96, 1, 1, 255), 2, Data::Unsigned(20)),
    /// ];
    /// let results = client.write_multiple(&requests);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn write_multiple(
        &mut self,
        requests: &[(u16, ObisCode, i8, Data)],
    ) -> Result<Vec<DataAccessResult>, ClientError<T::Error>> {
        if !self.session.state.associated {
            return Err(ClientError::NotAssociated);
        }

        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let invoke_id = self.session.next_invoke_id();

        // Build attribute descriptor list and value list
        let mut attribute_descriptor_list = Vec::new();
        let mut value_list = Vec::new();

        for (class_id, obis_code, attribute_id, value) in requests {
            attribute_descriptor_list.push(AttributeDescriptor {
                class_id: *class_id,
                instance_id: *obis_code,
                attribute_id: *attribute_id,
            });
            value_list.push(value.clone());
        }

        let request = SetRequest::WithList(SetRequestWithList {
            invoke_id,
            attribute_descriptor_list,
            value_list,
        });

        let encoded = request.encode();
        self.transport.send(&encoded).map_err(ClientError::TransportError)?;

        let bytes_read =
            self.transport.recv(self.buffer.as_mut()).map_err(ClientError::TransportError)?;
        if bytes_read == 0 {
            return Err(ClientError::ConnectionClosed);
        }

        let (_rem, response) = SetResponse::parse(&self.buffer.as_ref()[..bytes_read])
            .map_err(|_| ClientError::ParseError)?;

        match response {
            SetResponse::WithList(list_response) => {
                if list_response.invoke_id != invoke_id {
                    return Err(ClientError::InvokeIdMismatch);
                }

                Ok(list_response.results)
            }
            _ => Err(ClientError::UnexpectedResponse),
        }
    }

    /// Read ProfileGeneric buffer with date/time range filtering.
    ///
    /// This is a convenience method for reading load profile data within a specific
    /// time range. It automatically constructs the RangeDescriptor for selective access.
    ///
    /// # Arguments
    ///
    /// * `obis_code` - OBIS code of the ProfileGeneric object
    /// * `from` - Start date/time (inclusive)
    /// * `to` - End date/time (inclusive)
    ///
    /// # Returns
    ///
    /// Vector of rows, where each row is a vector of Data values (columns).
    /// The first column is typically the timestamp (DateTime).
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if:
    /// - Not associated with the server
    /// - Transport error occurs
    /// - Response is not an Array of Structures
    /// - Data access error from server
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::{ObisCode, DateTime, Date, Time};
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// // Read last 24 hours of load profile data
    /// let obis = ObisCode::new(1, 0, 99, 1, 0, 255);
    /// let from = DateTime::new(
    ///     Date::new(2025, 1, 29, 0xFF),
    ///     Time::new(Some(0), Some(0), Some(0), None),
    ///     None,
    ///     None
    /// );
    /// let to = DateTime::new(
    ///     Date::new(2025, 1, 30, 0xFF),
    ///     Time::new(Some(0), Some(0), Some(0), None),
    ///     None,
    ///     None
    /// );
    /// let profile_data = client.read_load_profile(obis, from, to);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn read_load_profile(
        &mut self,
        obis_code: ObisCode,
        from: DateTime,
        to: DateTime,
    ) -> Result<Vec<Vec<Data>>, ClientError<T::Error>> {
        // Construct RangeDescriptor for DateTime filtering
        // The restricting object is typically the Clock column (first column)
        let range_descriptor = RangeDescriptor {
            restricting_object: CaptureObjectDefinition {
                class_id: CLOCK_CLASS_ID,
                logical_name: ObisCode::new(0, 0, 1, 0, 0, 255),
                attribute_index: CLOCK_TIME_ATTRIBUTE_ID,
                data_index: 0,
            },
            from_value: Data::DateTime(from),
            to_value: Data::DateTime(to),
            selected_values: Vec::new(), // All columns
        };

        let access_selector = AccessSelector { selector: 1, parameters: range_descriptor.encode() };

        // Read ProfileGeneric.buffer (attribute 2) with selective access
        let data = self.read(
            PROFILE_GENERIC_CLASS_ID,
            obis_code,
            PROFILE_GENERIC_BUFFER_ATTRIBUTE_ID,
            Some(access_selector),
        )?;

        // Parse the response - ProfileGeneric buffer is encoded as compact-array
        // which is parsed as Structure(Vec<Data>) where each element is Structure(row)
        match data {
            Data::Structure(rows) => {
                let mut result = Vec::new();
                for row in rows {
                    match row {
                        Data::Structure(columns) => {
                            result.push(columns);
                        }
                        _ => return Err(ClientError::InvalidResponseData),
                    }
                }
                Ok(result)
            }
            _ => Err(ClientError::InvalidResponseData),
        }
    }

    /// Read current time from the Clock object.
    ///
    /// This is a convenience method that reads the standard Clock object
    /// at OBIS code 0.0.1.0.0.255, attribute 2 (time).
    ///
    /// # Returns
    ///
    /// Current DateTime value from the meter's clock.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if:
    /// - Not associated with the server
    /// - Transport error occurs
    /// - Response is not a DateTime
    /// - Data access error from server
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// let current_time = client.read_clock();
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn read_clock(&mut self) -> Result<DateTime, ClientError<T::Error>> {
        let data = self.read(
            CLOCK_CLASS_ID,
            ObisCode::new(0, 0, 1, 0, 0, 255),
            CLOCK_TIME_ATTRIBUTE_ID,
            None,
        )?;

        match data {
            Data::DateTime(dt) => Ok(dt),
            _ => Err(ClientError::InvalidResponseData),
        }
    }

    /// Set the clock time on the meter.
    ///
    /// This is a convenience method that writes to the standard Clock object
    /// at OBIS code 0.0.1.0.0.255, attribute 2 (time).
    ///
    /// # Arguments
    ///
    /// * `time` - DateTime value to set
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if:
    /// - Not associated with the server
    /// - Transport error occurs
    /// - Data access error from server (e.g., read-only, permission denied)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::{DateTime, Date, Time};
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// let new_time = DateTime::new(
    ///     Date::new(2025, 1, 30, 0xFF),
    ///     Time::new(Some(12), Some(0), Some(0), None),
    ///     None,
    ///     None
    /// );
    /// client.set_clock(new_time);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn set_clock(&mut self, time: DateTime) -> Result<(), ClientError<T::Error>> {
        self.write(
            CLOCK_CLASS_ID,
            ObisCode::new(0, 0, 1, 0, 0, 255),
            CLOCK_TIME_ATTRIBUTE_ID,
            Data::DateTime(time),
            None,
        )?;
        Ok(())
    }

    /// Read multiple attributes with automatic chunking.
    ///
    /// This method splits large bulk read operations into smaller chunks based on
    /// the maximum attributes per request. This is useful when:
    /// - The server has PDU size limitations
    /// - Compatibility with devices that reject large requests
    /// - Following Gurux DLMS.c defaults (10 attributes per request)
    ///
    /// # Arguments
    ///
    /// * `requests` - Slice of tuples (class_id, obis_code, attribute_id)
    /// * `max_per_request` - Optional override for max attributes per request.
    ///   If None, uses the value from ClientSettings. If that's also None,
    ///   sends all attributes in a single request.
    ///
    /// # Returns
    ///
    /// Vector of Results, one per request. Each element is either:
    /// - `Ok(Data)` - Successfully read data
    /// - `Err(DataAccessResult)` - Error for that specific attribute
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if:
    /// - Not associated with the server
    /// - Transport error occurs in any chunk
    /// - Response cannot be parsed
    /// - Invoke ID mismatch
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::ObisCode;
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// // Read 25 registers - will be split into 3 chunks (10+10+5)
    /// let mut requests = Vec::new();
    /// for i in 0..25 {
    ///     requests.push((3, ObisCode::new(1, 0, i, 8, 0, 255), 2));
    /// }
    /// let results = client.read_multiple_chunked(&requests, None);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn read_multiple_chunked(
        &mut self,
        requests: &[(u16, ObisCode, i8)],
        max_per_request: Option<usize>,
    ) -> Result<Vec<Result<Data, DataAccessResult>>, ClientError<T::Error>> {
        if !self.session.state.associated {
            return Err(ClientError::NotAssociated);
        }

        if requests.is_empty() {
            return Ok(Vec::new());
        }

        // Determine chunk size
        let chunk_size = max_per_request
            .or(self.session.settings.max_attributes_per_request)
            .unwrap_or(requests.len()); // No limit if both are None

        let mut all_results = Vec::with_capacity(requests.len());

        // Process requests in chunks
        for chunk in requests.chunks(chunk_size) {
            let chunk_results = self.read_multiple(chunk)?;
            all_results.extend(chunk_results);
        }

        Ok(all_results)
    }

    /// Write multiple attributes with automatic chunking.
    ///
    /// This method splits large bulk write operations into smaller chunks based on
    /// the maximum attributes per request. This ensures compatibility with devices
    /// that have PDU size limitations or reject large requests.
    ///
    /// # Arguments
    ///
    /// * `requests` - Slice of tuples (class_id, obis_code, attribute_id, value)
    /// * `max_per_request` - Optional override for max attributes per request.
    ///   If None, uses the value from ClientSettings. If that's also None,
    ///   sends all attributes in a single request.
    ///
    /// # Returns
    ///
    /// Vector of `DataAccessResult`, one per request. Each element indicates
    /// success or the specific error for that write operation.
    ///
    /// # Errors
    ///
    /// Returns `ClientError` if:
    /// - Not associated with the server
    /// - Transport error occurs in any chunk
    /// - Response cannot be parsed
    /// - Invoke ID mismatch
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use dlms_cosem::client::{ClientBuilder, ClientSettings};
    /// # use dlms_cosem::{ObisCode, Data};
    /// # #[derive(Debug)]
    /// # struct MyTransport;
    /// # impl dlms_cosem::transport::Transport for MyTransport {
    /// #     type Error = ();
    /// #     fn send(&mut self, _data: &[u8]) -> Result<(), ()> { Ok(()) }
    /// #     fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, ()> { Ok(0) }
    /// # }
    /// # let transport = MyTransport;
    /// # let settings = ClientSettings::default();
    /// # let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
    /// // Write 15 data objects - will be split into 2 chunks (10+5)
    /// let mut requests = Vec::new();
    /// for i in 0..15 {
    ///     requests.push((1, ObisCode::new(0, 0, 96, 1, i, 255), 2, Data::Unsigned(i as u8)));
    /// }
    /// let results = client.write_multiple_chunked(&requests, None);
    /// ```
    #[cfg(all(feature = "encode", feature = "parse"))]
    pub fn write_multiple_chunked(
        &mut self,
        requests: &[(u16, ObisCode, i8, Data)],
        max_per_request: Option<usize>,
    ) -> Result<Vec<DataAccessResult>, ClientError<T::Error>> {
        if !self.session.state.associated {
            return Err(ClientError::NotAssociated);
        }

        if requests.is_empty() {
            return Ok(Vec::new());
        }

        // Determine chunk size
        let chunk_size = max_per_request
            .or(self.session.settings.max_attributes_per_request)
            .unwrap_or(requests.len()); // No limit if both are None

        let mut all_results = Vec::with_capacity(requests.len());

        // Process requests in chunks
        for chunk in requests.chunks(chunk_size) {
            let chunk_results = self.write_multiple(chunk)?;
            all_results.extend(chunk_results);
        }

        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::association::AcseServiceUserDiagnostics;
    use alloc::vec::Vec;
    use core::cell::RefCell;

    #[derive(Debug)]
    struct MockTransport {
        sent_data: RefCell<Vec<Vec<u8>>>,
        response_queue: RefCell<Vec<Vec<u8>>>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self { sent_data: RefCell::new(Vec::new()), response_queue: RefCell::new(Vec::new()) }
        }

        fn push_response(&self, data: Vec<u8>) {
            self.response_queue.borrow_mut().push(data);
        }
    }

    impl Transport for MockTransport {
        type Error = ();

        fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            self.sent_data.borrow_mut().push(data.to_vec());
            Ok(())
        }

        fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
            let mut queue = self.response_queue.borrow_mut();
            if queue.is_empty() {
                return Ok(0);
            }
            let response = queue.remove(0);
            let len = core::cmp::min(buffer.len(), response.len());
            buffer[..len].copy_from_slice(&response[..len]);
            Ok(len)
        }
    }

    #[test]
    fn test_client_builder_heap() {
        let transport = MockTransport::new();
        let settings = ClientSettings::default();
        let client = ClientBuilder::new(transport, settings).build_with_heap(8192);

        assert_eq!(client.buffer.len(), 8192);
        assert_eq!(client.session().settings().client_address, 16);
        assert!(!client.session().state().associated);
    }

    #[test]
    fn test_client_builder_heap_connect() {
        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        let result = client.connect();

        assert!(result.is_ok());
        assert!(client.session().state().associated);
    }

    #[test]
    fn test_handle_aare_with_negotiated_parameters() {
        use crate::association::{Conformance, InitiateResponse};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();
        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);

        let negotiated_conformance = Conformance::GET | Conformance::SET;
        let initiate_resp = InitiateResponse::new_ln(negotiated_conformance, 1024);
        let aare =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);

        // Directly call handle_aare on the session
        let result = client.session.handle_aare(&aare);

        assert!(result.is_ok());
        assert!(client.session().state().associated);
        assert_eq!(client.session().state().negotiated_max_pdu_size, 1024);
        assert_eq!(
            client.session().state().negotiated_conformance,
            Some(negotiated_conformance.to_bytes().to_vec())
        );
    }

    #[cfg(feature = "heapless-buffer")]
    #[test]
    fn test_client_builder_heapless() {
        let transport = MockTransport::new();
        let settings = ClientSettings::default();
        // Use Box to avoid stack overflow in tests
        let client: Box<DlmsClient<MockTransport, heapless::Vec<u8, 1024>>> =
            Box::new(ClientBuilder::new(transport, settings).build_with_heapless::<1024>());

        assert_eq!(client.buffer.len(), 1024);
        assert_eq!(client.session().settings().client_address, 16);
        assert!(!client.session().state().associated);
    }

    #[cfg(feature = "heapless-buffer")]
    #[test]
    fn test_heapless_client_connect_success() {
        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        // Use Box to avoid stack overflow in tests
        let mut client: Box<DlmsClient<MockTransport, heapless::Vec<u8, 1024>>> =
            Box::new(ClientBuilder::new(transport, settings).build_with_heapless::<1024>());
        let result = client.connect();

        assert!(result.is_ok());
        assert!(client.session().state().associated);
    }

    #[cfg(feature = "heapless-buffer")]
    #[test]
    #[should_panic(expected = "Buffer size must be at least 256 bytes")]
    fn test_client_builder_heapless_panics_on_small_buffer() {
        let transport = MockTransport::new();
        let settings = ClientSettings::default();
        // This will panic during construction
        let _client: Box<DlmsClient<MockTransport, heapless::Vec<u8, 128>>> =
            Box::new(ClientBuilder::new(transport, settings).build_with_heapless::<128>());
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_client_read_success() {
        use crate::data::Data;
        use crate::get::{GetDataResult, GetResponse, GetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // First, connect
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare GET response
        let get_response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0,
            result: GetDataResult::Data(Data::DoubleLongUnsigned(12345)),
        });
        client.transport.push_response(get_response.encode());

        // Execute read
        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let result = client.read(3, obis, 2, None);

        assert!(result.is_ok());
        match result.unwrap() {
            Data::DoubleLongUnsigned(val) => assert_eq!(val, 12345),
            _ => panic!("Unexpected data type"),
        }
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_client_read_not_associated() {
        let transport = MockTransport::new();
        let settings = ClientSettings::default();
        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);

        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let result = client.read(3, obis, 2, None);

        assert!(matches!(result, Err(ClientError::NotAssociated)));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_client_read_data_access_error() {
        use crate::get::{GetDataResult, GetResponse, GetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare error response
        let get_response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0,
            result: GetDataResult::DataAccessError(DataAccessResult::ObjectUndefined),
        });
        client.transport.push_response(get_response.encode());

        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let result = client.read(3, obis, 2, None);

        assert!(matches!(
            result,
            Err(ClientError::DataAccessError(DataAccessResult::ObjectUndefined))
        ));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_client_write_success() {
        use crate::data::Data;
        use crate::set::{SetResponse, SetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare SET response
        let set_response = SetResponse::Normal(SetResponseNormal {
            invoke_id: 0,
            result: DataAccessResult::Success,
        });
        client.transport.push_response(set_response.encode());

        // Execute write
        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let value = Data::DoubleLongUnsigned(54321);
        let result = client.write(3, obis, 2, value, None);

        assert!(result.is_ok());
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_client_write_error() {
        use crate::data::Data;
        use crate::set::{SetResponse, SetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare error response
        let set_response = SetResponse::Normal(SetResponseNormal {
            invoke_id: 0,
            result: DataAccessResult::ReadWriteDenied,
        });
        client.transport.push_response(set_response.encode());

        let obis = ObisCode::new(1, 0, 1, 8, 0, 255);
        let value = Data::DoubleLongUnsigned(54321);
        let result = client.write(3, obis, 2, value, None);

        assert!(matches!(
            result,
            Err(ClientError::DataAccessError(DataAccessResult::ReadWriteDenied))
        ));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_client_method_success() {
        use crate::action::{ActionResponse, ActionResponseNormal, ActionResult};
        use crate::data::Data;

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare ACTION response with return data
        let action_response = ActionResponse::Normal(ActionResponseNormal {
            invoke_id: 0,
            result: ActionResult::Success(Some(crate::action::GetDataResult::Data(
                Data::Unsigned(42),
            ))),
        });
        client.transport.push_response(action_response.encode());

        // Execute method
        let obis = ObisCode::new(0, 0, 1, 0, 0, 255);
        let result = client.method(8, obis, 1, None);

        assert!(result.is_ok());
        match result.unwrap() {
            Some(Data::Unsigned(val)) => assert_eq!(val, 42),
            _ => panic!("Unexpected return data"),
        }
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_client_method_no_return_data() {
        use crate::action::{ActionResponse, ActionResponseNormal, ActionResult};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare ACTION response without return data
        let action_response = ActionResponse::Normal(ActionResponseNormal {
            invoke_id: 0,
            result: ActionResult::Success(None),
        });
        client.transport.push_response(action_response.encode());

        let obis = ObisCode::new(0, 0, 1, 0, 0, 255);
        let result = client.method(8, obis, 1, None);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_client_method_error() {
        use crate::action::{ActionResponse, ActionResponseNormal, ActionResult};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare error response
        let action_response = ActionResponse::Normal(ActionResponseNormal {
            invoke_id: 0,
            result: ActionResult::ObjectUndefined,
        });
        client.transport.push_response(action_response.encode());

        let obis = ObisCode::new(0, 0, 1, 0, 0, 255);
        let result = client.method(8, obis, 1, None);

        assert!(matches!(result, Err(ClientError::ActionError(ActionResult::ObjectUndefined))));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_multiple_success() {
        use crate::get::{GetDataResult, GetResponse, GetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare GET-Response-With-List
        let response = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(12345)),
                GetDataResult::Data(Data::DoubleLongUnsigned(67890)),
                GetDataResult::Data(Data::DoubleLongUnsigned(11111)),
            ],
        });
        client.transport.push_response(response.encode());

        let requests = [
            (3, ObisCode::new(1, 0, 1, 8, 0, 255), 2),
            (3, ObisCode::new(1, 0, 2, 8, 0, 255), 2),
            (3, ObisCode::new(1, 0, 3, 8, 0, 255), 2),
        ];
        let results = client.read_multiple(&requests);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 3);
        assert!(matches!(results[0], Ok(Data::DoubleLongUnsigned(12345))));
        assert!(matches!(results[1], Ok(Data::DoubleLongUnsigned(67890))));
        assert!(matches!(results[2], Ok(Data::DoubleLongUnsigned(11111))));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_multiple_with_errors() {
        use crate::get::{GetDataResult, GetResponse, GetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare response with mixed success/error
        let response = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(12345)),
                GetDataResult::DataAccessError(DataAccessResult::ObjectUndefined),
                GetDataResult::Data(Data::DoubleLongUnsigned(11111)),
            ],
        });
        client.transport.push_response(response.encode());

        let requests = [
            (3, ObisCode::new(1, 0, 1, 8, 0, 255), 2),
            (3, ObisCode::new(1, 0, 2, 8, 0, 255), 2),
            (3, ObisCode::new(1, 0, 3, 8, 0, 255), 2),
        ];
        let results = client.read_multiple(&requests);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 3);
        assert!(matches!(results[0], Ok(Data::DoubleLongUnsigned(12345))));
        assert!(matches!(results[1], Err(DataAccessResult::ObjectUndefined)));
        assert!(matches!(results[2], Ok(Data::DoubleLongUnsigned(11111))));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_multiple_empty() {
        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        let requests = [];
        let results = client.read_multiple(&requests);

        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_write_multiple_success() {
        use crate::set::{SetResponse, SetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare SET-Response-With-List (all success)
        let response = SetResponse::WithList(SetResponseWithList {
            invoke_id: 0,
            results: vec![DataAccessResult::Success, DataAccessResult::Success],
        });
        client.transport.push_response(response.encode());

        let requests = [
            (1, ObisCode::new(0, 0, 96, 1, 0, 255), 2, Data::Unsigned(10)),
            (1, ObisCode::new(0, 0, 96, 1, 1, 255), 2, Data::Unsigned(20)),
        ];
        let results = client.write_multiple(&requests);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], DataAccessResult::Success);
        assert_eq!(results[1], DataAccessResult::Success);
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_write_multiple_with_errors() {
        use crate::set::{SetResponse, SetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare response with mixed results
        let response = SetResponse::WithList(SetResponseWithList {
            invoke_id: 0,
            results: vec![DataAccessResult::Success, DataAccessResult::ReadWriteDenied],
        });
        client.transport.push_response(response.encode());

        let requests = [
            (1, ObisCode::new(0, 0, 96, 1, 0, 255), 2, Data::Unsigned(10)),
            (1, ObisCode::new(0, 0, 96, 1, 1, 255), 2, Data::Unsigned(20)),
        ];
        let results = client.write_multiple(&requests);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], DataAccessResult::Success);
        assert_eq!(results[1], DataAccessResult::ReadWriteDenied);
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_clock_success() {
        use crate::get::{GetDataResult, GetResponse, GetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare GET response with DateTime
        let test_date = crate::data::Date::new(2025, 1, 30, 0xFF);
        let test_time = crate::data::Time::new(Some(12), Some(0), Some(0), None);
        let test_dt = DateTime::new(test_date, test_time, None, None);
        let response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0,
            result: GetDataResult::Data(Data::DateTime(test_dt)),
        });
        client.transport.push_response(response.encode());

        let result = client.read_clock();

        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.date.year, 2025);
        assert_eq!(dt.date.month, 1);
        assert_eq!(dt.date.day_of_month, 30);
        assert_eq!(dt.time.hour, Some(12));
        assert_eq!(dt.time.minute, Some(0));
        assert_eq!(dt.time.second, Some(0));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_clock_invalid_data_type() {
        use crate::get::{GetDataResult, GetResponse, GetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare GET response with wrong data type
        let response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0,
            result: GetDataResult::Data(Data::DoubleLongUnsigned(12345)),
        });
        client.transport.push_response(response.encode());

        let result = client.read_clock();

        assert!(matches!(result, Err(ClientError::InvalidResponseData)));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_set_clock_success() {
        use crate::set::{SetResponse, SetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare SET response (success)
        let response = SetResponse::Normal(SetResponseNormal {
            invoke_id: 0,
            result: DataAccessResult::Success,
        });
        client.transport.push_response(response.encode());

        let test_date = crate::data::Date::new(2025, 1, 30, 0xFF);
        let test_time = crate::data::Time::new(Some(12), Some(0), Some(0), None);
        let new_time = DateTime::new(test_date, test_time, None, None);
        let result = client.set_clock(new_time);

        assert!(result.is_ok());
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_load_profile_success() {
        use crate::get::{GetDataResult, GetResponse, GetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare ProfileGeneric buffer response (compact-array of structures)
        let date1 = crate::data::Date::new(2025, 1, 29, 0xFF);
        let time1 = crate::data::Time::new(Some(0), Some(0), Some(0), None);
        let dt1 = DateTime::new(date1, time1, None, None);
        let date2 = crate::data::Date::new(2025, 1, 29, 0xFF);
        let time2 = crate::data::Time::new(Some(0), Some(15), Some(0), None);
        let dt2 = DateTime::new(date2, time2, None, None);
        let buffer_data = Data::Structure(vec![
            Data::Structure(vec![Data::DateTime(dt1), Data::DoubleLongUnsigned(1000)]),
            Data::Structure(vec![Data::DateTime(dt2), Data::DoubleLongUnsigned(2000)]),
        ]);

        let response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0,
            result: GetDataResult::Data(buffer_data),
        });
        client.transport.push_response(response.encode());

        let obis = ObisCode::new(1, 0, 99, 1, 0, 255);
        let from_date = crate::data::Date::new(2025, 1, 29, 0xFF);
        let from_time = crate::data::Time::new(Some(0), Some(0), Some(0), None);
        let from = DateTime::new(from_date, from_time, None, None);
        let to_date = crate::data::Date::new(2025, 1, 30, 0xFF);
        let to_time = crate::data::Time::new(Some(0), Some(0), Some(0), None);
        let to = DateTime::new(to_date, to_time, None, None);
        let result = client.read_load_profile(obis, from, to);

        assert!(result.is_ok());
        let rows = result.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 2);
        assert!(matches!(rows[0][0], Data::DateTime(_)));
        assert!(matches!(rows[0][1], Data::DoubleLongUnsigned(1000)));
        assert!(matches!(rows[1][1], Data::DoubleLongUnsigned(2000)));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_load_profile_invalid_format() {
        use crate::get::{GetDataResult, GetResponse, GetResponseNormal};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare invalid response (not a Structure)
        let response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0,
            result: GetDataResult::Data(Data::DoubleLongUnsigned(12345)),
        });
        client.transport.push_response(response.encode());

        let obis = ObisCode::new(1, 0, 99, 1, 0, 255);
        let from_date = crate::data::Date::new(2025, 1, 29, 0xFF);
        let from_time = crate::data::Time::new(Some(0), Some(0), Some(0), None);
        let from = DateTime::new(from_date, from_time, None, None);
        let to_date = crate::data::Date::new(2025, 1, 30, 0xFF);
        let to_time = crate::data::Time::new(Some(0), Some(0), Some(0), None);
        let to = DateTime::new(to_date, to_time, None, None);
        let result = client.read_load_profile(obis, from, to);

        assert!(matches!(result, Err(ClientError::InvalidResponseData)));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_multiple_chunked_single_chunk() {
        use crate::get::{GetDataResult, GetResponse, GetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare GET-Response-With-List for 5 attributes (under default limit of 10)
        let response = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(1)),
                GetDataResult::Data(Data::DoubleLongUnsigned(2)),
                GetDataResult::Data(Data::DoubleLongUnsigned(3)),
                GetDataResult::Data(Data::DoubleLongUnsigned(4)),
                GetDataResult::Data(Data::DoubleLongUnsigned(5)),
            ],
        });
        client.transport.push_response(response.encode());

        let requests = [
            (3, ObisCode::new(1, 0, 1, 8, 0, 255), 2),
            (3, ObisCode::new(1, 0, 2, 8, 0, 255), 2),
            (3, ObisCode::new(1, 0, 3, 8, 0, 255), 2),
            (3, ObisCode::new(1, 0, 4, 8, 0, 255), 2),
            (3, ObisCode::new(1, 0, 5, 8, 0, 255), 2),
        ];
        let results = client.read_multiple_chunked(&requests, None);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 5);
        assert!(matches!(results[0], Ok(Data::DoubleLongUnsigned(1))));
        assert!(matches!(results[1], Ok(Data::DoubleLongUnsigned(2))));
        assert!(matches!(results[2], Ok(Data::DoubleLongUnsigned(3))));
        assert!(matches!(results[3], Ok(Data::DoubleLongUnsigned(4))));
        assert!(matches!(results[4], Ok(Data::DoubleLongUnsigned(5))));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_multiple_chunked_multiple_chunks() {
        use crate::get::{GetDataResult, GetResponse, GetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare responses for 3 chunks: 10 + 10 + 5 = 25 attributes
        // First chunk (10 items)
        let response1 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: (1..=10).map(|i| GetDataResult::Data(Data::DoubleLongUnsigned(i))).collect(),
        });
        client.transport.push_response(response1.encode());

        // Second chunk (10 items)
        let response2 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 1,
            results: (11..=20).map(|i| GetDataResult::Data(Data::DoubleLongUnsigned(i))).collect(),
        });
        client.transport.push_response(response2.encode());

        // Third chunk (5 items)
        let response3 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 2,
            results: (21..=25).map(|i| GetDataResult::Data(Data::DoubleLongUnsigned(i))).collect(),
        });
        client.transport.push_response(response3.encode());

        // Create 25 requests
        let mut requests = Vec::new();
        for i in 0..25 {
            requests.push((3, ObisCode::new(1, 0, i, 8, 0, 255), 2));
        }

        let results = client.read_multiple_chunked(&requests, None);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 25);
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok());
            if let Ok(Data::DoubleLongUnsigned(val)) = result {
                assert_eq!(*val, (i + 1) as u32);
            }
        }
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_read_multiple_chunked_exact_boundary() {
        use crate::get::{GetDataResult, GetResponse, GetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare responses for exactly 20 attributes (2 chunks of 10)
        let response1 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: (1..=10).map(|i| GetDataResult::Data(Data::DoubleLongUnsigned(i))).collect(),
        });
        client.transport.push_response(response1.encode());

        let response2 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 1,
            results: (11..=20).map(|i| GetDataResult::Data(Data::DoubleLongUnsigned(i))).collect(),
        });
        client.transport.push_response(response2.encode());

        let mut requests = Vec::new();
        for i in 0..20 {
            requests.push((3, ObisCode::new(1, 0, i, 8, 0, 255), 2));
        }

        let results = client.read_multiple_chunked(&requests, None);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 20);
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_write_multiple_chunked_success() {
        use crate::set::{SetResponse, SetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Prepare responses for 2 chunks: 10 + 5 = 15 writes
        let response1 = SetResponse::WithList(SetResponseWithList {
            invoke_id: 0,
            results: vec![DataAccessResult::Success; 10],
        });
        client.transport.push_response(response1.encode());

        let response2 = SetResponse::WithList(SetResponseWithList {
            invoke_id: 1,
            results: vec![DataAccessResult::Success; 5],
        });
        client.transport.push_response(response2.encode());

        let mut requests = Vec::new();
        for i in 0..15 {
            requests.push((1, ObisCode::new(0, 0, 96, 1, i, 255), 2, Data::Unsigned(i)));
        }

        let results = client.write_multiple_chunked(&requests, None);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 15);
        for result in results {
            assert_eq!(result, DataAccessResult::Success);
        }
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_chunked_with_errors() {
        use crate::get::{GetDataResult, GetResponse, GetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // First chunk succeeds
        let response1 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: (1..=10).map(|i| GetDataResult::Data(Data::DoubleLongUnsigned(i))).collect(),
        });
        client.transport.push_response(response1.encode());

        // Second chunk has mixed results
        let response2 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 1,
            results: vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(11)),
                GetDataResult::DataAccessError(DataAccessResult::ObjectUndefined),
                GetDataResult::Data(Data::DoubleLongUnsigned(13)),
            ],
        });
        client.transport.push_response(response2.encode());

        let mut requests = Vec::new();
        for i in 0..13 {
            requests.push((3, ObisCode::new(1, 0, i, 8, 0, 255), 2));
        }

        let results = client.read_multiple_chunked(&requests, None);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 13);
        // First 10 succeed
        for result in results.iter().take(10) {
            assert!(result.is_ok());
        }
        // 11th succeeds
        assert!(matches!(results[10], Ok(Data::DoubleLongUnsigned(11))));
        // 12th fails
        assert!(matches!(results[11], Err(DataAccessResult::ObjectUndefined)));
        // 13th succeeds
        assert!(matches!(results[12], Ok(Data::DoubleLongUnsigned(13))));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_chunked_empty_request() {
        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        let requests: Vec<(u16, ObisCode, i8)> = Vec::new();
        let results = client.read_multiple_chunked(&requests, None);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_chunked_custom_size() {
        use crate::get::{GetDataResult, GetResponse, GetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings::default();

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Override chunk size to 3 - should create 4 chunks (3+3+3+1)
        // First chunk (3 items)
        let response1 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(1)),
                GetDataResult::Data(Data::DoubleLongUnsigned(2)),
                GetDataResult::Data(Data::DoubleLongUnsigned(3)),
            ],
        });
        client.transport.push_response(response1.encode());

        // Second chunk (3 items)
        let response2 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 1,
            results: vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(4)),
                GetDataResult::Data(Data::DoubleLongUnsigned(5)),
                GetDataResult::Data(Data::DoubleLongUnsigned(6)),
            ],
        });
        client.transport.push_response(response2.encode());

        // Third chunk (3 items)
        let response3 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 2,
            results: vec![
                GetDataResult::Data(Data::DoubleLongUnsigned(7)),
                GetDataResult::Data(Data::DoubleLongUnsigned(8)),
                GetDataResult::Data(Data::DoubleLongUnsigned(9)),
            ],
        });
        client.transport.push_response(response3.encode());

        // Fourth chunk (1 item)
        let response4 = GetResponse::WithList(GetResponseWithList {
            invoke_id: 3,
            results: vec![GetDataResult::Data(Data::DoubleLongUnsigned(10))],
        });
        client.transport.push_response(response4.encode());

        let mut requests = Vec::new();
        for i in 0..10 {
            requests.push((3, ObisCode::new(1, 0, i, 8, 0, 255), 2));
        }

        // Override max_per_request to 3
        let results = client.read_multiple_chunked(&requests, Some(3));

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 10);
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok());
            if let Ok(Data::DoubleLongUnsigned(val)) = result {
                assert_eq!(*val, (i + 1) as u32);
            }
        }
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_chunked_no_limit() {
        use crate::get::{GetDataResult, GetResponse, GetResponseWithList};

        let transport = MockTransport::new();
        let settings = ClientSettings { max_attributes_per_request: None, ..Default::default() };

        // Connect first
        let aare = AareApdu {
            protocol_version: 1,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        };
        transport.push_response(aare.encode());

        let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);
        client.connect().unwrap();

        // Should send all 25 in single request
        let response = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0,
            results: (1..=25).map(|i| GetDataResult::Data(Data::DoubleLongUnsigned(i))).collect(),
        });
        client.transport.push_response(response.encode());

        let mut requests = Vec::new();
        for i in 0..25 {
            requests.push((3, ObisCode::new(1, 0, i, 8, 0, 255), 2));
        }

        let results = client.read_multiple_chunked(&requests, None);

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 25);
    }
}
