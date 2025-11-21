//! RLRE APDU (A-Release Response)
//!
//! This module implements encoding and parsing of RLRE APDUs used to
//! acknowledge the graceful release (disconnection) of an application association.
//!
//! Reference: DLMS Green Book Ed. 12, Section 11 and Table 147

use alloc::vec::Vec;
use core::fmt;

use super::ReleaseResponseReason;

#[cfg(feature = "encode")]
use alloc::vec;

#[cfg(feature = "encode")]
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

/// RLRE APDU (A-Release Response) - Tag 0x63
///
/// Sent by the server to acknowledge a release request.
/// Uses ASN.1 BER encoding with APPLICATION class tag 3.
///
/// Reference: Green Book Table 147
#[derive(Debug, Clone, PartialEq)]
pub struct RlreApdu {
    /// Reason for release response - optional
    pub reason: Option<ReleaseResponseReason>,
    /// User information (can carry xDLMS APDUs when ciphering is used) - optional
    pub user_information: Option<Vec<u8>>,
}

impl RlreApdu {
    /// Create a new RLRE APDU with default values (normal release, no user info)
    pub fn new() -> Self {
        Self { reason: Some(ReleaseResponseReason::Normal), user_information: None }
    }

    /// Create RLRE with a specific reason
    pub fn with_reason(reason: ReleaseResponseReason) -> Self {
        Self { reason: Some(reason), user_information: None }
    }

    /// Create RLRE with user information (for ciphered communication)
    pub fn with_user_info(user_info: Vec<u8>) -> Self {
        Self { reason: Some(ReleaseResponseReason::Normal), user_information: Some(user_info) }
    }

    /// Encode RLRE APDU to bytes
    ///
    /// Returns BER-encoded APDU with APPLICATION tag 3 (0x63)
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

        // Wrap in APPLICATION tag 3 (0x63)
        encode_application(3, TagType::Constructed, &content)
    }

    /// Parse RLRE APDU from bytes
    ///
    /// Expects BER-encoded APDU starting with APPLICATION tag 3 (0x63)
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        // Parse APPLICATION tag 3 (0x63)
        let (input, (tag_class, _tag_type, tag_number)) = parse_tag(input)?;
        if tag_class != TagClass::Application || tag_number != 3 {
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
                    reason = ReleaseResponseReason::from_u8(reason_byte);
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

impl Default for RlreApdu {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RlreApdu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RLRE(")?;
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
    fn test_rlre_new() {
        let rlre = RlreApdu::new();
        assert_eq!(rlre.reason, Some(ReleaseResponseReason::Normal));
        assert_eq!(rlre.user_information, None);
    }

    #[test]
    fn test_rlre_with_reason() {
        let rlre = RlreApdu::with_reason(ReleaseResponseReason::NotFinished);
        assert_eq!(rlre.reason, Some(ReleaseResponseReason::NotFinished));
        assert_eq!(rlre.user_information, None);
    }

    #[test]
    fn test_rlre_with_user_info() {
        let user_info = vec![0x01, 0x02, 0x03];
        let rlre = RlreApdu::with_user_info(user_info.clone());
        assert_eq!(rlre.reason, Some(ReleaseResponseReason::Normal));
        assert_eq!(rlre.user_information, Some(user_info));
    }

    #[test]
    fn test_rlre_display() {
        let rlre = RlreApdu::new();
        let display = format!("{}", rlre);
        assert!(display.contains("RLRE"));
        assert!(display.contains("Normal"));
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_rlre_encode_minimal() {
        // Minimal RLRE with only normal reason
        let rlre = RlreApdu::new();
        let bytes = rlre.encode();

        // Expected structure:
        // [0x63] - APPLICATION tag 3 (constructed)
        // [length]
        // [0x80][0x01][0x00] - context tag 0, length 1, reason=0 (Normal)

        assert_eq!(bytes[0], 0x63); // APPLICATION tag 3
        assert!(bytes.len() >= 4); // At least tag + length + reason tag/len/value

        // Verify reason is encoded
        assert!(bytes.contains(&0x80)); // Context tag 0
        assert!(bytes.contains(&0x00)); // Normal reason value
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_rlre_encode_with_not_finished() {
        let rlre = RlreApdu::with_reason(ReleaseResponseReason::NotFinished);
        let bytes = rlre.encode();

        assert_eq!(bytes[0], 0x63);
        assert!(bytes.contains(&0x80)); // Context tag 0
        assert!(bytes.contains(&0x01)); // NotFinished reason value
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_rlre_encode_with_user_info() {
        let user_info = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let rlre = RlreApdu::with_user_info(user_info.clone());
        let bytes = rlre.encode();

        assert_eq!(bytes[0], 0x63);
        assert!(bytes.contains(&0xBE)); // Context tag 30 (0x1E with constructed bit)
        // Should contain the user info bytes wrapped in OCTET STRING
        assert!(bytes.windows(4).any(|w| w == [0xDE, 0xAD, 0xBE, 0xEF]));
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_rlre_encode_empty() {
        // RLRE with no reason and no user info
        let rlre = RlreApdu { reason: None, user_information: None };
        let bytes = rlre.encode();

        // Should still have APPLICATION tag 3, but with zero or minimal content
        assert_eq!(bytes[0], 0x63);
        // Length should be 0 or minimal
        assert!(bytes.len() <= 3); // Tag + length byte(s) only
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_rlre_roundtrip_minimal() {
        let original = RlreApdu::new();
        let bytes = original.encode();
        let (remaining, parsed) = RlreApdu::parse(&bytes).expect("Failed to parse");

        assert!(remaining.is_empty());
        assert_eq!(parsed.reason, original.reason);
        assert_eq!(parsed.user_information, original.user_information);
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_rlre_roundtrip_with_reason() {
        let original = RlreApdu::with_reason(ReleaseResponseReason::NotFinished);
        let bytes = original.encode();
        let (remaining, parsed) = RlreApdu::parse(&bytes).expect("Failed to parse");

        assert!(remaining.is_empty());
        assert_eq!(parsed.reason, Some(ReleaseResponseReason::NotFinished));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_rlre_roundtrip_with_user_info() {
        let user_info = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let original = RlreApdu::with_user_info(user_info.clone());
        let bytes = original.encode();
        let (remaining, parsed) = RlreApdu::parse(&bytes).expect("Failed to parse");

        assert!(remaining.is_empty());
        assert_eq!(parsed.user_information, Some(user_info));
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_rlre_roundtrip_empty() {
        let original = RlreApdu { reason: None, user_information: None };
        let bytes = original.encode();
        let (remaining, parsed) = RlreApdu::parse(&bytes).expect("Failed to parse");

        assert!(remaining.is_empty());
        assert_eq!(parsed.reason, None);
        assert_eq!(parsed.user_information, None);
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_rlre_parse_invalid_tag() {
        // Wrong APPLICATION tag (using RLRQ tag 0x62 instead of RLRE 0x63)
        let bytes = vec![0x62, 0x00];
        let result = RlreApdu::parse(&bytes);
        assert!(result.is_err());
    }

    #[cfg(feature = "parse")]
    #[test]
    fn test_rlre_parse_truncated() {
        // Incomplete RLRE
        let bytes = vec![0x63];
        let result = RlreApdu::parse(&bytes);
        assert!(result.is_err());
    }
}
