//! SET service implementation for DLMS/COSEM protocol
//!
//! This module implements the SET service as specified in DLMS Green Book Ed. 12.
//! The SET service allows writing attributes to COSEM objects on a DLMS server.
//!
//! # APDU Tags
//! - SET-Request: 0xC1 (193)
//! - SET-Response: 0xC5 (197)
//!
//! # Green Book References
//! - Table 71: Service parameters of the SET service
//! - Table 72: SET service request and response types
//! - Table 96: SET service types and APDUs
//! - Table 100: Mapping between the SET and the Write service
//! - Figure 125: MSC of the SET service
//! - Figure 126: MSC of the SET service with block transfer
//!
//! # Examples
//!
//! ## Creating a SET-Request-Normal
//! ```
//! use dlms_cosem::set::{SetRequest, SetRequestNormal};
//! use dlms_cosem::{ObisCode, Data};
//!
//! let request = SetRequest::Normal(SetRequestNormal {
//!     invoke_id: 0x01,
//!     class_id: 3,  // Register
//!     instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
//!     attribute_id: 2,
//!     access_selection: None,
//!     value: Data::DoubleLongUnsigned(12345),
//! });
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;

use crate::data::Data;
use crate::obis_code::ObisCode;

#[cfg(feature = "encode")]
use crate::data::ByteBuffer;

// Re-export types from get.rs that are shared
pub use crate::get::{AccessSelector, AttributeDescriptor, DataAccessResult};

/// SET service request types
///
/// As specified in Green Book Table 72.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SetRequest {
    /// SET-Request-Normal: Write a single attribute (choice 0x01)
    Normal(SetRequestNormal),
    /// SET-Request-With-First-Datablock: Start block transfer (choice 0x02)
    FirstDataBlock(SetRequestFirstDataBlock),
    /// SET-Request-With-Datablock: Continue block transfer (choice 0x03)
    WithDataBlock(SetRequestWithDataBlock),
    /// SET-Request-With-List: Write multiple attributes (choice 0x04)
    WithList(SetRequestWithList),
}

/// SET-Request-Normal: Write a single COSEM attribute
///
/// Encoding format:
/// ```text
/// C1 01 [invoke_id] [class_id:2] [instance_id:6] [attr_id] [access_sel?] [value]
/// │  │  │           │             │               │          │            └─ Data value (A-XDR)
/// │  │  │           │             │               │          └──────────── Optional access selector
/// │  │  │           │             │               └─────────────────────── attribute_id (1 byte, signed)
/// │  │  │           │             └─────────────────────────────────────── instance_id (OBIS code, 6 bytes)
/// │  │  │           └───────────────────────────────────────────────────── class_id (2 bytes, big-endian)
/// │  │  └───────────────────────────────────────────────────────────────── invoke_id (1 byte)
/// │  └──────────────────────────────────────────────────────────────────── choice: Normal (0x01)
/// └─────────────────────────────────────────────────────────────────────── tag: SET-Request (0xC1)
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetRequestNormal {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// COSEM interface class ID (2 bytes, big-endian)
    pub class_id: u16,
    /// Logical name (OBIS code, 6 bytes)
    pub instance_id: ObisCode,
    /// Attribute index (1 byte, signed)
    pub attribute_id: i8,
    /// Optional selective access parameters
    pub access_selection: Option<AccessSelector>,
    /// Value to write (A-XDR encoded)
    pub value: Data,
}

/// SET-Request-With-First-Datablock: Start block transfer for large values
///
/// Used when the value to write is too large for a single APDU.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetRequestFirstDataBlock {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// COSEM interface class ID (2 bytes, big-endian)
    pub class_id: u16,
    /// Logical name (OBIS code, 6 bytes)
    pub instance_id: ObisCode,
    /// Attribute index (1 byte, signed)
    pub attribute_id: i8,
    /// Optional selective access parameters
    pub access_selection: Option<AccessSelector>,
    /// Last block indicator (boolean)
    pub last_block: bool,
    /// Block number (4 bytes, big-endian, unsigned)
    pub block_number: u32,
    /// Raw data block (octet-string)
    pub raw_data: Vec<u8>,
}

/// SET-Request-With-Datablock: Continue block transfer
///
/// Used to send subsequent blocks after FirstDataBlock.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetRequestWithDataBlock {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// Last block indicator (boolean)
    pub last_block: bool,
    /// Block number (4 bytes, big-endian, unsigned)
    pub block_number: u32,
    /// Raw data block (octet-string)
    pub raw_data: Vec<u8>,
}

/// SET-Request-With-List: Write multiple attributes
///
/// Allows writing multiple attributes in a single request.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetRequestWithList {
    /// Invoke ID and priority (1 byte)
    pub invoke_id: u8,
    /// List of attribute descriptors
    pub attribute_descriptor_list: Vec<AttributeDescriptor>,
    /// List of values to write (one per descriptor)
    pub value_list: Vec<Data>,
}

/// SET service response types
///
/// As specified in Green Book Table 72.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum SetResponse {
    /// SET-Response-Normal: Result of single attribute write (choice 0x01)
    Normal(SetResponseNormal),
    /// SET-Response-Datablock: Block transfer acknowledgment (choice 0x02)
    DataBlock(SetResponseDataBlock),
    /// SET-Response-Last-Datablock: Last block acknowledgment (choice 0x03)
    LastDataBlock(SetResponseLastDataBlock),
    /// SET-Response-Last-Datablock-With-List: Last block with list (choice 0x04)
    LastDataBlockWithList(SetResponseLastDataBlockWithList),
    /// SET-Response-With-List: Multiple results (choice 0x05)
    WithList(SetResponseWithList),
}

/// SET-Response-Normal: Result of writing a single attribute
///
/// Encoding format:
/// ```text
/// C5 01 [invoke_id] [result]
/// │  │  │           └─ DataAccessResult (1 byte)
/// │  │  └──────────── invoke_id (1 byte)
/// │  └─────────────── choice: Normal (0x01)
/// └────────────────── tag: SET-Response (0xC5)
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetResponseNormal {
    /// Invoke ID (must match request)
    pub invoke_id: u8,
    /// Result of the write operation
    pub result: DataAccessResult,
}

/// SET-Response-Datablock: Block transfer acknowledgment
///
/// Acknowledges receipt of a data block and requests next block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetResponseDataBlock {
    /// Invoke ID (must match request)
    pub invoke_id: u8,
    /// Block number acknowledged (4 bytes, big-endian, unsigned)
    pub block_number: u32,
}

/// SET-Response-Last-Datablock: Acknowledgment of last data block
///
/// Indicates successful completion of block transfer and provides final result.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetResponseLastDataBlock {
    /// Invoke ID (must match request)
    pub invoke_id: u8,
    /// Result of the write operation
    pub result: DataAccessResult,
    /// Block number of last block (4 bytes, big-endian, unsigned)
    pub block_number: u32,
}

/// SET-Response-Last-Datablock-With-List: Last block with list results
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetResponseLastDataBlockWithList {
    /// Invoke ID (must match request)
    pub invoke_id: u8,
    /// Results for each attribute
    pub results: Vec<DataAccessResult>,
    /// Block number of last block (4 bytes, big-endian, unsigned)
    pub block_number: u32,
}

/// SET-Response-With-List: Results for multiple attributes
///
/// Contains one result per attribute in the request.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SetResponseWithList {
    /// Invoke ID (must match request)
    pub invoke_id: u8,
    /// Results for each attribute (one per write operation)
    pub results: Vec<DataAccessResult>,
}

// ============================================================================
// Encoding Implementation
// ============================================================================

#[cfg(feature = "encode")]
impl SetRequest {
    /// Encode SET-Request to bytes
    ///
    /// Returns a vector containing the complete APDU with tag and choice.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(0xC1); // SET-Request tag

        match self {
            SetRequest::Normal(req) => {
                buf.push(0x01); // choice: Normal
                buf.push(req.invoke_id);
                buf.push_u16(req.class_id); // big-endian
                buf.extend_from_slice(&req.instance_id.encode());
                buf.push(req.attribute_id as u8);

                // Access selection (optional)
                if let Some(selector) = &req.access_selection {
                    buf.push(0x01); // access-selection present
                    buf.push(selector.selector);
                    buf.extend_from_slice(&selector.parameters.encode());
                } else {
                    buf.push(0x00); // no access-selection
                }

                // Value (A-XDR encoded)
                buf.extend_from_slice(&req.value.encode());
            }
            SetRequest::FirstDataBlock(req) => {
                buf.push(0x02); // choice: FirstDataBlock
                buf.push(req.invoke_id);
                buf.push_u16(req.class_id);
                buf.extend_from_slice(&req.instance_id.encode());
                buf.push(req.attribute_id as u8);

                // Access selection (optional)
                if let Some(selector) = &req.access_selection {
                    buf.push(0x01);
                    buf.push(selector.selector);
                    buf.extend_from_slice(&selector.parameters.encode());
                } else {
                    buf.push(0x00);
                }

                // Last block indicator
                buf.push(if req.last_block { 0x01 } else { 0x00 });
                // Block number (4 bytes, big-endian)
                buf.push_u32(req.block_number);
                // Raw data as octet-string
                buf.push(0x09); // octet-string tag
                buf.push(req.raw_data.len() as u8);
                buf.extend_from_slice(&req.raw_data);
            }
            SetRequest::WithDataBlock(req) => {
                buf.push(0x03); // choice: WithDataBlock
                buf.push(req.invoke_id);
                buf.push(if req.last_block { 0x01 } else { 0x00 });
                buf.push_u32(req.block_number);
                buf.push(0x09); // octet-string tag
                buf.push(req.raw_data.len() as u8);
                buf.extend_from_slice(&req.raw_data);
            }
            SetRequest::WithList(req) => {
                buf.push(0x04); // choice: WithList
                buf.push(req.invoke_id);

                // Attribute descriptor list (array)
                buf.push(0x01); // array tag
                buf.push(req.attribute_descriptor_list.len() as u8);
                for desc in &req.attribute_descriptor_list {
                    buf.push(0x02); // structure tag
                    buf.push(0x04); // 4 elements
                    buf.push_u16(desc.class_id);
                    buf.extend_from_slice(&desc.instance_id.encode());
                    buf.push(desc.attribute_id as u8);
                    buf.push(0x00); // no access-selection for list items
                }

                // Value list (array)
                buf.push(0x01); // array tag
                buf.push(req.value_list.len() as u8);
                for value in &req.value_list {
                    buf.extend_from_slice(&value.encode());
                }
            }
        }

        buf
    }
}

#[cfg(feature = "encode")]
impl SetResponse {
    /// Encode SET-Response to bytes
    ///
    /// Returns a vector containing the complete APDU with tag and choice.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(0xC5); // SET-Response tag

        match self {
            SetResponse::Normal(resp) => {
                buf.push(0x01); // choice: Normal
                buf.push(resp.invoke_id);
                buf.push(resp.result as u8);
            }
            SetResponse::DataBlock(resp) => {
                buf.push(0x02); // choice: DataBlock
                buf.push(resp.invoke_id);
                buf.push_u32(resp.block_number);
            }
            SetResponse::LastDataBlock(resp) => {
                buf.push(0x03); // choice: LastDataBlock
                buf.push(resp.invoke_id);
                buf.push(resp.result as u8);
                buf.push_u32(resp.block_number);
            }
            SetResponse::LastDataBlockWithList(resp) => {
                buf.push(0x04); // choice: LastDataBlockWithList
                buf.push(resp.invoke_id);
                // Results array
                buf.push(0x01); // array tag
                buf.push(resp.results.len() as u8);
                for result in &resp.results {
                    buf.push(*result as u8);
                }
                buf.push_u32(resp.block_number);
            }
            SetResponse::WithList(resp) => {
                buf.push(0x05); // choice: WithList
                buf.push(resp.invoke_id);
                // Results array
                buf.push(0x01); // array tag
                buf.push(resp.results.len() as u8);
                for result in &resp.results {
                    buf.push(*result as u8);
                }
            }
        }

        buf
    }
}

// ============================================================================
// Parsing Implementation
// ============================================================================

impl SetRequest {
    /// Parse SET-Request from bytes
    ///
    /// Expects input starting with the tag byte (0xC1).
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> nom::IResult<&[u8], Self> {
        use nom::number::streaming::{be_u16, be_u32, u8 as nom_u8};

        let (input, tag) = nom_u8(input)?;
        if tag != 0xC1 {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        let (input, choice) = nom_u8(input)?;

        match choice {
            0x01 => {
                // SET-Request-Normal
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

                let (input, value) = Data::parse(input)?;

                Ok((
                    input,
                    SetRequest::Normal(SetRequestNormal {
                        invoke_id,
                        class_id,
                        instance_id,
                        attribute_id: attribute_id as i8,
                        access_selection,
                        value,
                    }),
                ))
            }
            0x02 => {
                // SET-Request-FirstDataBlock
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

                let (input, last_block) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;

                // Parse octet-string
                let (input, tag) = nom_u8(input)?;
                if tag != 0x09 {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
                let (input, len) = nom_u8(input)?;
                let (input, raw_data) = nom::bytes::streaming::take(len as usize)(input)?;

                Ok((
                    input,
                    SetRequest::FirstDataBlock(SetRequestFirstDataBlock {
                        invoke_id,
                        class_id,
                        instance_id,
                        attribute_id: attribute_id as i8,
                        access_selection,
                        last_block: last_block != 0x00,
                        block_number,
                        raw_data: raw_data.to_vec(),
                    }),
                ))
            }
            0x03 => {
                // SET-Request-WithDataBlock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, last_block) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;

                let (input, tag) = nom_u8(input)?;
                if tag != 0x09 {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
                let (input, len) = nom_u8(input)?;
                let (input, raw_data) = nom::bytes::streaming::take(len as usize)(input)?;

                Ok((
                    input,
                    SetRequest::WithDataBlock(SetRequestWithDataBlock {
                        invoke_id,
                        last_block: last_block != 0x00,
                        block_number,
                        raw_data: raw_data.to_vec(),
                    }),
                ))
            }
            0x04 => {
                // SET-Request-WithList
                let (input, invoke_id) = nom_u8(input)?;

                // Parse attribute descriptor list
                let (input, array_tag) = nom_u8(input)?;
                if array_tag != 0x01 {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
                let (input, count) = nom_u8(input)?;
                let descriptor_count = count as usize;

                let mut remaining = input;
                let mut descriptors = Vec::with_capacity(descriptor_count);
                for _ in 0..descriptor_count {
                    let (input, struct_tag) = nom_u8(remaining)?;
                    if struct_tag != 0x02 {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                    let (input, elem_count) = nom_u8(input)?;
                    if elem_count != 0x04 {
                        return Err(nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Tag,
                        )));
                    }
                    let (input, class_id) = be_u16(input)?;
                    let (input, instance_id) = ObisCode::parse(input)?;
                    let (input, attribute_id) = nom_u8(input)?;
                    let (input, _) = nom_u8(input)?; // access-selection (ignored for list)

                    descriptors.push(AttributeDescriptor {
                        class_id,
                        instance_id,
                        attribute_id: attribute_id as i8,
                    });
                    remaining = input;
                }

                // Parse value list
                let (input, array_tag) = nom_u8(remaining)?;
                if array_tag != 0x01 {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
                let (input, count) = nom_u8(input)?;
                let value_count = count as usize;

                let mut remaining = input;
                let mut values = Vec::with_capacity(value_count);
                for _ in 0..value_count {
                    let (input, value) = Data::parse(remaining)?;
                    values.push(value);
                    remaining = input;
                }

                Ok((
                    remaining,
                    SetRequest::WithList(SetRequestWithList {
                        invoke_id,
                        attribute_descriptor_list: descriptors,
                        value_list: values,
                    }),
                ))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
        }
    }
}

impl SetResponse {
    /// Parse SET-Response from bytes
    ///
    /// Expects input starting with the tag byte (0xC5).
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> nom::IResult<&[u8], Self> {
        use nom::number::streaming::{be_u32, u8 as nom_u8};

        let (input, tag) = nom_u8(input)?;
        if tag != 0xC5 {
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)));
        }
        let (input, choice) = nom_u8(input)?;

        match choice {
            0x01 => {
                // SET-Response-Normal
                let (input, invoke_id) = nom_u8(input)?;
                let (input, result_byte) = nom_u8(input)?;
                let result = DataAccessResult::from_u8(result_byte).ok_or_else(|| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
                })?;

                Ok((input, SetResponse::Normal(SetResponseNormal { invoke_id, result })))
            }
            0x02 => {
                // SET-Response-DataBlock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, block_number) = be_u32(input)?;

                Ok((
                    input,
                    SetResponse::DataBlock(SetResponseDataBlock { invoke_id, block_number }),
                ))
            }
            0x03 => {
                // SET-Response-LastDataBlock
                let (input, invoke_id) = nom_u8(input)?;
                let (input, result_byte) = nom_u8(input)?;
                let result = DataAccessResult::from_u8(result_byte).ok_or_else(|| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
                })?;
                let (input, block_number) = be_u32(input)?;

                Ok((
                    input,
                    SetResponse::LastDataBlock(SetResponseLastDataBlock {
                        invoke_id,
                        result,
                        block_number,
                    }),
                ))
            }
            0x04 => {
                // SET-Response-LastDataBlockWithList
                let (input, invoke_id) = nom_u8(input)?;
                let (input, array_tag) = nom_u8(input)?;
                if array_tag != 0x01 {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
                let (input, count) = nom_u8(input)?;
                let result_count = count as usize;

                let mut remaining = input;
                let mut results = Vec::with_capacity(result_count);
                for _ in 0..result_count {
                    let (input, result_byte) = nom_u8(remaining)?;
                    let result = DataAccessResult::from_u8(result_byte).ok_or_else(|| {
                        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
                    })?;
                    results.push(result);
                    remaining = input;
                }

                let (input, block_number) = be_u32(remaining)?;

                Ok((
                    input,
                    SetResponse::LastDataBlockWithList(SetResponseLastDataBlockWithList {
                        invoke_id,
                        results,
                        block_number,
                    }),
                ))
            }
            0x05 => {
                // SET-Response-WithList
                let (input, invoke_id) = nom_u8(input)?;
                let (input, array_tag) = nom_u8(input)?;
                if array_tag != 0x01 {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
                let (input, count) = nom_u8(input)?;
                let result_count = count as usize;

                let mut remaining = input;
                let mut results = Vec::with_capacity(result_count);
                for _ in 0..result_count {
                    let (input, result_byte) = nom_u8(remaining)?;
                    let result = DataAccessResult::from_u8(result_byte).ok_or_else(|| {
                        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
                    })?;
                    results.push(result);
                    remaining = input;
                }

                Ok((remaining, SetResponse::WithList(SetResponseWithList { invoke_id, results })))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
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
    fn test_set_request_normal_encode_basic() {
        let request = SetRequest::Normal(SetRequestNormal {
            invoke_id: 0x01,
            class_id: 3, // Register
            instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_id: 2,
            access_selection: None,
            value: Data::DoubleLongUnsigned(12345),
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC1); // SET-Request tag
        assert_eq!(encoded[1], 0x01); // choice: Normal
        assert_eq!(encoded[2], 0x01); // invoke_id
        assert_eq!(u16::from_be_bytes([encoded[3], encoded[4]]), 3); // class_id
        assert_eq!(&encoded[5..11], &[1, 0, 1, 8, 0, 255]); // OBIS code
        assert_eq!(encoded[11], 2); // attribute_id
        assert_eq!(encoded[12], 0x00); // no access selection
        assert_eq!(encoded[13], 0x06); // DoubleLongUnsigned tag
        assert_eq!(u32::from_be_bytes([encoded[14], encoded[15], encoded[16], encoded[17]]), 12345);
    }

    #[test]
    fn test_set_request_normal_with_access_selection() {
        let request = SetRequest::Normal(SetRequestNormal {
            invoke_id: 0x02,
            class_id: 7, // Profile Generic
            instance_id: ObisCode::new(1, 0, 99, 1, 0, 255),
            attribute_id: 2,
            access_selection: Some(AccessSelector {
                selector: 1,
                parameters: Data::DoubleLongUnsigned(100),
            }),
            value: Data::LongUnsigned(500),
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC1);
        assert_eq!(encoded[1], 0x01);
        assert_eq!(encoded[2], 0x02); // invoke_id
        assert_eq!(encoded[12], 0x01); // access selection present
        assert_eq!(encoded[13], 1); // selector
        assert_eq!(encoded[14], 0x06); // DoubleLongUnsigned tag for parameters
    }

    #[test]
    fn test_set_request_normal_parse_basic() {
        let bytes = vec![
            0xC1, 0x01, // tag, choice
            0x01, // invoke_id
            0x00, 0x03, // class_id = 3
            0x01, 0x00, 0x01, 0x08, 0x00, 0xFF, // OBIS
            0x02, // attribute_id
            0x00, // no access selection
            0x06, 0x00, 0x00, 0x30, 0x39, // DoubleLongUnsigned(12345)
        ];

        let (remaining, request) = SetRequest::parse(&bytes).unwrap();
        assert!(remaining.is_empty());

        match request {
            SetRequest::Normal(req) => {
                assert_eq!(req.invoke_id, 0x01);
                assert_eq!(req.class_id, 3);
                assert_eq!(req.attribute_id, 2);
                assert!(req.access_selection.is_none());
                assert_eq!(req.value, Data::DoubleLongUnsigned(12345));
            }
            _ => panic!("Expected Normal variant"),
        }
    }

    #[test]
    fn test_set_request_normal_roundtrip() {
        let original = SetRequest::Normal(SetRequestNormal {
            invoke_id: 0x05,
            class_id: 3,
            instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
            attribute_id: 2,
            access_selection: None,
            value: Data::DoubleLongUnsigned(999),
        });

        let encoded = original.encode();
        let (remaining, decoded) = SetRequest::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_set_request_with_datablock_encode() {
        let request = SetRequest::WithDataBlock(SetRequestWithDataBlock {
            invoke_id: 0x03,
            last_block: false,
            block_number: 1,
            raw_data: vec![0x01, 0x02, 0x03, 0x04],
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC1); // SET-Request tag
        assert_eq!(encoded[1], 0x03); // choice: WithDataBlock
        assert_eq!(encoded[2], 0x03); // invoke_id
        assert_eq!(encoded[3], 0x00); // not last block
        assert_eq!(u32::from_be_bytes([encoded[4], encoded[5], encoded[6], encoded[7]]), 1); // block_number
        assert_eq!(encoded[8], 0x09); // octet-string tag
        assert_eq!(encoded[9], 4); // length
        assert_eq!(&encoded[10..14], &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_set_request_with_datablock_roundtrip() {
        let original = SetRequest::WithDataBlock(SetRequestWithDataBlock {
            invoke_id: 0x04,
            last_block: true,
            block_number: 5,
            raw_data: vec![0xAA, 0xBB, 0xCC],
        });

        let encoded = original.encode();
        let (remaining, decoded) = SetRequest::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_set_request_with_list_encode() {
        let request = SetRequest::WithList(SetRequestWithList {
            invoke_id: 0x06,
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
            value_list: vec![Data::DoubleLongUnsigned(100), Data::DoubleLongUnsigned(200)],
        });

        let encoded = request.encode();

        assert_eq!(encoded[0], 0xC1);
        assert_eq!(encoded[1], 0x04); // choice: WithList
        assert_eq!(encoded[2], 0x06); // invoke_id
        assert_eq!(encoded[3], 0x01); // array tag
        assert_eq!(encoded[4], 2); // 2 descriptors
    }

    #[test]
    fn test_set_request_with_list_roundtrip() {
        let original = SetRequest::WithList(SetRequestWithList {
            invoke_id: 0x07,
            attribute_descriptor_list: vec![AttributeDescriptor {
                class_id: 3,
                instance_id: ObisCode::new(1, 0, 1, 8, 0, 255),
                attribute_id: 2,
            }],
            value_list: vec![Data::DoubleLongUnsigned(500)],
        });

        let encoded = original.encode();
        let (remaining, decoded) = SetRequest::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_set_response_normal_encode_success() {
        let response = SetResponse::Normal(SetResponseNormal {
            invoke_id: 0x01,
            result: DataAccessResult::Success,
        });

        let encoded = response.encode();

        assert_eq!(encoded, vec![0xC5, 0x01, 0x01, 0x00]);
    }

    #[test]
    fn test_set_response_normal_encode_error() {
        let response = SetResponse::Normal(SetResponseNormal {
            invoke_id: 0x02,
            result: DataAccessResult::ReadWriteDenied,
        });

        let encoded = response.encode();

        assert_eq!(encoded, vec![0xC5, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_set_response_normal_parse_success() {
        let bytes = vec![0xC5, 0x01, 0x05, 0x00];

        let (remaining, response) = SetResponse::parse(&bytes).unwrap();
        assert!(remaining.is_empty());

        match response {
            SetResponse::Normal(resp) => {
                assert_eq!(resp.invoke_id, 0x05);
                assert_eq!(resp.result, DataAccessResult::Success);
            }
            _ => panic!("Expected Normal variant"),
        }
    }

    #[test]
    fn test_set_response_normal_parse_error() {
        let bytes = vec![0xC5, 0x01, 0x03, 0x03]; // ReadWriteDenied

        let (remaining, response) = SetResponse::parse(&bytes).unwrap();
        assert!(remaining.is_empty());

        match response {
            SetResponse::Normal(resp) => {
                assert_eq!(resp.invoke_id, 0x03);
                assert_eq!(resp.result, DataAccessResult::ReadWriteDenied);
            }
            _ => panic!("Expected Normal variant"),
        }
    }

    #[test]
    fn test_set_response_normal_roundtrip_success() {
        let original = SetResponse::Normal(SetResponseNormal {
            invoke_id: 0x08,
            result: DataAccessResult::Success,
        });

        let encoded = original.encode();
        let (remaining, decoded) = SetResponse::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_set_response_normal_roundtrip_error() {
        let original = SetResponse::Normal(SetResponseNormal {
            invoke_id: 0x09,
            result: DataAccessResult::ObjectUndefined,
        });

        let encoded = original.encode();
        let (remaining, decoded) = SetResponse::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_set_response_datablock_encode() {
        let response =
            SetResponse::DataBlock(SetResponseDataBlock { invoke_id: 0x0A, block_number: 3 });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC5);
        assert_eq!(encoded[1], 0x02); // choice: DataBlock
        assert_eq!(encoded[2], 0x0A); // invoke_id
        assert_eq!(u32::from_be_bytes([encoded[3], encoded[4], encoded[5], encoded[6]]), 3);
    }

    #[test]
    fn test_set_response_datablock_roundtrip() {
        let original =
            SetResponse::DataBlock(SetResponseDataBlock { invoke_id: 0x0B, block_number: 10 });

        let encoded = original.encode();
        let (remaining, decoded) = SetResponse::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_set_response_with_list_encode() {
        let response = SetResponse::WithList(SetResponseWithList {
            invoke_id: 0x0C,
            results: vec![
                DataAccessResult::Success,
                DataAccessResult::Success,
                DataAccessResult::ObjectUndefined,
            ],
        });

        let encoded = response.encode();

        assert_eq!(encoded[0], 0xC5);
        assert_eq!(encoded[1], 0x05); // choice: WithList
        assert_eq!(encoded[2], 0x0C); // invoke_id
        assert_eq!(encoded[3], 0x01); // array tag
        assert_eq!(encoded[4], 3); // 3 results
        assert_eq!(encoded[5], 0x00); // Success
        assert_eq!(encoded[6], 0x00); // Success
        assert_eq!(encoded[7], 0x04); // ObjectUndefined
    }

    #[test]
    fn test_set_response_with_list_roundtrip() {
        let original = SetResponse::WithList(SetResponseWithList {
            invoke_id: 0x0D,
            results: vec![DataAccessResult::Success, DataAccessResult::ReadWriteDenied],
        });

        let encoded = original.encode();
        let (remaining, decoded) = SetResponse::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_set_request_parse_invalid_tag() {
        let bytes = vec![0xC0, 0x01]; // Wrong tag (GET-Request)
        assert!(SetRequest::parse(&bytes).is_err());
    }

    #[test]
    fn test_set_request_parse_invalid_choice() {
        let bytes = vec![0xC1, 0x99]; // Invalid choice
        assert!(SetRequest::parse(&bytes).is_err());
    }

    #[test]
    fn test_set_response_parse_invalid_error_code() {
        let bytes = vec![0xC5, 0x01, 0x01, 0xFF]; // Invalid DataAccessResult
        assert!(SetResponse::parse(&bytes).is_err());
    }

    #[test]
    fn test_set_request_first_datablock_roundtrip() {
        let original = SetRequest::FirstDataBlock(SetRequestFirstDataBlock {
            invoke_id: 0x10,
            class_id: 7,
            instance_id: ObisCode::new(1, 0, 99, 1, 0, 255),
            attribute_id: 2,
            access_selection: None,
            last_block: false,
            block_number: 1,
            raw_data: vec![0xDE, 0xAD, 0xBE, 0xEF],
        });

        let encoded = original.encode();
        let (remaining, decoded) = SetRequest::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_set_response_last_datablock_roundtrip() {
        let original = SetResponse::LastDataBlock(SetResponseLastDataBlock {
            invoke_id: 0x11,
            result: DataAccessResult::Success,
            block_number: 5,
        });

        let encoded = original.encode();
        let (remaining, decoded) = SetResponse::parse(&encoded).unwrap();

        assert!(remaining.is_empty());
        assert_eq!(decoded, original);
    }
}
