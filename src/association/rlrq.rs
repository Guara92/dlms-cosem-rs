//! RLRQ APDU (A-Release Request)
//!
//! This module implements encoding and parsing of RLRQ APDUs used to
//! gracefully release (disconnect) an application association with a DLMS/COSEM server.
//!
//! Reference: DLMS Green Book Ed. 12, Section 11 and Table 146

use alloc::vec::Vec;
use core::fmt;

use super::ReleaseRequestReason;

#[cfg(any(feature = "encode", feature = "parse"))]
use super::ber::TagType;

#[cfg(feature = "encode")]
use super::ber::{encode_application, encode_context_specific, encode_octet_string};

#[cfg(feature = "parse")]
use nom::{
    IResult,
    error::{Error, ErrorKind},
};

#[cfg(feature = "parse")]
use super::ber::{TagClass, parse_length, parse_octet_string, parse_tag};

/// RLRQ APDU (A-Release Request) - Tag 0x62
///
/// Sent by the client to gracefully release an application association.
/// Uses ASN.1 BER encoding with APPLICATION class tag 2.
///
/// Reference: Green Book Table 146
#[derive(Debug, Clone, PartialEq)]
pub struct RlrqApdu {
    /// Reason for release request - optional
    pub reason: Option<ReleaseRequestReason>,
    /// User information (can carry xDLMS APDUs when ciphering is used) - optional
    pub user_information: Option<Vec<u8>>,
}

impl RlrqApdu {
    /// Create a new RLRQ APDU with default values (normal release, no user info)
    pub fn new() -> Self {
        Self { reason: Some(ReleaseRequestReason::Normal), user_information: None }
    }

    /// Create RLRQ with a specific reason
    pub fn with_reason(reason: ReleaseRequestReason) -> Self {
        Self { reason: Some(reason), user_information: None }
    }

    /// Create RLRQ with user information (for ciphered communication)
    pub fn with_user_info(user_info: Vec<u8>) -> Self {
        Self { reason: Some(ReleaseRequestReason::Normal), user_information: Some(user_info) }
    }

    /// Encode RLRQ APDU to bytes
    ///
    /// Returns BER-encoded APDU with APPLICATION tag 2 (0x62)
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut content = Vec::new();

        // [0x80][1][reason_value] - context-specific, primitive, tag 0
        if let Some(reason) = self.reason {
            let reason_bytes = vec![reason.as_u8()];
            content.extend_from_slice(&encode_context_specific(
                0,
                TagType::Primitive,
                &reason_bytes,
            ));
        }

        // [0xBE][length][user_info] - context-specific, constructed, tag 30 (0x1E)
        if let Some(ref user_info) = self.user_information {
            // User information is encoded as OCTET STRING inside context tag 30
            let octet_string = encode_octet_string(user_info);
            content.extend_from_slice(&encode_context_specific(
                30,
                TagType::Constructed,
                &octet_string,
            ));
        }

        // Wrap in APPLICATION tag 2 (0x62)
        encode_application(2, TagType::Constructed, &content)
    }

    /// Parse RLRQ APDU from bytes
    ///
    /// Expects BER-encoded APDU starting with APPLICATION tag 2 (0x62)
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        // Parse APPLICATION tag 2 (0x62)
        let (input, (tag_class, _tag_type, tag_number)) = parse_tag(input)?;
        if tag_class != TagClass::Application || tag_number != 2 {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }

        let (input, length) = parse_length(input)?;
        let (input, content) = nom::bytes::streaming::take(length)(input)?;

        let mut reason = None;
        let mut user_information = None;
        let mut remaining = content;

        while !remaining.is_empty() {
            let (rest, (tag_class, _tag_type, tag_number)) = parse_tag(remaining)?;

            match (tag_class, tag_number) {
                // Context-specific tag 0 - reason
                (TagClass::ContextSpecific, 0) => {
                    let (rest, len) = parse_length(rest)?;
                    if len != 1 {
                        return Err(nom::Err::Error(Error::new(rest, ErrorKind::LengthValue)));
                    }
                    let (rest, reason_byte) = nom::number::streaming::u8(rest)?;
                    reason = ReleaseRequestReason::from_u8(reason_byte);
                    remaining = rest;
                }
                // Context-specific tag 30 - user_information
                (TagClass::ContextSpecific, 30) => {
                    let (rest, len) = parse_length(rest)?;
                    let (rest, ui_content) = nom::bytes::streaming::take(len)(rest)?;
                    // Parse the OCTET STRING inside
                    let (_, ui_bytes) = parse_octet_string(ui_content)?;
                    user_information = Some(ui_bytes.to_vec());
                    remaining = rest;
                }
                _ => {
                    // Unknown tag - skip it
                    let (rest, len) = parse_length(rest)?;
                    let (rest, _) = nom::bytes::streaming::take(len)(rest)?;
                    remaining = rest;
                }
            }
        }

        Ok((input, Self { reason, user_information }))
    }
}

impl Default for RlrqApdu {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RlrqApdu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RLRQ(")?;
        if let Some(reason) = self.reason {
            write!(f, "reason={}", reason)?;
        }
        if self.user_information.is_some() {
            write!(f, ", with user_info")?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rlrq_new() {
        let rlrq = RlrqApdu::new();
        assert_eq!(rlrq.reason, Some(ReleaseRequestReason::Normal));
        assert_eq!(rlrq.user_information, None);
    }

    #[test]
    fn test_rlrq_with_reason() {
        let rlrq = RlrqApdu::with_reason(ReleaseRequestReason::NotFinished);
        assert_eq!(rlrq.reason, Some(ReleaseRequestReason::NotFinished));
        assert_eq!(rlrq.user_information, None);
    }

    #[test]
    fn test_rlrq_with_user_info() {
        let user_info = vec![0x01, 0x02, 0x03];
        let rlrq = RlrqApdu::with_user_info(user_info.clone());
        assert_eq!(rlrq.reason, Some(ReleaseRequestReason::Normal));
        assert_eq!(rlrq.user_information, Some(user_info));
    }

    #[test]
    fn test_rlrq_display() {
        let rlrq = RlrqApdu::new();
        let display = format!("{}", rlrq);
        assert!(display.contains("RLRQ"));
        assert!(display.contains("Normal"));
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_rlrq_encode_minimal() {
        // Minimal RLRQ with only normal reason
        let rlrq = RlrqApdu::new();
        let bytes = rlrq.encode();

        // Expected structure:
        // [0x62] - APPLICATION tag 2 (constructed)
        // [length]
        // [0x80][0x01][0x00] - context tag 0, length 1, reason=0 (Normal)

        assert_eq!(bytes[0], 0x62); // APPLICATION tag 2
        assert!(bytes.len() >= 4); // At least tag + length + reason tag/len/value

        // Verify reason is encoded
        assert!(bytes.contains(&0x80)); // Context tag 0
        assert!(bytes.contains(&0x00)); // Normal reason value
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_rlrq_encode_with_not_finished() {
        let rlrq = RlrqApdu::with_reason(ReleaseRequestReason::NotFinished);
        let bytes = rlrq.encode();

        assert_eq!(bytes[0], 0x62);
        assert!(bytes.contains(&0x80)); // Context tag 0
        assert!(bytes.contains(&0x01)); // NotFinished reason value
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_rlrq_encode_with_user_info() {
        let user_info = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let rlrq = RlrqApdu::with_user_info(user_info.clone());
        let bytes = rlrq.encode();

        assert_eq!(bytes[0], 0x62);
        assert!(bytes.contains(&0xBE)); // Context tag 30 (0x1E with constructed bit)
        // Should contain the user info bytes wrapped in OCTET STRING
        assert!(bytes.windows(4).any(|w| w == [0xDE, 0xAD, 0xBE, 0xEF]));
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_rlrq_encode_empty() {
        // RLRQ with no reason and no user info
        let rlrq = RlrqApdu { reason: None, user_information: None };
        let bytes = rlrq.encode();

        // Should still have APPLICATION tag 2, but with zero or minimal content
        assert_eq!(bytes[0], 0x62);
        // Length should be 0 or minimal
        assert!(bytes.len() <= 3); // Tag + length byte(s) only
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_rlrq_roundtrip_minimal() {
        let original = RlrqApdu::new();
        let bytes = original.encode();
        let (remaining, parsed) = RlrqApdu::parse(&bytes).expect("Failed to parse");

        assert!(remaining.is_empty());
        assert_eq!(parsed.reason, original.reason);
        assert_eq!(parsed.user_information, original.user_information);
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_rlrq_roundtrip_with_reason() {
        let original = RlrqApdu::with_reason(ReleaseRequestReason::NotFinished);
        let bytes = original.encode();
        let (remaining, parsed) = RlrqApdu::parse(&bytes).expect("Failed to parse");

        assert!(remaining.is_empty());
        assert_eq!(parsed.reason, Some(ReleaseRequestReason::NotFinished));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_rlrq_roundtrip_with_user_info() {
        let user_info = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let original = RlrqApdu::with_user_info(user_info.clone());
        let bytes = original.encode();
        let (remaining, parsed) = RlrqApdu::parse(&bytes).expect("Failed to parse");

        assert!(remaining.is_empty());
        assert_eq!(parsed.user_information, Some(user_info));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_rlrq_roundtrip_empty() {
        let original = RlrqApdu { reason: None, user_information: None };
        let bytes = original.encode();
        let (remaining, parsed) = RlrqApdu::parse(&bytes).expect("Failed to parse");

        assert!(remaining.is_empty());
        assert_eq!(parsed.reason, None);
        assert_eq!(parsed.user_information, None);
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_rlrq_parse_invalid_tag() {
        // Wrong APPLICATION tag (using AARE tag 0x61 instead of RLRQ 0x62)
        let bytes = vec![0x61, 0x00];
        let result = RlrqApdu::parse(&bytes);
        assert!(result.is_err());
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_rlrq_parse_truncated() {
        // Incomplete RLRQ
        let bytes = vec![0x62];
        let result = RlrqApdu::parse(&bytes);
        assert!(result.is_err());
    }
}
