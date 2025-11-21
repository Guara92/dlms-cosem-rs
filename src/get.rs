//! GET service implementation for DLMS/COSEM protocol
//!
//! This module implements the GET service as specified in DLMS Green Book Ed. 12.
//! The GET service allows reading attributes from COSEM objects on a DLMS server.
//!
//! # APDU Tags
//! - GET-Request: 0xC0 (192)
//! - GET-Response: 0xC4 (196)
//!
//! # Green Book References
//! - Table 69: Service parameters of the GET service
//! - Table 70: GET service request and response types
//! - Table 95: GET service types and APDUs
//! - Table 164: GET service example (line 1458: C0 01 00 03 01 01 01 08 00 FF 02)
//!
//! # Examples
//!
//! ## Creating a GET-Request-Normal
//! ```
//! use dlms_cosem::get::{GetRequest, GetRequestNormal};
//! use dlms_cosem::ObisCode;
//!
//! let request = GetRequest::Normal(GetRequestNormal {
//!     invoke_id: 0x00,
//!     class_id: 3,  // Register
//!     instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
//!     attribute_id: 2,
//!     access_selection: None,
//! });
//! ```


extern crate alloc;

use alloc::vec::Vec;
#[cfg(feature = "parse")]
use nom::IResult;

use crate::data::Data;
use crate::obis_code::ObisCode;

#[cfg(feature = "encode")]
use crate::data::ByteBuffer;

/// GET service request types
///
/// As specified in Green Book Table 70.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GetRequest {
    /// GET-Request-Normal: Read a single attribute (choice 0x01)
    Normal(GetRequestNormal),
    /// GET-Request-Next: Continue block transfer (choice 0x02)
    NextDataBlock(GetRequestNext),
    /// GET-Request-With-List: Read multiple attributes (choice 0x03)
    WithList(GetRequestWithList),
}

/// GET-Request-Normal: Read a single COSEM attribute
///
/// Encoding format (Green Book example, line 1458):
/// ```text
/// C0 01 00 03 01 01 01 08 00 FF 02
/// │  │  │  │  │  └─────────┘  └─── attribute_id (1 byte)
/// │  │  │  │  └─────────────────── instance_id (OBIS code, 6 bytes)
/// │  │  │  └────────────────────── class_id (2 bytes, big-endian)
/// │  │  └───────────────────────── invoke_id (1 byte)
/// │  └──────────────────────────── choice: Normal (0x01)
/// └─────────────────────────────── tag: GET-Request (0xC0)
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GetRequestNormal {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// COSEM interface class ID (2 bytes, big-endian)
    pub class_id: u16,
    /// Logical name (OBIS code, 6 bytes)
    pub instance_id: ObisCode,
    /// Attribute index (1 byte, signed)
    pub attribute_id: i8,
    /// Optional selective access descriptor
    pub access_selection: Option<AccessSelector>,
}

/// GET-Request-Next: Request next data block in block transfer
///
/// Used for long GET responses that don't fit in a single APDU.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GetRequestNext {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// Block number to request (4 bytes, big-endian)
    pub block_number: u32,
}

/// GET-Request-With-List: Read multiple attributes
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GetRequestWithList {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// List of attribute descriptors
    pub attribute_descriptor_list: Vec<AttributeDescriptor>,
}

/// Attribute descriptor for GET-Request-With-List
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AttributeDescriptor {
    /// COSEM interface class ID (2 bytes, big-endian)
    pub class_id: u16,
    /// Logical name (OBIS code, 6 bytes)
    pub instance_id: ObisCode,
    /// Attribute index (1 byte, signed)
    pub attribute_id: i8,
}

/// Selective access descriptor for filtering data
///
/// Used with GET requests to filter or select specific data ranges.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AccessSelector {
    /// Access selector type (1 byte)
    pub selector: u8,
    /// Access parameters (encoded as Data)
    pub parameters: Data,
}

/// GET service response types
///
/// As specified in Green Book Table 70.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GetResponse {
    /// GET-Response-Normal: Single attribute result (choice 0x01)
    Normal(GetResponseNormal),
    /// GET-Response-With-Datablock: Block transfer response (choice 0x02)
    WithDataBlock(GetResponseWithDataBlock),
    /// GET-Response-With-List: Multiple attributes result (choice 0x03)
    WithList(GetResponseWithList),
}

/// GET-Response-Normal: Result of reading a single attribute
///
/// Encoding format:
/// ```text
/// C4 01 [invoke_id] [result_choice] [data_or_error]
/// │  │
/// │  └─── choice: Normal (0x01)
/// └────── tag: GET-Response (0xC4)
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GetResponseNormal {
    /// Invoke ID (matches request)
    pub invoke_id: u8,
    /// Result: either data or error
    pub result: GetDataResult,
}

/// GET-Response-With-Datablock: Block transfer response
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GetResponseWithDataBlock {
    /// Invoke ID (matches request)
    pub invoke_id: u8,
    /// Whether this is the last block
    pub last_block: bool,
    /// Block number (4 bytes, big-endian)
    pub block_number: u32,
    /// Result: either raw data or error
    pub result: GetDataBlockResult,
}

/// GET-Response-With-List: Result of reading multiple attributes
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct GetResponseWithList {
    /// Invoke ID (matches request)
    pub invoke_id: u8,
    /// List of results (one per requested attribute)
    pub results: Vec<GetDataResult>,
}

/// Result of a GET request: either data or an error
///
/// Encoded as a choice:
/// - Choice 0x00: Data present
/// - Choice 0x01: DataAccessError
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GetDataResult {
    /// Choice 0x00: Attribute data successfully retrieved
    Data(Data),
    /// Choice 0x01: Error accessing the attribute
    DataAccessError(DataAccessResult),
}

/// Result of a GET block transfer: either raw data or an error
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum GetDataBlockResult {
    /// Choice 0x00: Raw data block
    RawData(Vec<u8>),
    /// Choice 0x01: Error accessing the data
    DataAccessError(DataAccessResult),
}

/// Data access error codes
///
/// As specified in Green Book (Blue Book Section 4.1.8.3.2).
/// Values: 0, 1, 2, 3, 4, 9, 11-19, 250
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
pub enum DataAccessResult {
    /// Success (0)
    Success = 0,
    /// Hardware fault (1)
    HardwareFault = 1,
    /// Temporary failure (2)
    TemporaryFailure = 2,
    /// Read-write denied (3)
    ReadWriteDenied = 3,
    /// Object undefined (4)
    ObjectUndefined = 4,
    /// Object class inconsistent (9)
    ObjectClassInconsistent = 9,
    /// Object unavailable (11)
    ObjectUnavailable = 11,
    /// Type unmatched (12)
    TypeUnmatched = 12,
    /// Scope of access violated (13)
    ScopeOfAccessViolated = 13,
    /// Data block unavailable (14)
    DataBlockUnavailable = 14,
    /// Long GET aborted (15)
    LongGetAborted = 15,
    /// No long GET in progress (16)
    NoLongGetInProgress = 16,
    /// Long SET aborted (17)
    LongSetAborted = 17,
    /// No long SET in progress (18)
    NoLongSetInProgress = 18,
    /// Data block number invalid (19)
    DataBlockNumberInvalid = 19,
    /// Other reason (250)
    OtherReason = 250,
}

impl DataAccessResult {
    /// Convert a u8 value to a DataAccessResult
    ///
    /// Returns None if the value is not a valid DataAccessResult code.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Success),
            1 => Some(Self::HardwareFault),
            2 => Some(Self::TemporaryFailure),
            3 => Some(Self::ReadWriteDenied),
            4 => Some(Self::ObjectUndefined),
            9 => Some(Self::ObjectClassInconsistent),
            11 => Some(Self::ObjectUnavailable),
            12 => Some(Self::TypeUnmatched),
            13 => Some(Self::ScopeOfAccessViolated),
            14 => Some(Self::DataBlockUnavailable),
            15 => Some(Self::LongGetAborted),
            16 => Some(Self::NoLongGetInProgress),
            17 => Some(Self::LongSetAborted),
            18 => Some(Self::NoLongSetInProgress),
            19 => Some(Self::DataBlockNumberInvalid),
            250 => Some(Self::OtherReason),
            _ => None,
        }
    }
}

// ============================================================================
// ENCODING (Feature-gated)
// ============================================================================

#[cfg(feature = "encode")]
impl GetRequest {
    /// Encode GET-Request to DLMS A-XDR format
    ///
    /// Returns a byte vector containing:
    /// - Tag (0xC0)
    /// - Choice (0x01, 0x02, or 0x03)
    /// - Request-specific fields
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(0xC0); // GET-Request tag

        match self {
            GetRequest::Normal(req) => {
                buf.push(0x01); // Choice: Normal
                buf.push(req.invoke_id);
                buf.push_u16(req.class_id); // Big-endian
                buf.extend_from_slice(&req.instance_id.encode());
                buf.push(req.attribute_id as u8);

                // Access selection (optional)
                if let Some(ref access) = req.access_selection {
                    buf.push(0x01); // Access selection present
                    buf.push(access.selector);
                    buf.extend_from_slice(&access.parameters.encode());
                } else {
                    buf.push(0x00); // No access selection
                }
            }
            GetRequest::NextDataBlock(req) => {
                buf.push(0x02); // Choice: NextDataBlock
                buf.push(req.invoke_id);
                buf.push_u32(req.block_number); // Big-endian
            }
            GetRequest::WithList(req) => {
                buf.push(0x03); // Choice: WithList
                buf.push(req.invoke_id);

                // Encode list count
                buf.push(req.attribute_descriptor_list.len() as u8);

                // Encode each descriptor
                for desc in &req.attribute_descriptor_list {
                    buf.push_u16(desc.class_id); // Big-endian
                    buf.extend_from_slice(&desc.instance_id.encode());
                    buf.push(desc.attribute_id as u8);
                }
            }
        }

        buf
    }
}

#[cfg(feature = "encode")]
impl GetResponse {
    /// Encode GET-Response to DLMS A-XDR format
    ///
    /// Returns a byte vector containing:
    /// - Tag (0xC4)
    /// - Choice (0x01, 0x02, or 0x03)
    /// - Response-specific fields
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(0xC4); // GET-Response tag

        match self {
            GetResponse::Normal(resp) => {
                buf.push(0x01); // Choice: Normal
                buf.push(resp.invoke_id);

                // Encode result
                match &resp.result {
                    GetDataResult::Data(data) => {
                        buf.push(0x00); // Choice: Data
                        buf.extend_from_slice(&data.encode());
                    }
                    GetDataResult::DataAccessError(error) => {
                        buf.push(0x01); // Choice: DataAccessError
                        buf.push(*error as u8);
                    }
                }
            }
            GetResponse::WithDataBlock(resp) => {
                buf.push(0x02); // Choice: WithDataBlock
                buf.push(resp.invoke_id);
                buf.push(if resp.last_block { 0x01 } else { 0x00 });
                buf.push_u32(resp.block_number); // Big-endian

                // Encode result
                match &resp.result {
                    GetDataBlockResult::RawData(data) => {
                        buf.push(0x00); // Choice: RawData
                        buf.extend_from_slice(data);
                    }
                    GetDataBlockResult::DataAccessError(error) => {
                        buf.push(0x01); // Choice: DataAccessError
                        buf.push(*error as u8);
                    }
                }
            }
            GetResponse::WithList(resp) => {
                buf.push(0x03); // Choice: WithList
                buf.push(resp.invoke_id);

                // Encode result count
                buf.push(resp.results.len() as u8);

                // Encode each result
                for result in &resp.results {
                    match result {
                        GetDataResult::Data(data) => {
                            buf.push(0x00); // Choice: Data
                            buf.extend_from_slice(&data.encode());
                        }
                        GetDataResult::DataAccessError(error) => {
                            buf.push(0x01); // Choice: DataAccessError
                            buf.push(*error as u8);
                        }
                    }
                }
            }
        }

        buf
    }
}

// ============================================================================
// PARSING
// ============================================================================

impl GetRequest {
    /// Parse a GET-Request from DLMS A-XDR format
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        use nom::number::streaming::{be_u16, be_u32, u8 as nom_u8};

        let (input, tag) = nom_u8(input)?;
        if tag != 0xC0 {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        let (input, choice) = nom_u8(input)?;

        match choice {
            0x01 => {
                // GET-Request-Normal
                let (input, invoke_id) = nom_u8(input)?;
                let (input, class_id) = be_u16(input)?;
                let (input, instance_id) = ObisCode::parse(input)?;
                let (input, attribute_id) = nom_u8(input)?;
                let (input, access_selection_present) = nom_u8(input)?;

                let (input, access_selection) = if access_selection_present == 0x01 {
                    let (input, selector) = nom_u8(input)?;
                    let (input, parameters) = Data::parse(input)?;
                    (input, Some(AccessSelector { selector, parameters }))
                } else {
                    (input, None)
                };

                Ok((
                    input,
                    GetRequest::Normal(GetRequestNormal {
                        invoke_id,
                        class_id,
                        instance_id,
                        attribute_id: attribute_id as i8,
                        access_selection,
                    }),
                ))
            }
            0x02 => {
                // GET-Request-Next
                let (input, invoke_id) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;
                Ok((input, GetRequest::NextDataBlock(GetRequestNext { invoke_id, block_number })))
            }
            0x03 => {
                // GET-Request-With-List
                let (input, invoke_id) = nom_u8(input)?;
                let (input, count) = nom_u8(input)?;

                let mut descriptors = Vec::with_capacity(count as usize);
                let mut remaining = input;

                for _ in 0..count {
                    let (input, class_id) = be_u16(remaining)?;
                    let (input, instance_id) = ObisCode::parse(input)?;
                    let (input, attribute_id) = nom_u8(input)?;

                    descriptors.push(AttributeDescriptor {
                        class_id,
                        instance_id,
                        attribute_id: attribute_id as i8,
                    });

                    remaining = input;
                }

                Ok((
                    remaining,
                    GetRequest::WithList(GetRequestWithList {
                        invoke_id,
                        attribute_descriptor_list: descriptors,
                    }),
                ))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
        }
    }
}

impl GetResponse {
    /// Parse a GET-Response from DLMS A-XDR format
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        use nom::number::streaming::{be_u32, u8 as nom_u8};

        let (input, tag) = nom_u8(input)?;
        if tag != 0xC4 {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        let (input, choice) = nom_u8(input)?;

        match choice {
            0x01 => {
                // GET-Response-Normal
                let (input, invoke_id) = nom_u8(input)?;
                let (input, result_choice) = nom_u8(input)?;

                let (input, result) = if result_choice == 0x00 {
                    let (input, data) = Data::parse(input)?;
                    (input, GetDataResult::Data(data))
                } else {
                    let (input, error_code) = nom_u8(input)?;
                    let error = DataAccessResult::from_u8(error_code).ok_or_else(|| {
                        nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Verify,
                        ))
                    })?;
                    (input, GetDataResult::DataAccessError(error))
                };

                Ok((input, GetResponse::Normal(GetResponseNormal { invoke_id, result })))
            }
            0x02 => {
                // GET-Response-With-Datablock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, last_block_byte) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;
                let (input, result_choice) = nom_u8(input)?;

                let last_block = last_block_byte != 0x00;

                let (input, result) = if result_choice == 0x00 {
                    // Raw data (rest of input for now - needs proper length handling)
                    (input, GetDataBlockResult::RawData(input.to_vec()))
                } else {
                    let (input, error_code) = nom_u8(input)?;
                    let error = DataAccessResult::from_u8(error_code).ok_or_else(|| {
                        nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Verify,
                        ))
                    })?;
                    (input, GetDataBlockResult::DataAccessError(error))
                };

                Ok((
                    input,
                    GetResponse::WithDataBlock(GetResponseWithDataBlock {
                        invoke_id,
                        last_block,
                        block_number,
                        result,
                    }),
                ))
            }
            0x03 => {
                // GET-Response-With-List
                let (input, invoke_id) = nom_u8(input)?;
                let (input, count) = nom_u8(input)?;

                let mut results = Vec::with_capacity(count as usize);
                let mut remaining = input;

                for _ in 0..count {
                    let (input, result_choice) = nom_u8(remaining)?;

                    let (input, result) = if result_choice == 0x00 {
                        let (input, data) = Data::parse(input)?;
                        (input, GetDataResult::Data(data))
                    } else {
                        let (input, error_code) = nom_u8(input)?;
                        let error = DataAccessResult::from_u8(error_code).ok_or_else(|| {
                            nom::Err::Error(nom::error::Error::new(
                                input,
                                nom::error::ErrorKind::Verify,
                            ))
                        })?;
                        (input, GetDataResult::DataAccessError(error))
                    };

                    results.push(result);
                    remaining = input;
                }

                Ok((remaining, GetResponse::WithList(GetResponseWithList { invoke_id, results })))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(all(test, feature = "encode", feature = "parse"))]
mod tests {
    use super::*;

    // ========================================================================
    // DataAccessResult Tests
    // ========================================================================

    #[test]
    fn test_data_access_result_from_u8_valid() {
        assert_eq!(DataAccessResult::from_u8(0), Some(DataAccessResult::Success));
        assert_eq!(DataAccessResult::from_u8(1), Some(DataAccessResult::HardwareFault));
        assert_eq!(DataAccessResult::from_u8(2), Some(DataAccessResult::TemporaryFailure));
        assert_eq!(DataAccessResult::from_u8(3), Some(DataAccessResult::ReadWriteDenied));
        assert_eq!(DataAccessResult::from_u8(4), Some(DataAccessResult::ObjectUndefined));
        assert_eq!(DataAccessResult::from_u8(9), Some(DataAccessResult::ObjectClassInconsistent));
        assert_eq!(DataAccessResult::from_u8(11), Some(DataAccessResult::ObjectUnavailable));
        assert_eq!(DataAccessResult::from_u8(12), Some(DataAccessResult::TypeUnmatched));
        assert_eq!(DataAccessResult::from_u8(13), Some(DataAccessResult::ScopeOfAccessViolated));
        assert_eq!(DataAccessResult::from_u8(14), Some(DataAccessResult::DataBlockUnavailable));
        assert_eq!(DataAccessResult::from_u8(15), Some(DataAccessResult::LongGetAborted));
        assert_eq!(DataAccessResult::from_u8(16), Some(DataAccessResult::NoLongGetInProgress));
        assert_eq!(DataAccessResult::from_u8(17), Some(DataAccessResult::LongSetAborted));
        assert_eq!(DataAccessResult::from_u8(18), Some(DataAccessResult::NoLongSetInProgress));
        assert_eq!(DataAccessResult::from_u8(19), Some(DataAccessResult::DataBlockNumberInvalid));
        assert_eq!(DataAccessResult::from_u8(250), Some(DataAccessResult::OtherReason));
    }

    #[test]
    fn test_data_access_result_from_u8_invalid() {
        assert_eq!(DataAccessResult::from_u8(5), None);
        assert_eq!(DataAccessResult::from_u8(10), None);
        assert_eq!(DataAccessResult::from_u8(20), None);
        assert_eq!(DataAccessResult::from_u8(255), None);
    }

    // ========================================================================
    // GET-Request-Normal Tests (TDD RED PHASE)
    // ========================================================================

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_request_normal_encode_basic() {
        // Test encoding GET-Request-Normal without access selection
        let request = GetRequest::Normal(GetRequestNormal {
            invoke_id: 0x00,
            class_id: 3,
            instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_id: 2,
            access_selection: None,
        });

        let encoded = request.encode();

        // Expected: C0 01 00 00 03 01 00 01 08 00 FF 02 00
        assert_eq!(encoded[0], 0xC0); // GET-Request tag
        assert_eq!(encoded[1], 0x01); // Choice: Normal
        assert_eq!(encoded[2], 0x00); // invoke_id
        assert_eq!(encoded[3..5], [0x00, 0x03]); // class_id (big-endian)
        assert_eq!(encoded[5..11], [0x01, 0x00, 0x01, 0x08, 0x00, 0xFF]); // OBIS code
        assert_eq!(encoded[11], 0x02); // attribute_id
        assert_eq!(encoded[12], 0x00); // No access selection
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_request_normal_green_book_example() {
        // Green Book Ed. 12, line 1458:
        // C0 01 00 00 03 01 00 01 08 00 FF 02
        // This is GET-Request-Normal reading Register (class 3) attribute 2

        let request = GetRequest::Normal(GetRequestNormal {
            invoke_id: 0x00,
            class_id: 3,                                    // Register
            instance_id: ObisCode::new(1, 0, 1, 8, 0, 255), // 1-0:1.8.0*255
            attribute_id: 2,                                // value attribute
            access_selection: None,
        });

        let encoded = request.encode();

        // Verify matches Green Book example (without final access selection byte)
        assert_eq!(
            &encoded[..12],
            &[0xC0, 0x01, 0x00, 0x00, 0x03, 0x01, 0x00, 0x01, 0x08, 0x00, 0xFF, 0x02]
        );
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_request_normal_with_access_selection() {
        // Test GET-Request-Normal with selective access
        let request = GetRequest::Normal(GetRequestNormal {
            invoke_id: 0x01,
            class_id: 7, // Profile Generic
            instance_id: ObisCode::new(1, 0, 99, 1, 0, 255),
            attribute_id: 2, // buffer attribute
            access_selection: Some(AccessSelector {
                selector: 1,                    // Range descriptor
                parameters: Data::Unsigned(10), // Example parameter
            }),
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC0); // GET-Request tag
        assert_eq!(encoded[1], 0x01); // Choice: Normal
        assert_eq!(encoded[2], 0x01); // invoke_id
        assert_eq!(encoded[3..5], [0x00, 0x07]); // class_id (Profile Generic)
        assert_eq!(encoded[12], 0x01); // Access selection present
        assert_eq!(encoded[13], 0x01); // Selector type
    }

    #[test]
    fn test_get_request_normal_parse_basic() {
        // Test parsing GET-Request-Normal without access selection
        let bytes = [0xC0, 0x01, 0x00, 0x00, 0x03, 0x01, 0x00, 0x01, 0x08, 0x00, 0xFF, 0x02, 0x00];

        let (remaining, request) = GetRequest::parse(&bytes).unwrap();

        assert!(remaining.is_empty());

        match request {
            GetRequest::Normal(req) => {
                assert_eq!(req.invoke_id, 0x00);
                assert_eq!(req.class_id, 3);
                assert_eq!(req.instance_id, ObisCode::new(1, 0, 1, 8, 0, 255));
                assert_eq!(req.attribute_id, 2);
                assert!(req.access_selection.is_none());
            }
            _ => panic!("Expected Normal variant"),
        }
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_request_normal_roundtrip() {
        // Test encode → parse → verify symmetry
        let original = GetRequest::Normal(GetRequestNormal {
            invoke_id: 0x42,
            class_id: 8, // Clock
            instance_id: ObisCode::new(0, 0, 1, 0, 0, 255),
            attribute_id: 2,
            access_selection: None,
        });

        let encoded = original.encode();
        let (remaining, parsed) = GetRequest::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(parsed, original);
    }

    // ========================================================================
    // GET-Request-Next Tests
    // ========================================================================

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_request_next_encode() {
        let request =
            GetRequest::NextDataBlock(GetRequestNext { invoke_id: 0x01, block_number: 0x00000002 });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC0); // GET-Request tag
        assert_eq!(encoded[1], 0x02); // Choice: NextDataBlock
        assert_eq!(encoded[2], 0x01); // invoke_id
        assert_eq!(encoded[3..7], [0x00, 0x00, 0x00, 0x02]); // block_number (big-endian)
    }

    #[test]
    fn test_get_request_next_parse() {
        let bytes = [0xC0, 0x02, 0x01, 0x00, 0x00, 0x00, 0x02];

        let (remaining, request) = GetRequest::parse(&bytes).unwrap();

        assert!(remaining.is_empty());

        match request {
            GetRequest::NextDataBlock(req) => {
                assert_eq!(req.invoke_id, 0x01);
                assert_eq!(req.block_number, 2);
            }
            _ => panic!("Expected NextDataBlock variant"),
        }
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_request_next_roundtrip() {
        let original =
            GetRequest::NextDataBlock(GetRequestNext { invoke_id: 0xFF, block_number: 0x12345678 });

        let encoded = original.encode();
        let (remaining, parsed) = GetRequest::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(parsed, original);
    }

    // ========================================================================
    // GET-Request-With-List Tests
    // ========================================================================

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_request_with_list_encode() {
        let request = GetRequest::WithList(GetRequestWithList {
            invoke_id: 0x05,
            attribute_descriptor_list: vec![
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
            ],
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC0); // GET-Request tag
        assert_eq!(encoded[1], 0x03); // Choice: WithList
        assert_eq!(encoded[2], 0x05); // invoke_id
        assert_eq!(encoded[3], 0x02); // List count
    }

    #[test]
    fn test_get_request_with_list_parse() {
        let bytes = [
            0xC0, 0x03, 0x05, 0x02, // Tag, choice, invoke_id, count
            0x00, 0x03, 0x01, 0x00, 0x01, 0x08, 0x00, 0xFF, 0x02, // First descriptor
            0x00, 0x03, 0x01, 0x00, 0x02, 0x08, 0x00, 0xFF, 0x02, // Second descriptor
        ];

        let (remaining, request) = GetRequest::parse(&bytes).unwrap();

        assert!(remaining.is_empty());

        match request {
            GetRequest::WithList(req) => {
                assert_eq!(req.invoke_id, 0x05);
                assert_eq!(req.attribute_descriptor_list.len(), 2);
                assert_eq!(req.attribute_descriptor_list[0].class_id, 3);
                assert_eq!(
                    req.attribute_descriptor_list[1].instance_id,
                    ObisCode::new(1, 0, 2, 8, 0, 255)
                );
            }
            _ => panic!("Expected WithList variant"),
        }
    }

    // ========================================================================
    // GET-Response-Normal Tests
    // ========================================================================

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_response_normal_encode_data() {
        // Test GET-Response-Normal with successful data result
        let response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0x00,
            result: GetDataResult::Data(Data::DoubleLongUnsigned(12345678)),
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC4); // GET-Response tag
        assert_eq!(encoded[1], 0x01); // Choice: Normal
        assert_eq!(encoded[2], 0x00); // invoke_id
        assert_eq!(encoded[3], 0x00); // Result choice: Data
        assert_eq!(encoded[4], 0x06); // Data type: DoubleLongUnsigned
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_response_normal_encode_error() {
        // Test GET-Response-Normal with error result
        let response = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0x01,
            result: GetDataResult::DataAccessError(DataAccessResult::ObjectUndefined),
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC4); // GET-Response tag
        assert_eq!(encoded[1], 0x01); // Choice: Normal
        assert_eq!(encoded[2], 0x01); // invoke_id
        assert_eq!(encoded[3], 0x01); // Result choice: DataAccessError
        assert_eq!(encoded[4], 0x04); // Error code: ObjectUndefined
    }

    #[test]
    fn test_get_response_normal_parse_data() {
        // Parse GET-Response-Normal with data
        let bytes = [
            0xC4, 0x01, 0x00, 0x00, // Tag, choice, invoke_id, result_choice
            0x06, 0x00, 0xBC, 0x61, 0x4E, // DoubleLongUnsigned(12345678)
        ];

        let (remaining, response) = GetResponse::parse(&bytes).unwrap();

        assert!(remaining.is_empty());

        match response {
            GetResponse::Normal(resp) => {
                assert_eq!(resp.invoke_id, 0x00);
                match resp.result {
                    GetDataResult::Data(Data::DoubleLongUnsigned(value)) => {
                        assert_eq!(value, 12345678);
                    }
                    _ => panic!("Expected Data result with DoubleLongUnsigned"),
                }
            }
            _ => panic!("Expected Normal variant"),
        }
    }

    #[test]
    fn test_get_response_normal_parse_error() {
        // Parse GET-Response-Normal with error
        let bytes = [0xC4, 0x01, 0x01, 0x01, 0x04];

        let (remaining, response) = GetResponse::parse(&bytes).unwrap();

        assert!(remaining.is_empty());

        match response {
            GetResponse::Normal(resp) => {
                assert_eq!(resp.invoke_id, 0x01);
                match resp.result {
                    GetDataResult::DataAccessError(error) => {
                        assert_eq!(error, DataAccessResult::ObjectUndefined);
                    }
                    _ => panic!("Expected DataAccessError result"),
                }
            }
            _ => panic!("Expected Normal variant"),
        }
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_response_normal_roundtrip_data() {
        let original = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0x42,
            result: GetDataResult::Data(Data::Long(12345)),
        });

        let encoded = original.encode();
        let (remaining, parsed) = GetResponse::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_response_normal_roundtrip_error() {
        let original = GetResponse::Normal(GetResponseNormal {
            invoke_id: 0x99,
            result: GetDataResult::DataAccessError(DataAccessResult::ReadWriteDenied),
        });

        let encoded = original.encode();
        let (remaining, parsed) = GetResponse::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(parsed, original);
    }

    // ========================================================================
    // GET-Response-With-List Tests
    // ========================================================================

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_response_with_list_encode() {
        let response = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0x05,
            results: vec![
                GetDataResult::Data(Data::Long(100)),
                GetDataResult::Data(Data::Long(200)),
                GetDataResult::DataAccessError(DataAccessResult::ObjectUnavailable),
            ],
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC4); // GET-Response tag
        assert_eq!(encoded[1], 0x03); // Choice: WithList
        assert_eq!(encoded[2], 0x05); // invoke_id
        assert_eq!(encoded[3], 0x03); // Result count
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_response_with_list_roundtrip() {
        let original = GetResponse::WithList(GetResponseWithList {
            invoke_id: 0x10,
            results: vec![
                GetDataResult::Data(Data::Unsigned(42)),
                GetDataResult::DataAccessError(DataAccessResult::TemporaryFailure),
            ],
        });

        let encoded = original.encode();
        let (remaining, parsed) = GetResponse::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(parsed, original);
    }

    // ========================================================================
    // Edge Cases and Error Handling
    // ========================================================================

    #[test]
    fn test_get_request_parse_invalid_tag() {
        let bytes = [0xFF, 0x01, 0x00]; // Invalid tag

        assert!(GetRequest::parse(&bytes).is_err());
    }

    #[test]
    fn test_get_request_parse_invalid_choice() {
        let bytes = [0xC0, 0xFF, 0x00]; // Invalid choice

        assert!(GetRequest::parse(&bytes).is_err());
    }

    #[test]
    fn test_get_response_parse_invalid_error_code() {
        let bytes = [0xC4, 0x01, 0x00, 0x01, 0xFF]; // Invalid error code

        assert!(GetResponse::parse(&bytes).is_err());
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_get_request_empty_list() {
        let request = GetRequest::WithList(GetRequestWithList {
            invoke_id: 0x01,
            attribute_descriptor_list: vec![],
        });

        let encoded = request.encode();
        let (remaining, parsed) = GetRequest::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        match parsed {
            GetRequest::WithList(req) => {
                assert_eq!(req.attribute_descriptor_list.len(), 0);
            }
            _ => panic!("Expected WithList variant"),
        }
    }
}
