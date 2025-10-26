//! xDLMS InitiateRequest and InitiateResponse APDUs
//!
//! These APDUs are carried in the user_information field of AARQ/AARE APDUs.
//! They are encoded using A-XDR (Adapted eXternal Data Representation).
//!
//! Reference: DLMS Green Book Ed. 12, Section 11.2 and Table 134-135

use alloc::vec::Vec;
use core::fmt;

use super::{Conformance, DLMS_VERSION, VAA_NAME_LN};

#[cfg(feature = "parse")]
use nom::{
    IResult,
    bytes::complete::take,
    number::complete::{be_u8, be_u16},
};

/// xDLMS InitiateRequest APDU
///
/// Sent by the client in the AARQ user_information field to propose
/// association parameters.
///
/// Reference: Green Book Table 134
#[derive(Debug, Clone, PartialEq)]
pub struct InitiateRequest {
    /// Optional dedicated key for ciphering (rarely used)
    pub dedicated_key: Option<Vec<u8>>,
    /// Whether server should respond (always true in practice)
    pub response_allowed: bool,
    /// Quality of service (optional, rarely used)
    pub proposed_quality_of_service: Option<u8>,
    /// DLMS version number (typically 6)
    pub proposed_dlms_version_number: u8,
    /// Client conformance bits (which services client supports)
    pub proposed_conformance: Conformance,
    /// Maximum PDU size client can receive
    pub client_max_receive_pdu_size: u16,
}

impl InitiateRequest {
    /// Create a new InitiateRequest with typical defaults
    ///
    /// # Arguments
    ///
    /// * `conformance` - Client conformance bits
    /// * `max_pdu_size` - Maximum PDU size client can receive (typically 0xFFFF)
    pub fn new(conformance: Conformance, max_pdu_size: u16) -> Self {
        Self {
            dedicated_key: None,
            response_allowed: true,
            proposed_quality_of_service: None,
            proposed_dlms_version_number: DLMS_VERSION,
            proposed_conformance: conformance,
            client_max_receive_pdu_size: max_pdu_size,
        }
    }

    /// Create a new InitiateRequest with typical LN client conformance
    pub fn new_ln(max_pdu_size: u16) -> Self {
        Self::new(Conformance::TYPICAL_CLIENT_LN, max_pdu_size)
    }

    /// Create a new InitiateRequest with typical SN client conformance
    pub fn new_sn(max_pdu_size: u16) -> Self {
        Self::new(Conformance::TYPICAL_CLIENT_SN, max_pdu_size)
    }

    /// Encode to A-XDR format
    ///
    /// Returns the encoded bytes for inclusion in AARQ user_information
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // dedicated_key: optional OCTET STRING
        if let Some(ref key) = self.dedicated_key {
            buf.push(0x01); // present
            buf.push(key.len() as u8);
            buf.extend_from_slice(key);
        } else {
            buf.push(0x00); // not present
        }

        // response_allowed: BOOLEAN
        buf.push(if self.response_allowed { 0x01 } else { 0x00 });

        // proposed_quality_of_service: optional INTEGER
        if let Some(qos) = self.proposed_quality_of_service {
            buf.push(0x01); // present
            buf.push(qos);
        } else {
            buf.push(0x00); // not present
        }

        // proposed_dlms_version_number: Unsigned8
        buf.push(self.proposed_dlms_version_number);

        // proposed_conformance: BIT STRING (24 bits = 3 bytes)
        let conf_bytes = self.proposed_conformance.to_bytes();
        buf.extend_from_slice(&conf_bytes);

        // client_max_receive_pdu_size: Unsigned16
        buf.extend_from_slice(&self.client_max_receive_pdu_size.to_be_bytes());

        buf
    }

    /// Calculate the encoded length
    #[cfg(feature = "encode")]
    pub fn encoded_len(&self) -> usize {
        let mut len = 0;

        // dedicated_key
        len += 1; // presence flag
        if let Some(ref key) = self.dedicated_key {
            len += 1 + key.len(); // length + data
        }

        // response_allowed
        len += 1;

        // proposed_quality_of_service
        len += 1; // presence flag
        if self.proposed_quality_of_service.is_some() {
            len += 1; // value
        }

        // proposed_dlms_version_number
        len += 1;

        // proposed_conformance (3 bytes)
        len += 3;

        // client_max_receive_pdu_size
        len += 2;

        len
    }

    /// Parse from A-XDR format
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, dedicated_key_present) = be_u8(input)?;
        let (input, dedicated_key) = if dedicated_key_present != 0 {
            let (input, len) = be_u8(input)?;
            let (input, key) = take(len as usize)(input)?;
            (input, Some(key.to_vec()))
        } else {
            (input, None)
        };

        let (input, response_allowed_byte) = be_u8(input)?;
        let response_allowed = response_allowed_byte != 0;

        let (input, qos_present) = be_u8(input)?;
        let (input, proposed_quality_of_service) = if qos_present != 0 {
            let (input, qos) = be_u8(input)?;
            (input, Some(qos))
        } else {
            (input, None)
        };

        let (input, proposed_dlms_version_number) = be_u8(input)?;

        let (input, conf_bytes) = take(3usize)(input)?;
        let proposed_conformance =
            Conformance::from_bytes([conf_bytes[0], conf_bytes[1], conf_bytes[2]]);

        let (input, client_max_receive_pdu_size) = be_u16(input)?;

        Ok((
            input,
            Self {
                dedicated_key,
                response_allowed,
                proposed_quality_of_service,
                proposed_dlms_version_number,
                proposed_conformance,
                client_max_receive_pdu_size,
            },
        ))
    }
}

impl fmt::Display for InitiateRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InitiateRequest(v{}, conf=0x{:06X}, max_pdu={})",
            self.proposed_dlms_version_number,
            self.proposed_conformance.bits(),
            self.client_max_receive_pdu_size
        )
    }
}

/// xDLMS InitiateResponse APDU
///
/// Sent by the server in the AARE user_information field to indicate
/// negotiated association parameters.
///
/// Reference: Green Book Table 135
#[derive(Debug, Clone, PartialEq)]
pub struct InitiateResponse {
    /// Negotiated quality of service (optional, rarely used)
    pub negotiated_quality_of_service: Option<u8>,
    /// DLMS version number (should match request)
    pub negotiated_dlms_version_number: u8,
    /// Server conformance (bitwise AND of client and server conformance)
    pub negotiated_conformance: Conformance,
    /// Maximum PDU size server can receive
    pub server_max_receive_pdu_size: u16,
    /// VAA name (0x0007 for LN, 0x0001 for SN)
    pub vaa_name: u16,
}

impl InitiateResponse {
    /// Create a new InitiateResponse
    ///
    /// # Arguments
    ///
    /// * `conformance` - Negotiated conformance (AND of client/server)
    /// * `max_pdu_size` - Maximum PDU size server can receive
    /// * `vaa_name` - VAA name (VAA_NAME_LN or VAA_NAME_SN)
    pub fn new(conformance: Conformance, max_pdu_size: u16, vaa_name: u16) -> Self {
        Self {
            negotiated_quality_of_service: None,
            negotiated_dlms_version_number: DLMS_VERSION,
            negotiated_conformance: conformance,
            server_max_receive_pdu_size: max_pdu_size,
            vaa_name,
        }
    }

    /// Create a new InitiateResponse for Logical Name referencing
    pub fn new_ln(conformance: Conformance, max_pdu_size: u16) -> Self {
        Self::new(conformance, max_pdu_size, VAA_NAME_LN)
    }

    /// Encode to A-XDR format
    ///
    /// Returns the encoded bytes for inclusion in AARE user_information
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // negotiated_quality_of_service: optional INTEGER
        if let Some(qos) = self.negotiated_quality_of_service {
            buf.push(0x01); // present
            buf.push(qos);
        } else {
            buf.push(0x00); // not present
        }

        // negotiated_dlms_version_number: Unsigned8
        buf.push(self.negotiated_dlms_version_number);

        // negotiated_conformance: BIT STRING (24 bits = 3 bytes)
        let conf_bytes = self.negotiated_conformance.to_bytes();
        buf.extend_from_slice(&conf_bytes);

        // server_max_receive_pdu_size: Unsigned16
        buf.extend_from_slice(&self.server_max_receive_pdu_size.to_be_bytes());

        // vaa_name: Unsigned16
        buf.extend_from_slice(&self.vaa_name.to_be_bytes());

        buf
    }

    /// Calculate the encoded length
    #[cfg(feature = "encode")]
    pub fn encoded_len(&self) -> usize {
        let mut len = 0;

        // negotiated_quality_of_service
        len += 1; // presence flag
        if self.negotiated_quality_of_service.is_some() {
            len += 1; // value
        }

        // negotiated_dlms_version_number
        len += 1;

        // negotiated_conformance (3 bytes)
        len += 3;

        // server_max_receive_pdu_size
        len += 2;

        // vaa_name
        len += 2;

        len
    }

    /// Parse from A-XDR format
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, qos_present) = be_u8(input)?;
        let (input, negotiated_quality_of_service) = if qos_present != 0 {
            let (input, qos) = be_u8(input)?;
            (input, Some(qos))
        } else {
            (input, None)
        };

        let (input, negotiated_dlms_version_number) = be_u8(input)?;

        let (input, conf_bytes) = take(3usize)(input)?;
        let negotiated_conformance =
            Conformance::from_bytes([conf_bytes[0], conf_bytes[1], conf_bytes[2]]);

        let (input, server_max_receive_pdu_size) = be_u16(input)?;
        let (input, vaa_name) = be_u16(input)?;

        Ok((
            input,
            Self {
                negotiated_quality_of_service,
                negotiated_dlms_version_number,
                negotiated_conformance,
                server_max_receive_pdu_size,
                vaa_name,
            },
        ))
    }
}

impl fmt::Display for InitiateResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InitiateResponse(v{}, conf=0x{:06X}, max_pdu={}, vaa=0x{:04X})",
            self.negotiated_dlms_version_number,
            self.negotiated_conformance.bits(),
            self.server_max_receive_pdu_size,
            self.vaa_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initiate_request_new() {
        let req = InitiateRequest::new_ln(0xFFFF);
        assert_eq!(req.proposed_dlms_version_number, DLMS_VERSION);
        assert_eq!(req.client_max_receive_pdu_size, 0xFFFF);
        assert!(req.response_allowed);
        assert!(req.dedicated_key.is_none());
        assert!(req.proposed_quality_of_service.is_none());
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_initiate_request_encode_simple() {
        let req = InitiateRequest {
            dedicated_key: None,
            response_allowed: true,
            proposed_quality_of_service: None,
            proposed_dlms_version_number: 6,
            proposed_conformance: Conformance::from_bits(0x001F8000),
            client_max_receive_pdu_size: 0xFFFF,
        };

        let encoded = req.encode();

        // Verify structure:
        // [0] = 0x00 (no dedicated key)
        // [1] = 0x01 (response allowed)
        // [2] = 0x00 (no QoS)
        // [3] = 0x06 (DLMS version 6)
        // [4..6] = conformance bytes
        // [7..8] = max PDU size

        assert_eq!(encoded[0], 0x00); // no dedicated key
        assert_eq!(encoded[1], 0x01); // response allowed
        assert_eq!(encoded[2], 0x00); // no QoS
        assert_eq!(encoded[3], 0x06); // DLMS version
        assert_eq!(&encoded[4..7], &[0x1F, 0x80, 0x00]); // conformance
        assert_eq!(&encoded[7..9], &[0xFF, 0xFF]); // max PDU
    }

    #[test]
    #[cfg(feature = "parse")]
    fn test_initiate_request_roundtrip() {
        #[cfg(feature = "encode")]
        {
            let req = InitiateRequest::new_ln(0x0400);
            let encoded = req.encode();
            let (remaining, parsed) = InitiateRequest::parse(&encoded).unwrap();

            assert!(remaining.is_empty());
            assert_eq!(parsed, req);
        }
    }

    #[test]
    fn test_initiate_response_new() {
        let resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        assert_eq!(resp.negotiated_dlms_version_number, DLMS_VERSION);
        assert_eq!(resp.server_max_receive_pdu_size, 0x0400);
        assert_eq!(resp.vaa_name, VAA_NAME_LN);
        assert!(resp.negotiated_quality_of_service.is_none());
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_initiate_response_encode_simple() {
        let resp = InitiateResponse {
            negotiated_quality_of_service: None,
            negotiated_dlms_version_number: 6,
            negotiated_conformance: Conformance::from_bits(0x00008000),
            server_max_receive_pdu_size: 0x0400,
            vaa_name: VAA_NAME_LN,
        };

        let encoded = resp.encode();

        // Verify structure:
        // [0] = 0x00 (no QoS)
        // [1] = 0x06 (DLMS version 6)
        // [2..4] = conformance bytes
        // [5..6] = max PDU size
        // [7..8] = VAA name

        assert_eq!(encoded[0], 0x00); // no QoS
        assert_eq!(encoded[1], 0x06); // DLMS version
        assert_eq!(&encoded[2..5], &[0x00, 0x80, 0x00]); // conformance
        assert_eq!(&encoded[5..7], &[0x04, 0x00]); // max PDU
        assert_eq!(&encoded[7..9], &[0x00, 0x07]); // VAA name LN
    }

    #[test]
    #[cfg(feature = "parse")]
    fn test_initiate_response_roundtrip() {
        #[cfg(feature = "encode")]
        {
            let resp = InitiateResponse::new_ln(Conformance::GET | Conformance::SET, 0x0400);
            let encoded = resp.encode();
            let (remaining, parsed) = InitiateResponse::parse(&encoded).unwrap();

            assert!(remaining.is_empty());
            assert_eq!(parsed, resp);
        }
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_initiate_request_encoded_len() {
        let req = InitiateRequest::new_ln(0xFFFF);
        let encoded = req.encode();
        assert_eq!(encoded.len(), req.encoded_len());
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_initiate_response_encoded_len() {
        let resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let encoded = resp.encode();
        assert_eq!(encoded.len(), resp.encoded_len());
    }
}
