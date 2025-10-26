//! ACTION service implementation for DLMS/COSEM protocol
//!
//! This module implements the ACTION service as specified in DLMS Green Book Ed. 12.
//! The ACTION service allows invoking methods on COSEM objects on a DLMS server.
//!
//! # APDU Tags
//! - ACTION-Request: 0xC3 (195)
//! - ACTION-Response: 0xC7 (199)
//!
//! # Green Book References
//! - Table 73: Service parameters of the ACTION service
//! - Table 74: ACTION service request and response types
//! - Table 97: ACTION service types and APDUs
//! - Table 101: Mapping between the ACTION and the Write service
//! - Figure 127: MSC of the ACTION service
//! - Figure 128: MSC of the ACTION service with block transfer
//!
//! # Examples
//!
//! ## Creating an ACTION-Request-Normal
//! ```
//! use dlms_cosem::action::{ActionRequest, ActionRequestNormal};
//! use dlms_cosem::{ObisCode, Data};
//!
//! let request = ActionRequest::Normal(ActionRequestNormal {
//!     invoke_id: 0x01,
//!     class_id: 8,  // Clock
//!     instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
//!     method_id: 1,  // adjust_to_quarter
//!     method_invocation_parameters: None,
//! });
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
#[cfg(feature = "parse")]
use nom::IResult;

use crate::data::Data;
use crate::obis_code::ObisCode;

#[cfg(feature = "encode")]
use crate::data::ByteBuffer;

/// ACTION service request types
///
/// As specified in Green Book Table 74.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ActionRequest {
    /// ACTION-Request-Normal: Invoke a single method (choice 0x01)
    Normal(ActionRequestNormal),
    /// ACTION-Request-Next-PBlock: Continue parameterized block transfer (choice 0x02)
    NextPBlock(ActionRequestNextPBlock),
    /// ACTION-Request-With-List: Invoke multiple methods (choice 0x03)
    WithList(ActionRequestWithList),
    /// ACTION-Request-With-First-PBlock: Start parameterized block transfer (choice 0x04)
    WithFirstPBlock(ActionRequestWithFirstPBlock),
    /// ACTION-Request-With-List-And-First-PBlock: Multiple methods with block transfer (choice 0x05)
    WithListAndFirstPBlock(ActionRequestWithListAndFirstPBlock),
}

/// ACTION-Request-Normal: Invoke a single COSEM method
///
/// Encoding format:
/// ```text
/// C3 01 [invoke_id] [class_id:2] [instance_id:6] [method_id] [params?]
/// │  │  │           │             │               │           └─ Optional method parameters (A-XDR)
/// │  │  │           │             │               └──────────── method_id (1 byte, signed)
/// │  │  │           │             └──────────────────────────── instance_id (OBIS code, 6 bytes)
/// │  │  │           └────────────────────────────────────────── class_id (2 bytes, big-endian)
/// │  │  └────────────────────────────────────────────────────── invoke_id (1 byte)
/// │  └───────────────────────────────────────────────────────── choice: Normal (0x01)
/// └──────────────────────────────────────────────────────────── tag: ACTION-Request (0xC3)
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionRequestNormal {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// COSEM interface class ID (2 bytes, big-endian)
    pub class_id: u16,
    /// Logical name (OBIS code, 6 bytes)
    pub instance_id: ObisCode,
    /// Method index (1 byte, signed)
    pub method_id: i8,
    /// Optional method invocation parameters (A-XDR encoded)
    pub method_invocation_parameters: Option<Data>,
}

/// ACTION-Request-Next-PBlock: Request next parameterized data block
///
/// Used for long ACTION requests where method parameters are transferred in blocks.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionRequestNextPBlock {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// Block number (4 bytes, big-endian)
    pub block_number: u32,
}

/// ACTION-Request-With-List: Invoke multiple methods
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionRequestWithList {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// List of method descriptors
    pub method_descriptors: Vec<MethodDescriptor>,
}

/// ACTION-Request-With-First-PBlock: Start parameterized block transfer
///
/// Used when method parameters are too large for a single APDU.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionRequestWithFirstPBlock {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// COSEM interface class ID (2 bytes, big-endian)
    pub class_id: u16,
    /// Logical name (OBIS code, 6 bytes)
    pub instance_id: ObisCode,
    /// Method index (1 byte, signed)
    pub method_id: i8,
    /// First block of method parameters
    pub pblock: DataBlockSa,
}

/// ACTION-Request-With-List-And-First-PBlock: Multiple methods with block transfer
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionRequestWithListAndFirstPBlock {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// List of method descriptors
    pub method_descriptors: Vec<MethodDescriptor>,
    /// First block of method parameters
    pub pblock: DataBlockSa,
}

/// Method descriptor for ACTION-Request-With-List
///
/// Identifies a method to invoke and its parameters.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MethodDescriptor {
    /// COSEM interface class ID (2 bytes, big-endian)
    pub class_id: u16,
    /// Logical name (OBIS code, 6 bytes)
    pub instance_id: ObisCode,
    /// Method index (1 byte, signed)
    pub method_id: i8,
    /// Optional method invocation parameters
    pub method_invocation_parameters: Option<Data>,
}

/// Parameterized data block (for block transfer)
///
/// Used for ACTION service with block transfer (SA = Selective Access).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct DataBlockSa {
    /// Last block flag (boolean)
    pub last_block: bool,
    /// Block number (4 bytes, big-endian)
    pub block_number: u32,
    /// Raw data bytes for this block
    pub raw_data: Vec<u8>,
}

/// ACTION service response types
///
/// As specified in Green Book Table 74.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ActionResponse {
    /// ACTION-Response-Normal: Single method result (choice 0x01)
    Normal(ActionResponseNormal),
    /// ACTION-Response-With-PBlock: Parameterized block transfer response (choice 0x02)
    WithPBlock(ActionResponseWithPBlock),
    /// ACTION-Response-With-List: Multiple method results (choice 0x03)
    WithList(ActionResponseWithList),
    /// ACTION-Response-Next-PBlock: Continue block transfer (choice 0x04)
    NextPBlock(ActionResponseNextPBlock),
}

/// ACTION-Response-Normal: Result of a single method invocation
///
/// Encoding format:
/// ```text
/// C7 01 [invoke_id] [result]
/// │  │  │           └─ ActionResult (1+ bytes)
/// │  │  └──────────── invoke_id (1 byte)
/// │  └─────────────── choice: Normal (0x01)
/// └────────────────── tag: ACTION-Response (0xC7)
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionResponseNormal {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// Result of the method invocation
    pub result: ActionResult,
}

/// ACTION-Response-With-PBlock: Response with parameterized data block
///
/// Used when return parameters are too large for a single APDU.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionResponseWithPBlock {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// Data block with return parameters
    pub pblock: DataBlockSa,
}

/// ACTION-Response-With-List: Results of multiple method invocations
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionResponseWithList {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// List of action results
    pub results: Vec<ActionResult>,
}

/// ACTION-Response-Next-PBlock: Continue block transfer
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ActionResponseNextPBlock {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// Block number (4 bytes, big-endian)
    pub block_number: u32,
}

/// Result of an ACTION service method invocation
///
/// As specified in Green Book Table 73.
/// Values align with DataAccessResult for consistency.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ActionResult {
    /// Success (choice 0x00) with optional return parameters
    Success(Option<GetDataResult>),
    /// Hardware fault (code 1)
    HardwareFault,
    /// Temporary failure (code 2)
    TemporaryFailure,
    /// Read-write denied (code 3)
    ReadWriteDenied,
    /// Object undefined (code 4)
    ObjectUndefined,
    /// Object class inconsistent (code 9)
    ObjectClassInconsistent,
    /// Object unavailable (code 11)
    ObjectUnavailable,
    /// Type unmatched (code 12)
    TypeUnmatched,
    /// Scope of access violated (code 13)
    ScopeOfAccessViolated,
    /// Data block unavailable (code 14)
    DataBlockUnavailable,
    /// Long action aborted (code 15)
    LongActionAborted,
    /// No long action in progress (code 16)
    NoLongActionInProgress,
    /// Other reason (code 250)
    OtherReason,
}

/// Result data for successful ACTION
///
/// Either the method return value (Data) or an error code.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GetDataResult {
    /// Method returned data successfully
    Data(Data),
    /// Method invocation failed with error code
    DataAccessError(u8),
}

// ============================================================================
// Encoding implementation (conditional on "encode" feature)
// ============================================================================

#[cfg(feature = "encode")]
impl ActionRequest {
    /// Encode ACTION-Request to bytes
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::action::{ActionRequest, ActionRequestNormal};
    /// use dlms_cosem::ObisCode;
    ///
    /// let request = ActionRequest::Normal(ActionRequestNormal {
    ///     invoke_id: 0x01,
    ///     class_id: 8,
    ///     instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
    ///     method_id: 1,
    ///     method_invocation_parameters: None,
    /// });
    ///
    /// let encoded = request.encode();
    /// assert_eq!(encoded[0], 0xC3); // ACTION-Request tag
    /// assert_eq!(encoded[1], 0x01); // Normal choice
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(0xC3); // ACTION-Request tag

        match self {
            ActionRequest::Normal(req) => {
                buffer.push(0x01); // Normal choice
                buffer.push(req.invoke_id);
                buffer.push_u16(req.class_id); // Big-endian
                buffer.extend_from_slice(&req.instance_id.encode());
                buffer.push(req.method_id as u8);

                // Optional method invocation parameters
                if let Some(ref params) = req.method_invocation_parameters {
                    buffer.push(0x01); // parameters present
                    buffer.extend(params.encode());
                } else {
                    buffer.push(0x00); // no parameters
                }
            }
            ActionRequest::NextPBlock(req) => {
                buffer.push(0x02); // NextPBlock choice
                buffer.push(req.invoke_id);
                buffer.push_u32(req.block_number); // Big-endian
            }
            ActionRequest::WithList(req) => {
                buffer.push(0x03); // WithList choice
                buffer.push(req.invoke_id);

                // Encode number of method descriptors
                buffer.push(req.method_descriptors.len() as u8);

                // Encode each method descriptor
                for descriptor in &req.method_descriptors {
                    buffer.push_u16(descriptor.class_id);
                    buffer.extend_from_slice(&descriptor.instance_id.encode());
                    buffer.push(descriptor.method_id as u8);

                    if let Some(ref params) = descriptor.method_invocation_parameters {
                        buffer.push(0x01); // parameters present
                        buffer.extend(params.encode());
                    } else {
                        buffer.push(0x00); // no parameters
                    }
                }
            }
            ActionRequest::WithFirstPBlock(req) => {
                buffer.push(0x04); // WithFirstPBlock choice
                buffer.push(req.invoke_id);
                buffer.push_u16(req.class_id);
                buffer.extend_from_slice(&req.instance_id.encode());
                buffer.push(req.method_id as u8);

                // Encode pblock
                buffer.push(req.pblock.last_block as u8);
                buffer.push_u32(req.pblock.block_number);
                buffer.extend(&req.pblock.raw_data);
            }
            ActionRequest::WithListAndFirstPBlock(req) => {
                buffer.push(0x05); // WithListAndFirstPBlock choice
                buffer.push(req.invoke_id);

                buffer.push(req.method_descriptors.len() as u8);
                for descriptor in &req.method_descriptors {
                    buffer.push_u16(descriptor.class_id);
                    buffer.extend_from_slice(&descriptor.instance_id.encode());
                    buffer.push(descriptor.method_id as u8);

                    if let Some(ref params) = descriptor.method_invocation_parameters {
                        buffer.push(0x01);
                        buffer.extend(params.encode());
                    } else {
                        buffer.push(0x00);
                    }
                }

                // Encode pblock
                buffer.push(req.pblock.last_block as u8);
                buffer.push_u32(req.pblock.block_number);
                buffer.extend(&req.pblock.raw_data);
            }
        }

        buffer
    }
}

#[cfg(feature = "encode")]
impl ActionResponse {
    /// Encode ACTION-Response to bytes
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::action::{ActionResponse, ActionResponseNormal, ActionResult};
    ///
    /// let response = ActionResponse::Normal(ActionResponseNormal {
    ///     invoke_id: 0x01,
    ///     result: ActionResult::Success(None),
    /// });
    ///
    /// let encoded = response.encode();
    /// assert_eq!(encoded[0], 0xC7); // ACTION-Response tag
    /// assert_eq!(encoded[1], 0x01); // Normal choice
    /// ```
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(0xC7); // ACTION-Response tag

        match self {
            ActionResponse::Normal(resp) => {
                buffer.push(0x01); // Normal choice
                buffer.push(resp.invoke_id);
                buffer.extend(resp.result.encode());
            }
            ActionResponse::WithPBlock(resp) => {
                buffer.push(0x02); // WithPBlock choice
                buffer.push(resp.invoke_id);
                buffer.push(resp.pblock.last_block as u8);
                buffer.push_u32(resp.pblock.block_number);
                buffer.extend(&resp.pblock.raw_data);
            }
            ActionResponse::WithList(resp) => {
                buffer.push(0x03); // WithList choice
                buffer.push(resp.invoke_id);
                buffer.push(resp.results.len() as u8);
                for result in &resp.results {
                    buffer.extend(result.encode());
                }
            }
            ActionResponse::NextPBlock(resp) => {
                buffer.push(0x04); // NextPBlock choice
                buffer.push(resp.invoke_id);
                buffer.push_u32(resp.block_number);
            }
        }

        buffer
    }
}

impl ActionResult {
    #[cfg(feature = "encode")]
    /// Convert ActionResult to error code (for encoding/parsing)
    fn to_error_code(&self) -> u8 {
        match self {
            ActionResult::Success(_) => 0,
            ActionResult::HardwareFault => 1,
            ActionResult::TemporaryFailure => 2,
            ActionResult::ReadWriteDenied => 3,
            ActionResult::ObjectUndefined => 4,
            ActionResult::ObjectClassInconsistent => 9,
            ActionResult::ObjectUnavailable => 11,
            ActionResult::TypeUnmatched => 12,
            ActionResult::ScopeOfAccessViolated => 13,
            ActionResult::DataBlockUnavailable => 14,
            ActionResult::LongActionAborted => 15,
            ActionResult::NoLongActionInProgress => 16,
            ActionResult::OtherReason => 250,
        }
    }

    #[cfg(feature = "parse")]
    /// Convert error code to ActionResult (for parsing)
    fn from_error_code(code: u8) -> Self {
        match code {
            0 => ActionResult::Success(None),
            1 => ActionResult::HardwareFault,
            2 => ActionResult::TemporaryFailure,
            3 => ActionResult::ReadWriteDenied,
            4 => ActionResult::ObjectUndefined,
            9 => ActionResult::ObjectClassInconsistent,
            11 => ActionResult::ObjectUnavailable,
            12 => ActionResult::TypeUnmatched,
            13 => ActionResult::ScopeOfAccessViolated,
            14 => ActionResult::DataBlockUnavailable,
            15 => ActionResult::LongActionAborted,
            16 => ActionResult::NoLongActionInProgress,
            _ => ActionResult::OtherReason, // Default for unknown codes (including 250)
        }
    }

    #[cfg(feature = "encode")]
    /// Encode ActionResult to bytes
    ///
    /// Format:
    /// - Success (0x00): 00 [optional GetDataResult]
    /// - Error codes: 01 [error_code]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        match self {
            ActionResult::Success(opt_data) => {
                buffer.push(0x00); // Success choice

                if let Some(result) = opt_data {
                    buffer.push(0x01); // data present
                    buffer.extend(result.encode());
                } else {
                    buffer.push(0x00); // no data
                }
            }
            _ => {
                buffer.push(0x01); // Error choice
                buffer.push(self.to_error_code());
            }
        }

        buffer
    }
}

#[cfg(feature = "encode")]
impl GetDataResult {
    /// Encode GetDataResult to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        match self {
            GetDataResult::Data(data) => {
                buffer.push(0x00); // Data choice
                buffer.extend(data.encode());
            }
            GetDataResult::DataAccessError(error_code) => {
                buffer.push(0x01); // DataAccessError choice
                buffer.push(*error_code);
            }
        }

        buffer
    }
}

// ============================================================================
// Parsing implementation
// ============================================================================

impl ActionRequest {
    /// Parse ACTION-Request from bytes
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::action::ActionRequest;
    ///
    /// let data = vec![0xC3, 0x01, 0x01, 0x00, 0x08, 0x00, 0x00, 0x01, 0x00, 0x00, 0xFF, 0x01, 0x00];
    /// let (remaining, request) = ActionRequest::parse(&data).unwrap();
    /// assert!(remaining.is_empty());
    /// ```
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        use nom::number::streaming::{be_u16, be_u32, u8 as nom_u8};

        let (input, tag) = nom_u8(input)?;
        if tag != 0xC3 {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        let (input, choice) = nom_u8(input)?;

        match choice {
            0x01 => {
                // Normal
                let (input, invoke_id) = nom_u8(input)?;
                let (input, class_id) = be_u16(input)?;
                let (input, instance_id) = ObisCode::parse(input)?;
                let (input, method_id) = nom_u8(input)?;
                let (input, has_params) = nom_u8(input)?;

                let (input, method_invocation_parameters) = if has_params != 0 {
                    let (input, params) = Data::parse(input)?;
                    (input, Some(params))
                } else {
                    (input, None)
                };

                Ok((
                    input,
                    ActionRequest::Normal(ActionRequestNormal {
                        invoke_id,
                        class_id,
                        instance_id,
                        method_id: method_id as i8,
                        method_invocation_parameters,
                    }),
                ))
            }
            0x02 => {
                // NextPBlock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;

                Ok((
                    input,
                    ActionRequest::NextPBlock(ActionRequestNextPBlock { invoke_id, block_number }),
                ))
            }
            0x03 => {
                // WithList
                let (input, invoke_id) = nom_u8(input)?;
                let (input, count) = nom_u8(input)?;

                let mut method_descriptors = Vec::new();
                let mut remaining = input;

                for _ in 0..count {
                    let (input, class_id) = be_u16(remaining)?;
                    let (input, instance_id) = ObisCode::parse(input)?;
                    let (input, method_id) = nom_u8(input)?;
                    let (input, has_params) = nom_u8(input)?;

                    let (input, method_invocation_parameters) = if has_params != 0 {
                        let (input, params) = Data::parse(input)?;
                        (input, Some(params))
                    } else {
                        (input, None)
                    };

                    method_descriptors.push(MethodDescriptor {
                        class_id,
                        instance_id,
                        method_id: method_id as i8,
                        method_invocation_parameters,
                    });

                    remaining = input;
                }

                Ok((
                    remaining,
                    ActionRequest::WithList(ActionRequestWithList {
                        invoke_id,
                        method_descriptors,
                    }),
                ))
            }
            0x04 => {
                // WithFirstPBlock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, class_id) = be_u16(input)?;
                let (input, instance_id) = ObisCode::parse(input)?;
                let (input, method_id) = nom_u8(input)?;
                let (input, last_block) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;

                // Remaining data is raw_data for this block
                let raw_data = input.to_vec();

                Ok((
                    &[],
                    ActionRequest::WithFirstPBlock(ActionRequestWithFirstPBlock {
                        invoke_id,
                        class_id,
                        instance_id,
                        method_id: method_id as i8,
                        pblock: DataBlockSa { last_block: last_block != 0, block_number, raw_data },
                    }),
                ))
            }
            0x05 => {
                // WithListAndFirstPBlock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, count) = nom_u8(input)?;

                let mut method_descriptors = Vec::new();
                let mut remaining = input;

                for _ in 0..count {
                    let (input, class_id) = be_u16(remaining)?;
                    let (input, instance_id) = ObisCode::parse(input)?;
                    let (input, method_id) = nom_u8(input)?;
                    let (input, has_params) = nom_u8(input)?;

                    let (input, method_invocation_parameters) = if has_params != 0 {
                        let (input, params) = Data::parse(input)?;
                        (input, Some(params))
                    } else {
                        (input, None)
                    };

                    method_descriptors.push(MethodDescriptor {
                        class_id,
                        instance_id,
                        method_id: method_id as i8,
                        method_invocation_parameters,
                    });

                    remaining = input;
                }

                let (input, last_block) = nom_u8(remaining)?;
                let (input, block_number) = be_u32(input)?;

                let raw_data = input.to_vec();

                Ok((
                    &[],
                    ActionRequest::WithListAndFirstPBlock(ActionRequestWithListAndFirstPBlock {
                        invoke_id,
                        method_descriptors,
                        pblock: DataBlockSa { last_block: last_block != 0, block_number, raw_data },
                    }),
                ))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Switch))),
        }
    }
}

impl ActionResponse {
    /// Parse ACTION-Response from bytes
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::action::ActionResponse;
    ///
    /// let data = vec![0xC7, 0x01, 0x01, 0x00, 0x00];
    /// let (remaining, response) = ActionResponse::parse(&data).unwrap();
    /// assert!(remaining.is_empty());
    /// ```
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        use nom::number::streaming::{be_u32, u8 as nom_u8};

        let (input, tag) = nom_u8(input)?;
        if tag != 0xC7 {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        let (input, choice) = nom_u8(input)?;

        match choice {
            0x01 => {
                // Normal
                let (input, invoke_id) = nom_u8(input)?;
                let (input, result) = ActionResult::parse(input)?;

                Ok((input, ActionResponse::Normal(ActionResponseNormal { invoke_id, result })))
            }
            0x02 => {
                // WithPBlock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, last_block) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;

                let raw_data = input.to_vec();

                Ok((
                    &[],
                    ActionResponse::WithPBlock(ActionResponseWithPBlock {
                        invoke_id,
                        pblock: DataBlockSa { last_block: last_block != 0, block_number, raw_data },
                    }),
                ))
            }
            0x03 => {
                // WithList
                let (input, invoke_id) = nom_u8(input)?;
                let (input, count) = nom_u8(input)?;

                let mut results = Vec::new();
                let mut remaining = input;

                for _ in 0..count {
                    let (input, result) = ActionResult::parse(remaining)?;
                    results.push(result);
                    remaining = input;
                }

                Ok((
                    remaining,
                    ActionResponse::WithList(ActionResponseWithList { invoke_id, results }),
                ))
            }
            0x04 => {
                // NextPBlock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;

                Ok((
                    input,
                    ActionResponse::NextPBlock(ActionResponseNextPBlock {
                        invoke_id,
                        block_number,
                    }),
                ))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Switch))),
        }
    }
}

impl ActionResult {
    #[cfg(feature = "parse")]
    /// Parse ActionResult from bytes
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        use nom::number::streaming::u8 as nom_u8;

        let (input, choice) = nom_u8(input)?;

        match choice {
            0x00 => {
                // Success
                let (input, has_data) = nom_u8(input)?;

                if has_data != 0 {
                    let (input, result) = GetDataResult::parse(input)?;
                    Ok((input, ActionResult::Success(Some(result))))
                } else {
                    Ok((input, ActionResult::Success(None)))
                }
            }
            0x01 => {
                // Error
                let (input, error_code) = nom_u8(input)?;
                Ok((input, ActionResult::from_error_code(error_code)))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Switch))),
        }
    }
}

impl GetDataResult {
    #[cfg(feature = "parse")]
    /// Parse GetDataResult from bytes
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        use nom::number::streaming::u8 as nom_u8;

        let (input, choice) = nom_u8(input)?;

        match choice {
            0x00 => {
                // Data
                let (input, data) = Data::parse(input)?;
                Ok((input, GetDataResult::Data(data)))
            }
            0x01 => {
                // DataAccessError
                let (input, error_code) = nom_u8(input)?;
                Ok((input, GetDataResult::DataAccessError(error_code)))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Switch))),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(all(test, feature = "encode", feature = "parse"))]
mod tests {
    use super::*;

    #[test]
    fn test_action_request_normal_no_params() {
        let request = ActionRequest::Normal(ActionRequestNormal {
            invoke_id: 0x01,
            class_id: 8,
            instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
            method_id: 1,
            method_invocation_parameters: None,
        });

        let encoded = request.encode();

        // Verify encoding
        assert_eq!(encoded[0], 0xC3); // ACTION-Request tag
        assert_eq!(encoded[1], 0x01); // Normal choice
        assert_eq!(encoded[2], 0x01); // invoke_id
        assert_eq!(encoded[3], 0x00); // class_id MSB
        assert_eq!(encoded[4], 0x08); // class_id LSB
        assert_eq!(encoded[5..11], [0x00, 0x00, 0x01, 0x00, 0x00, 0xFF]); // OBIS
        assert_eq!(encoded[11], 0x01); // method_id
        assert_eq!(encoded[12], 0x00); // no parameters

        // Round-trip test
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_action_request_normal_with_params() {
        let request = ActionRequest::Normal(ActionRequestNormal {
            invoke_id: 0x02,
            class_id: 8,
            instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
            method_id: 6,
            method_invocation_parameters: Some(Data::Integer(100)),
        });

        let encoded = request.encode();

        // Verify basic structure
        assert_eq!(encoded[0], 0xC3); // ACTION-Request tag
        assert_eq!(encoded[1], 0x01); // Normal choice
        assert_eq!(encoded[12], 0x01); // parameters present

        // Round-trip test
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_action_request_next_pblock() {
        let request = ActionRequest::NextPBlock(ActionRequestNextPBlock {
            invoke_id: 0x03,
            block_number: 12345,
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC3); // ACTION-Request tag
        assert_eq!(encoded[1], 0x02); // NextPBlock choice
        assert_eq!(encoded[2], 0x03); // invoke_id
        assert_eq!(encoded[3..7], [0x00, 0x00, 0x30, 0x39]); // block_number (12345 in big-endian)

        // Round-trip test
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_action_request_with_list() {
        let request = ActionRequest::WithList(ActionRequestWithList {
            invoke_id: 0x04,
            method_descriptors: vec![
                MethodDescriptor {
                    class_id: 8,
                    instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
                    method_id: 1,
                    method_invocation_parameters: None,
                },
                MethodDescriptor {
                    class_id: 3,
                    instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
                    method_id: 2,
                    method_invocation_parameters: Some(Data::Unsigned(42)),
                },
            ],
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC3); // ACTION-Request tag
        assert_eq!(encoded[1], 0x03); // WithList choice
        assert_eq!(encoded[2], 0x04); // invoke_id
        assert_eq!(encoded[3], 0x02); // count: 2 descriptors

        // Round-trip test
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_action_response_normal_success_no_data() {
        let response = ActionResponse::Normal(ActionResponseNormal {
            invoke_id: 0x01,
            result: ActionResult::Success(None),
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC7); // ACTION-Response tag
        assert_eq!(encoded[1], 0x01); // Normal choice
        assert_eq!(encoded[2], 0x01); // invoke_id
        assert_eq!(encoded[3], 0x00); // Success choice
        assert_eq!(encoded[4], 0x00); // no data

        // Round-trip test
        let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_action_response_normal_success_with_data() {
        let response = ActionResponse::Normal(ActionResponseNormal {
            invoke_id: 0x02,
            result: ActionResult::Success(Some(GetDataResult::Data(Data::Unsigned(123)))),
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC7); // ACTION-Response tag
        assert_eq!(encoded[1], 0x01); // Normal choice
        assert_eq!(encoded[2], 0x02); // invoke_id
        assert_eq!(encoded[3], 0x00); // Success choice
        assert_eq!(encoded[4], 0x01); // data present

        // Round-trip test
        let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_action_response_normal_error_codes() {
        let test_cases = vec![
            (ActionResult::HardwareFault, 1),
            (ActionResult::TemporaryFailure, 2),
            (ActionResult::ReadWriteDenied, 3),
            (ActionResult::ObjectUndefined, 4),
            (ActionResult::ObjectClassInconsistent, 9),
            (ActionResult::ObjectUnavailable, 11),
            (ActionResult::TypeUnmatched, 12),
            (ActionResult::ScopeOfAccessViolated, 13),
            (ActionResult::DataBlockUnavailable, 14),
            (ActionResult::LongActionAborted, 15),
            (ActionResult::NoLongActionInProgress, 16),
            (ActionResult::OtherReason, 250),
        ];

        for (result, expected_code) in test_cases {
            let response = ActionResponse::Normal(ActionResponseNormal {
                invoke_id: 0x05,
                result: result.clone(),
            });

            let encoded = response.encode();

            assert_eq!(encoded[0], 0xC7); // ACTION-Response tag
            assert_eq!(encoded[1], 0x01); // Normal choice
            assert_eq!(encoded[2], 0x05); // invoke_id
            assert_eq!(encoded[3], 0x01); // Error choice
            assert_eq!(encoded[4], expected_code); // error code

            // Round-trip test
            let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
            assert!(remaining.is_empty());
            assert_eq!(parsed, response);
        }
    }

    #[test]
    fn test_action_response_with_list() {
        let response = ActionResponse::WithList(ActionResponseWithList {
            invoke_id: 0x06,
            results: vec![
                ActionResult::Success(None),
                ActionResult::Success(Some(GetDataResult::Data(Data::Integer(42)))),
                ActionResult::ObjectUndefined,
            ],
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC7); // ACTION-Response tag
        assert_eq!(encoded[1], 0x03); // WithList choice
        assert_eq!(encoded[2], 0x06); // invoke_id
        assert_eq!(encoded[3], 0x03); // count: 3 results

        // Round-trip test
        let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_action_response_next_pblock() {
        let response = ActionResponse::NextPBlock(ActionResponseNextPBlock {
            invoke_id: 0x07,
            block_number: 54321,
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC7); // ACTION-Response tag
        assert_eq!(encoded[1], 0x04); // NextPBlock choice
        assert_eq!(encoded[2], 0x07); // invoke_id
        assert_eq!(encoded[3..7], [0x00, 0x00, 0xD4, 0x31]); // block_number (54321)

        // Round-trip test
        let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_get_data_result_data() {
        let result = GetDataResult::Data(Data::OctetString(vec![0x01, 0x02, 0x03]));
        let encoded = result.encode();

        assert_eq!(encoded[0], 0x00); // Data choice

        // Round-trip test
        let (remaining, parsed) = GetDataResult::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, result);
    }

    #[test]
    fn test_get_data_result_error() {
        let result = GetDataResult::DataAccessError(3);
        let encoded = result.encode();

        assert_eq!(encoded[0], 0x01); // DataAccessError choice
        assert_eq!(encoded[1], 0x03); // error code

        // Round-trip test
        let (remaining, parsed) = GetDataResult::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, result);
    }

    #[test]
    fn test_action_result_error_code_conversion() {
        // Test all error codes round-trip correctly
        for code in [0, 1, 2, 3, 4, 9, 11, 12, 13, 14, 15, 16, 250] {
            let result = ActionResult::from_error_code(code);
            let encoded_code = result.to_error_code();
            assert_eq!(encoded_code, code, "Code {} did not round-trip", code);
        }

        // Test unknown code maps to OtherReason
        let unknown = ActionResult::from_error_code(99);
        assert_eq!(unknown, ActionResult::OtherReason);
    }

    #[test]
    fn test_clock_adjust_to_quarter_example() {
        // Real-world example: Clock.adjust_to_quarter() method
        let request = ActionRequest::Normal(ActionRequestNormal {
            invoke_id: 0xC1,
            class_id: 8, // Clock interface class
            instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
            method_id: 1, // adjust_to_quarter
            method_invocation_parameters: None,
        });

        let encoded = request.encode();
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);

        // Successful response
        let response = ActionResponse::Normal(ActionResponseNormal {
            invoke_id: 0xC1,
            result: ActionResult::Success(None),
        });

        let encoded_resp = response.encode();
        let (remaining, parsed_resp) = ActionResponse::parse(&encoded_resp).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed_resp, response);
    }

    #[test]
    fn test_method_descriptor_round_trip() {
        let descriptor = MethodDescriptor {
            class_id: 15, // AssociationLN
            instance_id: ObisCode::new(0, 0, 40, 0, 0, 255),
            method_id: 3,
            method_invocation_parameters: Some(Data::Structure(vec![
                Data::Unsigned(5),
                Data::OctetString(vec![0xAA, 0xBB, 0xCC]),
            ])),
        };

        // Test via WithList request
        let request = ActionRequest::WithList(ActionRequestWithList {
            invoke_id: 0x10,
            method_descriptors: vec![descriptor],
        });

        let encoded = request.encode();
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_action_request_with_first_pblock() {
        let request = ActionRequest::WithFirstPBlock(ActionRequestWithFirstPBlock {
            invoke_id: 0x11,
            class_id: 7, // Profile Generic
            instance_id: ObisCode::new(1, 0, 99, 1, 0, 255),
            method_id: 2, // capture
            pblock: DataBlockSa {
                last_block: false,
                block_number: 1,
                raw_data: vec![0x01, 0x02, 0x03, 0x04, 0x05],
            },
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC3); // ACTION-Request tag
        assert_eq!(encoded[1], 0x04); // WithFirstPBlock choice
        assert_eq!(encoded[2], 0x11); // invoke_id

        // Round-trip test
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_action_request_with_list_and_first_pblock() {
        let request = ActionRequest::WithListAndFirstPBlock(ActionRequestWithListAndFirstPBlock {
            invoke_id: 0x12,
            method_descriptors: vec![MethodDescriptor {
                class_id: 8,
                instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
                method_id: 1,
                method_invocation_parameters: None,
            }],
            pblock: DataBlockSa { last_block: true, block_number: 5, raw_data: vec![0xAA, 0xBB] },
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC3); // ACTION-Request tag
        assert_eq!(encoded[1], 0x05); // WithListAndFirstPBlock choice

        // Round-trip test
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_action_response_with_pblock() {
        let response = ActionResponse::WithPBlock(ActionResponseWithPBlock {
            invoke_id: 0x13,
            pblock: DataBlockSa {
                last_block: false,
                block_number: 2,
                raw_data: vec![0x11, 0x22, 0x33, 0x44],
            },
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC7); // ACTION-Response tag
        assert_eq!(encoded[1], 0x02); // WithPBlock choice
        assert_eq!(encoded[2], 0x13); // invoke_id
        assert_eq!(encoded[3], 0x00); // last_block = false

        // Round-trip test
        let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_action_request_parse_invalid_tag() {
        let bytes = vec![0xC7, 0x01]; // Wrong tag (ACTION-Response instead of Request)
        assert!(ActionRequest::parse(&bytes).is_err());
    }

    #[test]
    fn test_action_response_parse_invalid_tag() {
        let bytes = vec![0xC3, 0x01]; // Wrong tag (ACTION-Request instead of Response)
        assert!(ActionResponse::parse(&bytes).is_err());
    }

    #[test]
    fn test_action_request_parse_invalid_choice() {
        let bytes = vec![0xC3, 0x99]; // Invalid choice
        assert!(ActionRequest::parse(&bytes).is_err());
    }

    #[test]
    fn test_action_response_parse_invalid_choice() {
        let bytes = vec![0xC7, 0x99]; // Invalid choice
        assert!(ActionResponse::parse(&bytes).is_err());
    }

    #[test]
    fn test_action_result_parse_invalid_choice() {
        let bytes = vec![0x99, 0x00]; // Invalid choice
        assert!(ActionResult::parse(&bytes).is_err());
    }

    #[test]
    fn test_get_data_result_parse_invalid_choice() {
        let bytes = vec![0x99]; // Invalid choice
        assert!(GetDataResult::parse(&bytes).is_err());
    }

    #[test]
    fn test_empty_method_descriptors_list() {
        let request = ActionRequest::WithList(ActionRequestWithList {
            invoke_id: 0x20,
            method_descriptors: vec![],
        });

        let encoded = request.encode();
        assert_eq!(encoded[3], 0x00); // count = 0

        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_empty_results_list() {
        let response =
            ActionResponse::WithList(ActionResponseWithList { invoke_id: 0x21, results: vec![] });

        let encoded = response.encode();
        assert_eq!(encoded[3], 0x00); // count = 0

        let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_action_result_success_with_data_access_error() {
        let result = ActionResult::Success(Some(GetDataResult::DataAccessError(3)));
        let encoded = result.encode();

        assert_eq!(encoded[0], 0x00); // Success choice
        assert_eq!(encoded[1], 0x01); // data present
        assert_eq!(encoded[2], 0x01); // DataAccessError choice
        assert_eq!(encoded[3], 0x03); // error code

        let (remaining, parsed) = ActionResult::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, result);
    }

    #[test]
    fn test_green_book_compliance_action_request_normal() {
        // Green Book compliant ACTION-Request-Normal structure
        // C3 01 [invoke_id] [class_id:2] [obis:6] [method_id] [params?]
        let request = ActionRequest::Normal(ActionRequestNormal {
            invoke_id: 0xC1,
            class_id: 8,
            instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
            method_id: 1,
            method_invocation_parameters: None,
        });

        let encoded = request.encode();

        // Verify Green Book structure byte-by-byte
        assert_eq!(encoded.len(), 13); // tag + choice + invoke_id + class_id(2) + obis(6) + method_id + params_flag
        assert_eq!(encoded[0], 0xC3); // ACTION-Request tag (Green Book Table 97)
        assert_eq!(encoded[1], 0x01); // Normal choice
        assert_eq!(encoded[2], 0xC1); // invoke_id
        assert_eq!(&encoded[3..5], &[0x00, 0x08]); // class_id = 8 (Clock)
        assert_eq!(&encoded[5..11], &[0x00, 0x00, 0x01, 0x00, 0x00, 0xFF]); // OBIS
        assert_eq!(encoded[11], 0x01); // method_id
        assert_eq!(encoded[12], 0x00); // no parameters

        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_green_book_compliance_action_response_normal() {
        // Green Book compliant ACTION-Response-Normal structure
        // C7 01 [invoke_id] [result]
        let response = ActionResponse::Normal(ActionResponseNormal {
            invoke_id: 0xC1,
            result: ActionResult::Success(None),
        });

        let encoded = response.encode();

        // Verify Green Book structure byte-by-byte
        assert_eq!(encoded.len(), 5); // tag + choice + invoke_id + result_choice + data_flag
        assert_eq!(encoded[0], 0xC7); // ACTION-Response tag (Green Book Table 97)
        assert_eq!(encoded[1], 0x01); // Normal choice
        assert_eq!(encoded[2], 0xC1); // invoke_id
        assert_eq!(encoded[3], 0x00); // Success choice
        assert_eq!(encoded[4], 0x00); // no return data

        let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_large_block_number() {
        // Test with maximum u32 block number
        let request = ActionRequest::NextPBlock(ActionRequestNextPBlock {
            invoke_id: 0xFF,
            block_number: u32::MAX,
        });

        let encoded = request.encode();
        assert_eq!(&encoded[3..7], &[0xFF, 0xFF, 0xFF, 0xFF]); // max u32 in big-endian

        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_complex_method_invocation_parameters() {
        // Test with complex nested Data structure
        let request = ActionRequest::Normal(ActionRequestNormal {
            invoke_id: 0x50,
            class_id: 15, // AssociationLN
            instance_id: ObisCode::new(0, 0, 40, 0, 0, 255),
            method_id: 4,
            method_invocation_parameters: Some(Data::Structure(vec![
                Data::Unsigned(10),
                Data::Structure(vec![Data::OctetString(vec![1, 2, 3]), Data::Integer(-100)]),
                Data::Structure(vec![Data::Unsigned(1), Data::Unsigned(0)]),
            ])),
        });

        let encoded = request.encode();

        // Round-trip test with complex parameters
        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_method_id_negative_values() {
        // Test with negative method_id (signed byte)
        let request = ActionRequest::Normal(ActionRequestNormal {
            invoke_id: 0x60,
            class_id: 1,
            instance_id: ObisCode::new(0, 0, 0, 0, 0, 0),
            method_id: -1,
            method_invocation_parameters: None,
        });

        let encoded = request.encode();
        assert_eq!(encoded[11], 0xFF); // -1 as u8 = 0xFF

        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_multiple_descriptors_with_mixed_parameters() {
        let request = ActionRequest::WithList(ActionRequestWithList {
            invoke_id: 0x70,
            method_descriptors: vec![
                MethodDescriptor {
                    class_id: 8,
                    instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
                    method_id: 1,
                    method_invocation_parameters: None,
                },
                MethodDescriptor {
                    class_id: 3,
                    instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
                    method_id: 2,
                    method_invocation_parameters: Some(Data::Unsigned(42)),
                },
                MethodDescriptor {
                    class_id: 7,
                    instance_id: ObisCode::new(1, 0, 99, 1, 0, 255),
                    method_id: 3,
                    method_invocation_parameters: Some(Data::Structure(vec![
                        Data::Integer(1),
                        Data::Integer(100),
                    ])),
                },
            ],
        });

        let encoded = request.encode();
        assert_eq!(encoded[3], 0x03); // count = 3 descriptors

        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }

    #[test]
    fn test_action_response_with_multiple_mixed_results() {
        let response = ActionResponse::WithList(ActionResponseWithList {
            invoke_id: 0x80,
            results: vec![
                ActionResult::Success(None),
                ActionResult::Success(Some(GetDataResult::Data(Data::Unsigned(123)))),
                ActionResult::ObjectUndefined,
                ActionResult::Success(Some(GetDataResult::DataAccessError(3))),
                ActionResult::TemporaryFailure,
            ],
        });

        let encoded = response.encode();
        assert_eq!(encoded[3], 0x05); // count = 5 results

        let (remaining, parsed) = ActionResponse::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, response);
    }

    #[test]
    fn test_pblock_last_block_true() {
        let request = ActionRequest::WithFirstPBlock(ActionRequestWithFirstPBlock {
            invoke_id: 0x90,
            class_id: 1,
            instance_id: ObisCode::new(0, 0, 0, 0, 0, 0),
            method_id: 1,
            pblock: DataBlockSa {
                last_block: true,
                block_number: 999,
                raw_data: vec![0xFF; 100], // Large data block
            },
        });

        let encoded = request.encode();

        // Verify last_block is encoded as 1
        let last_block_pos = 2 + 1 + 2 + 6 + 1; // invoke_id + class_id + obis + method_id
        assert_eq!(encoded[last_block_pos], 0x01); // last_block = true

        let (remaining, parsed) = ActionRequest::parse(&encoded).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(parsed, request);
    }
}
