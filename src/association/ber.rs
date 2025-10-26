//! ASN.1 BER (Basic Encoding Rules) helpers
//!
//! This module provides a minimal subset of ASN.1 BER encoding/parsing
//! required for DLMS/COSEM AARQ/AARE APDUs.
//!
//! Reference: ISO/IEC 8825-1:2015, DLMS Green Book Ed. 12 Section 11

use alloc::vec::Vec;

#[cfg(feature = "parse")]
use nom::{
    IResult,
    error::{Error, ErrorKind},
    number::streaming::u8 as nom_u8,
};

// ============================================================================
// BER Tag Classes and Types
// ============================================================================

/// BER tag class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TagClass {
    /// Universal (0b00)
    Universal,
    /// Application (0b01)
    Application,
    /// Context-specific (0b10)
    ContextSpecific,
    /// Private (0b11)
    Private,
}

impl TagClass {
    /// Encode tag class to upper 2 bits
    #[cfg(feature = "encode")]
    pub const fn to_bits(self) -> u8 {
        match self {
            TagClass::Universal => 0b00_000000,
            TagClass::Application => 0b01_000000,
            TagClass::ContextSpecific => 0b10_000000,
            TagClass::Private => 0b11_000000,
        }
    }

    /// Parse tag class from upper 2 bits
    #[allow(dead_code)]
    pub const fn from_bits(byte: u8) -> Self {
        match byte & 0b11_000000 {
            0b00_000000 => TagClass::Universal,
            0b01_000000 => TagClass::Application,
            0b10_000000 => TagClass::ContextSpecific,
            _ => TagClass::Private,
        }
    }
}

/// BER tag type (primitive or constructed)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    /// Primitive (0b0)
    Primitive,
    /// Constructed (0b1) - contains nested TLVs
    Constructed,
}

impl TagType {
    /// Encode tag type to bit 5
    #[cfg(feature = "encode")]
    pub const fn to_bit(self) -> u8 {
        match self {
            TagType::Primitive => 0b0000_0000,
            TagType::Constructed => 0b0010_0000,
        }
    }

    /// Parse tag type from bit 5
    #[allow(dead_code)]
    pub const fn from_bit(byte: u8) -> Self {
        if byte & 0b0010_0000 != 0 { TagType::Constructed } else { TagType::Primitive }
    }
}

// ============================================================================
// BER Tag Encoding/Parsing
// ============================================================================

/// Encode a single-byte BER tag
///
/// # Arguments
///
/// * `class` - Tag class (Universal, Application, Context-specific, Private)
/// * `tag_type` - Primitive or Constructed
/// * `tag_number` - Tag number (0-30 for single-byte encoding)
///
/// # Returns
///
/// Single byte tag value
///
/// # Panics
///
/// Panics if tag_number > 30 (multi-byte tags not supported)
#[cfg(feature = "encode")]
pub const fn encode_tag(class: TagClass, tag_type: TagType, tag_number: u8) -> u8 {
    assert!(tag_number <= 30, "Multi-byte tags not supported");
    class.to_bits() | tag_type.to_bit() | tag_number
}

/// Parse a single-byte BER tag
///
/// Returns (class, tag_type, tag_number)
#[cfg(feature = "parse")]
#[allow(dead_code)]
pub fn parse_tag(input: &[u8]) -> IResult<&[u8], (TagClass, TagType, u8)> {
    let (input, byte) = nom_u8(input)?;
    let class = TagClass::from_bits(byte);
    let tag_type = TagType::from_bit(byte);
    let tag_number = byte & 0b000_11111;

    if tag_number == 31 {
        // Multi-byte tag not supported
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }

    Ok((input, (class, tag_type, tag_number)))
}

// ============================================================================
// BER Length Encoding/Parsing (Definite Form Only)
// ============================================================================

/// Encode BER length in definite form
///
/// # Arguments
///
/// * `length` - Content length (0 to 0xFFFF supported)
///
/// # Returns
///
/// Vector containing encoded length (1-3 bytes)
///
/// # Examples
///
/// - Length 0-127: single byte [length]
/// - Length 128-255: [0x81, length]
/// - Length 256-65535: [0x82, hi_byte, lo_byte]
#[cfg(feature = "encode")]
pub fn encode_length(length: usize) -> Vec<u8> {
    if length <= 127 {
        // Short form: single byte
        alloc::vec![length as u8]
    } else if length <= 255 {
        // Long form: 1 byte length
        alloc::vec![0x81, length as u8]
    } else if length <= 65535 {
        // Long form: 2 byte length (big-endian)
        alloc::vec![0x82, (length >> 8) as u8, (length & 0xFF) as u8]
    } else {
        // Longer forms not supported in this implementation
        panic!("Length {} too large (max 65535)", length);
    }
}

/// Parse BER length in definite form
///
/// Returns the parsed length and remaining input
#[cfg(feature = "parse")]
pub fn parse_length(input: &[u8]) -> IResult<&[u8], usize> {
    let (input, first_byte) = nom_u8(input)?;

    if first_byte & 0x80 == 0 {
        // Short form: length is in the first byte
        Ok((input, first_byte as usize))
    } else {
        // Long form: first byte indicates number of length octets
        let num_octets = (first_byte & 0x7F) as usize;

        if num_octets == 0 {
            // Indefinite form not supported
            return Err(nom::Err::Error(Error::new(input, ErrorKind::LengthValue)));
        }

        if num_octets > 2 {
            // Only support up to 2-byte lengths (max 65535)
            return Err(nom::Err::Error(Error::new(input, ErrorKind::LengthValue)));
        }

        // Parse length bytes (big-endian)
        let mut length = 0usize;
        let mut remaining = input;
        for _ in 0..num_octets {
            let (rest, byte) = nom_u8(remaining)?;
            length = (length << 8) | (byte as usize);
            remaining = rest;
        }

        Ok((remaining, length))
    }
}

// ============================================================================
// Common BER Type Encoding
// ============================================================================

/// Encode an OBJECT IDENTIFIER (OID) in BER format
///
/// # Arguments
///
/// * `oid_bytes` - Pre-encoded OID bytes (DLMS uses fixed OIDs)
///
/// # Returns
///
/// Complete TLV: [0x06, length, ...oid_bytes]
#[cfg(feature = "encode")]
pub fn encode_object_identifier(oid_bytes: &[u8]) -> Vec<u8> {
    let mut result = alloc::vec![0x06]; // Universal Primitive OBJECT IDENTIFIER
    result.extend(encode_length(oid_bytes.len()));
    result.extend_from_slice(oid_bytes);
    result
}

/// Encode a BIT STRING in BER format
///
/// # Arguments
///
/// * `bits` - Bit string value
/// * `unused_bits` - Number of unused bits in last byte (0-7)
///
/// # Returns
///
/// Complete TLV: [0x03, length, unused_bits, ...bits]
#[cfg(feature = "encode")]
pub fn encode_bit_string(bits: &[u8], unused_bits: u8) -> Vec<u8> {
    let mut result = alloc::vec![0x03]; // Universal Primitive BIT STRING
    result.extend(encode_length(bits.len() + 1)); // +1 for unused_bits byte
    result.push(unused_bits);
    result.extend_from_slice(bits);
    result
}

/// Encode an OCTET STRING in BER format
///
/// # Arguments
///
/// * `octets` - Octet string value
///
/// # Returns
///
/// Complete TLV: [0x04, length, ...octets]
#[cfg(feature = "encode")]
pub fn encode_octet_string(octets: &[u8]) -> Vec<u8> {
    let mut result = alloc::vec![0x04]; // Universal Primitive OCTET STRING
    result.extend(encode_length(octets.len()));
    result.extend_from_slice(octets);
    result
}

/// Encode a context-specific tag with content
///
/// # Arguments
///
/// * `tag_number` - Context-specific tag number (0-30)
/// * `tag_type` - Primitive or Constructed
/// * `content` - Content bytes
///
/// # Returns
///
/// Complete TLV: [tag, length, ...content]
///
/// # Examples
///
/// - Context[0]: tag_number=0, Constructed → 0xA0
/// - Context[1]: tag_number=1, Constructed → 0xA1
#[cfg(feature = "encode")]
pub fn encode_context_specific(tag_number: u8, tag_type: TagType, content: &[u8]) -> Vec<u8> {
    let tag = encode_tag(TagClass::ContextSpecific, tag_type, tag_number);
    let mut result = alloc::vec![tag];
    result.extend(encode_length(content.len()));
    result.extend_from_slice(content);
    result
}

/// Encode a SEQUENCE in BER format (Universal Constructed 16)
///
/// # Arguments
///
/// * `content` - Pre-encoded sequence elements
///
/// # Returns
///
/// Complete TLV: [0x30, length, ...content]
#[cfg(feature = "encode")]
#[allow(dead_code)]
pub fn encode_sequence(content: &[u8]) -> Vec<u8> {
    let tag = encode_tag(TagClass::Universal, TagType::Constructed, 16);
    let mut result = alloc::vec![tag]; // 0x30
    result.extend(encode_length(content.len()));
    result.extend_from_slice(content);
    result
}

/// Encode an APPLICATION tag with content
///
/// # Arguments
///
/// * `tag_number` - Application tag number (0-30)
/// * `tag_type` - Primitive or Constructed
/// * `content` - Content bytes
///
/// # Returns
///
/// Complete TLV: [tag, length, ...content]
///
/// # Examples
///
/// - APPLICATION[0]: tag_number=0, Constructed → 0x60 (AARQ)
/// - APPLICATION[1]: tag_number=1, Constructed → 0x61 (AARE)
#[cfg(feature = "encode")]
pub fn encode_application(tag_number: u8, tag_type: TagType, content: &[u8]) -> Vec<u8> {
    let tag = encode_tag(TagClass::Application, tag_type, tag_number);
    let mut result = alloc::vec![tag];
    result.extend(encode_length(content.len()));
    result.extend_from_slice(content);
    result
}

// ============================================================================
// Common BER Type Parsing
// ============================================================================

/// Parse an OBJECT IDENTIFIER from BER format
///
/// Returns the OID bytes (without tag/length)
#[cfg(feature = "parse")]
#[allow(dead_code)]
pub fn parse_object_identifier(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, tag) = nom_u8(input)?;
    if tag != 0x06 {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, length) = parse_length(input)?;

    if input.len() < length {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    let oid_bytes = input[..length].to_vec();
    Ok((&input[length..], oid_bytes))
}

/// Parse a BIT STRING from BER format
///
/// Returns (bits, unused_bits)
#[cfg(feature = "parse")]
#[allow(dead_code)]
pub fn parse_bit_string(input: &[u8]) -> IResult<&[u8], (Vec<u8>, u8)> {
    let (input, tag) = nom_u8(input)?;
    if tag != 0x03 {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, length) = parse_length(input)?;

    if length < 1 || input.len() < length {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    let (input, unused_bits) = nom_u8(input)?;
    let bits = input[..(length - 1)].to_vec();
    Ok((&input[(length - 1)..], (bits, unused_bits)))
}

/// Parse an OCTET STRING from BER format
///
/// Returns the octet string bytes
#[cfg(feature = "parse")]
#[allow(dead_code)]
pub fn parse_octet_string(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, tag) = nom_u8(input)?;
    if tag != 0x04 {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, length) = parse_length(input)?;

    if input.len() < length {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    let octets = input[..length].to_vec();
    Ok((&input[length..], octets))
}

/// Parse a context-specific tag
///
/// Returns the content bytes (without tag/length)
#[cfg(feature = "parse")]
#[allow(dead_code)]
pub fn parse_context_specific(input: &[u8], expected_tag_number: u8) -> IResult<&[u8], Vec<u8>> {
    let (input, (class, _tag_type, tag_number)) = parse_tag(input)?;

    if class != TagClass::ContextSpecific || tag_number != expected_tag_number {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }

    let (input, length) = parse_length(input)?;

    if input.len() < length {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    let content = input[..length].to_vec();
    Ok((&input[length..], content))
}

/// Parse a SEQUENCE from BER format
///
/// Returns the sequence content bytes (without tag/length)
#[cfg(feature = "parse")]
#[allow(dead_code)]
pub fn parse_sequence(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, tag) = nom_u8(input)?;
    if tag != 0x30 {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
    }
    let (input, length) = parse_length(input)?;

    if input.len() < length {
        return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    let content = input[..length].to_vec();
    Ok((&input[length..], content))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_tag() {
        // Universal Primitive NULL (tag 5)
        assert_eq!(encode_tag(TagClass::Universal, TagType::Primitive, 5), 0x05);

        // Universal Primitive OBJECT IDENTIFIER (tag 6)
        assert_eq!(encode_tag(TagClass::Universal, TagType::Primitive, 6), 0x06);

        // Universal Constructed SEQUENCE (tag 16)
        assert_eq!(encode_tag(TagClass::Universal, TagType::Constructed, 16), 0x30);

        // Application Constructed 0 (AARQ)
        assert_eq!(encode_tag(TagClass::Application, TagType::Constructed, 0), 0x60);

        // Application Constructed 1 (AARE)
        assert_eq!(encode_tag(TagClass::Application, TagType::Constructed, 1), 0x61);

        // Context-specific Constructed 0
        assert_eq!(encode_tag(TagClass::ContextSpecific, TagType::Constructed, 0), 0xA0);

        // Context-specific Constructed 1
        assert_eq!(encode_tag(TagClass::ContextSpecific, TagType::Constructed, 1), 0xA1);
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_parse_tag() {
        // AARQ tag 0x60
        let (_, (class, tag_type, num)) = parse_tag(&[0x60]).unwrap();
        assert_eq!(class, TagClass::Application);
        assert_eq!(tag_type, TagType::Constructed);
        assert_eq!(num, 0);

        // Context[0] tag 0xA0
        let (_, (class, tag_type, num)) = parse_tag(&[0xA0]).unwrap();
        assert_eq!(class, TagClass::ContextSpecific);
        assert_eq!(tag_type, TagType::Constructed);
        assert_eq!(num, 0);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_length_short() {
        // Short form (0-127)
        assert_eq!(encode_length(0), vec![0x00]);
        assert_eq!(encode_length(5), vec![0x05]);
        assert_eq!(encode_length(127), vec![0x7F]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_length_long_1byte() {
        // Long form, 1 byte
        assert_eq!(encode_length(128), vec![0x81, 0x80]);
        assert_eq!(encode_length(255), vec![0x81, 0xFF]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_length_long_2byte() {
        // Long form, 2 bytes
        assert_eq!(encode_length(256), vec![0x82, 0x01, 0x00]);
        assert_eq!(encode_length(0x1234), vec![0x82, 0x12, 0x34]);
        assert_eq!(encode_length(65535), vec![0x82, 0xFF, 0xFF]);
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_parse_length() {
        // Short form
        assert_eq!(parse_length(&[0x05]).unwrap(), (&[][..], 5));
        assert_eq!(parse_length(&[0x7F]).unwrap(), (&[][..], 127));

        // Long form 1 byte
        assert_eq!(parse_length(&[0x81, 0x80]).unwrap(), (&[][..], 128));
        assert_eq!(parse_length(&[0x81, 0xFF]).unwrap(), (&[][..], 255));

        // Long form 2 bytes
        assert_eq!(parse_length(&[0x82, 0x01, 0x00]).unwrap(), (&[][..], 256));
        assert_eq!(parse_length(&[0x82, 0x12, 0x34]).unwrap(), (&[][..], 0x1234));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_object_identifier() {
        // OID for LN referencing: 2.16.756.5.8.1.1
        let oid = &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01];
        let encoded = encode_object_identifier(oid);
        assert_eq!(encoded[0], 0x06); // OID tag
        assert_eq!(encoded[1], 0x07); // Length
        assert_eq!(&encoded[2..], oid);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_bit_string() {
        let bits = &[0x80]; // Protocol version = 0 (bit 7 set, rest unused)
        let encoded = encode_bit_string(bits, 7);
        assert_eq!(encoded, vec![0x03, 0x02, 0x07, 0x80]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_octet_string() {
        let data = b"test";
        let encoded = encode_octet_string(data);
        assert_eq!(encoded[0], 0x04); // OCTET STRING tag
        assert_eq!(encoded[1], 0x04); // Length
        assert_eq!(&encoded[2..], data);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_context_specific() {
        let content = b"data";
        // Context[0] Constructed
        let encoded = encode_context_specific(0, TagType::Constructed, content);
        assert_eq!(encoded[0], 0xA0);
        assert_eq!(encoded[1], 0x04);
        assert_eq!(&encoded[2..], content);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_sequence() {
        let content = &[0x02, 0x01, 0x05]; // INTEGER 5
        let encoded = encode_sequence(content);
        assert_eq!(encoded[0], 0x30); // SEQUENCE tag
        assert_eq!(encoded[1], 0x03); // Length
        assert_eq!(&encoded[2..], content);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_application() {
        let content = b"aarq";
        // APPLICATION[0] Constructed (AARQ)
        let encoded = encode_application(0, TagType::Constructed, content);
        assert_eq!(encoded[0], 0x60);
        assert_eq!(encoded[1], 0x04);
        assert_eq!(&encoded[2..], content);
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_parse_object_identifier() {
        let input = &[0x06, 0x07, 0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01];
        let (rest, oid) = parse_object_identifier(input).unwrap();
        assert_eq!(rest, &[]);
        assert_eq!(oid, vec![0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01]);
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_parse_bit_string() {
        let input = &[0x03, 0x02, 0x07, 0x80];
        let (rest, (bits, unused)) = parse_bit_string(input).unwrap();
        assert_eq!(rest, &[]);
        assert_eq!(bits, vec![0x80]);
        assert_eq!(unused, 7);
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_parse_octet_string() {
        let input = &[0x04, 0x04, b't', b'e', b's', b't'];
        let (rest, octets) = parse_octet_string(input).unwrap();
        assert_eq!(rest, &[]);
        assert_eq!(octets, b"test");
    }

    #[test]
    #[cfg(all(feature = "parse", feature = "encode"))]
    fn test_roundtrip_oid() {
        let oid = &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01];
        let encoded = encode_object_identifier(oid);
        let (_, parsed) = parse_object_identifier(&encoded).unwrap();
        assert_eq!(parsed, oid);
    }

    #[test]
    #[cfg(all(feature = "parse", feature = "encode"))]
    fn test_roundtrip_bit_string() {
        let bits = &[0x80];
        let encoded = encode_bit_string(bits, 7);
        let (_, (parsed_bits, parsed_unused)) = parse_bit_string(&encoded).unwrap();
        assert_eq!(parsed_bits, bits);
        assert_eq!(parsed_unused, 7);
    }

    #[test]
    #[cfg(all(feature = "parse", feature = "encode"))]
    fn test_roundtrip_octet_string() {
        let data = b"test data";
        let encoded = encode_octet_string(data);
        let (_, parsed) = parse_octet_string(&encoded).unwrap();
        assert_eq!(parsed, data);
    }
}
