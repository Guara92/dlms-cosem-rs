//! AARQ APDU (A-Associate Request)
//!
//! This module implements encoding and parsing of AARQ APDUs used to initiate
//! an application association with a DLMS/COSEM server.
//!
//! Reference: DLMS Green Book Ed. 12, Section 11.3 and Table 136-137

use alloc::vec::Vec;
use core::fmt;

use super::{
    ApplicationContextName, AuthenticationValue, InitiateRequest, MechanismName, PROTOCOL_VERSION,
};

#[cfg(any(feature = "encode", feature = "parse"))]
use super::ber::TagType;

#[cfg(feature = "encode")]
use super::ber::{
    encode_application, encode_bit_string, encode_context_specific, encode_object_identifier,
    encode_octet_string,
};

#[cfg(feature = "parse")]
use nom::{
    IResult,
    error::{Error, ErrorKind},
    number::streaming::u8 as nom_u8,
};

#[cfg(feature = "parse")]
use super::ber::{
    TagClass, parse_bit_string, parse_length, parse_object_identifier, parse_octet_string,
    parse_tag,
};

/// AARQ APDU (A-Associate Request) - Tag 0x60
///
/// Sent by the client to initiate an application association.
/// Uses ASN.1 BER encoding with context-specific tags.
///
/// Reference: Green Book Table 136-137
#[derive(Debug, Clone, PartialEq)]
pub struct AarqApdu {
    /// Protocol version (default: 0 for version 1)
    pub protocol_version: u8,
    /// Application context name (LN/SN with/without ciphering)
    pub application_context_name: ApplicationContextName,
    /// Called AP Title (server) - optional
    pub called_ap_title: Option<Vec<u8>>,
    /// Called AE Qualifier (server) - optional
    pub called_ae_qualifier: Option<Vec<u8>>,
    /// Called AP invocation ID (server) - optional
    pub called_ap_invocation_id: Option<u32>,
    /// Called AE invocation ID (server) - optional
    pub called_ae_invocation_id: Option<u32>,
    /// Calling AP Title (client) - optional, typically system title
    pub calling_ap_title: Option<Vec<u8>>,
    /// Calling AE Qualifier (client) - optional
    pub calling_ae_qualifier: Option<Vec<u8>>,
    /// Calling AP invocation ID (client) - optional
    pub calling_ap_invocation_id: Option<u32>,
    /// Calling AE invocation ID (client) - optional
    pub calling_ae_invocation_id: Option<u32>,
    /// Sender ACSE requirements - optional
    pub sender_acse_requirements: Option<u8>,
    /// Authentication mechanism (OID)
    pub mechanism_name: Option<MechanismName>,
    /// Authentication value (password or challenge)
    pub calling_authentication_value: Option<AuthenticationValue>,
    /// xDLMS InitiateRequest APDU
    pub user_information: Option<InitiateRequest>,
}

impl AarqApdu {
    /// Create a new AARQ APDU with typical defaults for Logical Name referencing
    ///
    /// # Arguments
    ///
    /// * `conformance_bits` - Client conformance bits (24-bit value)
    /// * `max_pdu_size` - Maximum PDU size client can receive
    pub fn new_simple_ln(max_pdu_size: u16) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            called_ap_title: None,
            called_ae_qualifier: None,
            called_ap_invocation_id: None,
            called_ae_invocation_id: None,
            calling_ap_title: None,
            calling_ae_qualifier: None,
            calling_ap_invocation_id: None,
            calling_ae_invocation_id: None,
            sender_acse_requirements: None,
            mechanism_name: Some(MechanismName::LowestLevelSecurity),
            calling_authentication_value: None,
            user_information: Some(InitiateRequest::new_ln(max_pdu_size)),
        }
    }

    /// Create a new AARQ APDU with password authentication
    pub fn new_with_password(max_pdu_size: u16, password: Vec<u8>) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            application_context_name: ApplicationContextName::LogicalNameReferencing,
            called_ap_title: None,
            called_ae_qualifier: None,
            called_ap_invocation_id: None,
            called_ae_invocation_id: None,
            calling_ap_title: None,
            calling_ae_qualifier: None,
            calling_ap_invocation_id: None,
            calling_ae_invocation_id: None,
            sender_acse_requirements: None,
            mechanism_name: Some(MechanismName::LowLevelSecurity),
            calling_authentication_value: Some(AuthenticationValue::CharString(password)),
            user_information: Some(InitiateRequest::new_ln(max_pdu_size)),
        }
    }

    /// Create a new AARQ APDU with ciphering
    pub fn new_with_ciphering(max_pdu_size: u16, system_title: [u8; 8]) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            application_context_name: ApplicationContextName::LogicalNameReferencingWithCiphering,
            called_ap_title: None,
            called_ae_qualifier: None,
            called_ap_invocation_id: None,
            called_ae_invocation_id: None,
            calling_ap_title: Some(system_title.to_vec()),
            calling_ae_qualifier: None,
            calling_ap_invocation_id: None,
            calling_ae_invocation_id: None,
            sender_acse_requirements: None,
            mechanism_name: Some(MechanismName::HighLevelSecurityGmac),
            calling_authentication_value: None,
            user_information: Some(InitiateRequest::new_ln(max_pdu_size)),
        }
    }

    /// Encode to ASN.1 BER format
    ///
    /// Returns the complete AARQ APDU including the tag and length
    ///
    /// # BER Structure
    ///
    /// ```text
    /// 60 (APPLICATION 0 CONSTRUCTED) - AARQ
    ///   A0 (CONTEXT 0) - protocol-version [OPTIONAL]
    ///   A1 (CONTEXT 1) - application-context-name
    ///   A2 (CONTEXT 2) - called-AP-title [OPTIONAL]
    ///   A3 (CONTEXT 3) - called-AE-qualifier [OPTIONAL]
    ///   A4 (CONTEXT 4) - called-AP-invocation-id [OPTIONAL]
    ///   A5 (CONTEXT 5) - called-AE-invocation-id [OPTIONAL]
    ///   A6 (CONTEXT 6) - calling-AP-title [OPTIONAL]
    ///   A7 (CONTEXT 7) - calling-AE-qualifier [OPTIONAL]
    ///   A8 (CONTEXT 8) - calling-AP-invocation-id [OPTIONAL]
    ///   A9 (CONTEXT 9) - calling-AE-invocation-id [OPTIONAL]
    ///   8A (CONTEXT 10 PRIMITIVE) - sender-acse-requirements [OPTIONAL]
    ///   8B (CONTEXT 11 PRIMITIVE) - mechanism-name [OPTIONAL]
    ///   AC (CONTEXT 12) - calling-authentication-value [OPTIONAL]
    ///   BE (CONTEXT 30) - user-information [OPTIONAL]
    /// ```
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut content = Vec::new();

        // A0: protocol-version (BIT STRING, default 0 = version 1)
        // Bit 7 set, 7 unused bits
        let version_bits = if self.protocol_version == 0 { &[0x80] } else { &[0x00] };
        let protocol_version_encoded = encode_bit_string(version_bits, 7);
        content.extend(encode_context_specific(0, TagType::Constructed, &protocol_version_encoded));

        // A1: application-context-name (OBJECT IDENTIFIER)
        let app_context_oid = encode_object_identifier(self.application_context_name.oid_bytes());
        content.extend(encode_context_specific(1, TagType::Constructed, &app_context_oid));

        // A2: called-AP-title (OPTIONAL)
        if let Some(ref title) = self.called_ap_title {
            let title_encoded = encode_octet_string(title);
            content.extend(encode_context_specific(2, TagType::Constructed, &title_encoded));
        }

        // A3: called-AE-qualifier (OPTIONAL)
        if let Some(ref qualifier) = self.called_ae_qualifier {
            let qualifier_encoded = encode_octet_string(qualifier);
            content.extend(encode_context_specific(3, TagType::Constructed, &qualifier_encoded));
        }

        // A4: called-AP-invocation-id (OPTIONAL) - not commonly used, skip for now

        // A5: called-AE-invocation-id (OPTIONAL) - not commonly used, skip for now

        // A6: calling-AP-title (OPTIONAL) - typically system title for ciphering
        if let Some(ref title) = self.calling_ap_title {
            let title_encoded = encode_octet_string(title);
            content.extend(encode_context_specific(6, TagType::Constructed, &title_encoded));
        }

        // A7: calling-AE-qualifier (OPTIONAL)
        if let Some(ref qualifier) = self.calling_ae_qualifier {
            let qualifier_encoded = encode_octet_string(qualifier);
            content.extend(encode_context_specific(7, TagType::Constructed, &qualifier_encoded));
        }

        // A8: calling-AP-invocation-id (OPTIONAL) - not commonly used, skip for now

        // A9: calling-AE-invocation-id (OPTIONAL) - not commonly used, skip for now

        // 8A: sender-acse-requirements (OPTIONAL)
        if let Some(acse_req) = self.sender_acse_requirements {
            let acse_bits = encode_bit_string(&[acse_req], 0);
            content.extend(encode_context_specific(10, TagType::Primitive, &acse_bits));
        }

        // 8B: mechanism-name (OPTIONAL)
        if let Some(ref mechanism) = self.mechanism_name {
            let mechanism_oid = encode_object_identifier(mechanism.oid_bytes());
            content.extend(encode_context_specific(11, TagType::Primitive, &mechanism_oid));
        }

        // AC: calling-authentication-value (OPTIONAL)
        if let Some(ref auth_value) = self.calling_authentication_value {
            let auth_encoded = match auth_value {
                AuthenticationValue::CharString(password) => {
                    // CONTEXT 0 IMPLICIT OCTET STRING
                    let mut auth_content = Vec::new();
                    auth_content.extend(encode_context_specific(0, TagType::Primitive, password));
                    auth_content
                }
                AuthenticationValue::BitString(bits) => {
                    // CONTEXT 1 IMPLICIT BIT STRING
                    let bit_string = encode_bit_string(bits, 0);
                    let mut auth_content = Vec::new();
                    auth_content.extend(encode_context_specific(
                        1,
                        TagType::Primitive,
                        &bit_string,
                    ));
                    auth_content
                }
            };
            content.extend(encode_context_specific(12, TagType::Constructed, &auth_encoded));
        }

        // BE: user-information (OPTIONAL) - contains xDLMS InitiateRequest
        if let Some(ref user_info) = self.user_information {
            let initiate_encoded = user_info.encode();
            // user-information is OCTET STRING containing the xDLMS APDU
            let user_info_octets = encode_octet_string(&initiate_encoded);
            content.extend(encode_context_specific(30, TagType::Constructed, &user_info_octets));
        }

        // Wrap in APPLICATION[0] CONSTRUCTED tag (0x60)
        encode_application(0, TagType::Constructed, &content)
    }

    /// Parse from ASN.1 BER format
    ///
    /// # BER Structure
    ///
    /// ```text
    /// 60 (APPLICATION 0 CONSTRUCTED) - AARQ
    ///   A0 (CONTEXT 0) - protocol-version [OPTIONAL]
    ///   A1 (CONTEXT 1) - application-context-name
    ///   A2 (CONTEXT 2) - called-AP-title [OPTIONAL]
    ///   A3 (CONTEXT 3) - called-AE-qualifier [OPTIONAL]
    ///   A6 (CONTEXT 6) - calling-AP-title [OPTIONAL]
    ///   A7 (CONTEXT 7) - calling-AE-qualifier [OPTIONAL]
    ///   8A (CONTEXT 10 PRIMITIVE) - sender-acse-requirements [OPTIONAL]
    ///   8B (CONTEXT 11 PRIMITIVE) - mechanism-name [OPTIONAL]
    ///   AC (CONTEXT 12) - calling-authentication-value [OPTIONAL]
    ///   BE (CONTEXT 30) - user-information [OPTIONAL]
    /// ```
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        // Parse APPLICATION[0] CONSTRUCTED tag (0x60)
        let (input, (class, tag_type, tag_number)) = parse_tag(input)?;
        if class != TagClass::Application || tag_number != 0 {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }
        if tag_type != TagType::Constructed {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)));
        }

        // Parse length
        let (input, length) = parse_length(input)?;
        if input.len() < length {
            return Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)));
        }

        // Extract AARQ content
        let aarq_content = &input[..length];
        let remaining = &input[length..];

        // Parse AARQ fields from content
        let mut content = aarq_content;

        let mut protocol_version = PROTOCOL_VERSION;
        let mut application_context_name = None;
        let mut called_ap_title = None;
        let mut called_ae_qualifier = None;
        let mut called_ap_invocation_id = None;
        let mut called_ae_invocation_id = None;
        let mut calling_ap_title = None;
        let mut calling_ae_qualifier = None;
        let mut calling_ap_invocation_id = None;
        let mut calling_ae_invocation_id = None;
        let mut sender_acse_requirements = None;
        let mut mechanism_name = None;
        let mut calling_authentication_value = None;
        let mut user_information = None;

        // Parse all context-specific fields
        while !content.is_empty() {
            let (new_content, (class, tag_type, tag_number)) = parse_tag(content)?;

            if class != TagClass::ContextSpecific {
                return Err(nom::Err::Error(Error::new(content, ErrorKind::Tag)));
            }

            let (new_content, field_length) = parse_length(new_content)?;
            if new_content.len() < field_length {
                return Err(nom::Err::Error(Error::new(new_content, ErrorKind::Eof)));
            }

            let field_content = &new_content[..field_length];
            content = &new_content[field_length..];

            match tag_number {
                // A0: protocol-version (BIT STRING)
                0 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, (bits, _unused)) = parse_bit_string(field_content)?;
                    // Bit 0 set means version 1 (default is 0)
                    protocol_version =
                        if !bits.is_empty() && (bits[0] & 0x80) != 0 { 0 } else { 1 };
                }

                // A1: application-context-name (OBJECT IDENTIFIER)
                1 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, oid_bytes) = parse_object_identifier(field_content)?;
                    application_context_name = ApplicationContextName::from_oid_bytes(&oid_bytes);
                }

                // A2: called-AP-title (OCTET STRING)
                2 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, octets) = parse_octet_string(field_content)?;
                    called_ap_title = Some(octets);
                }

                // A3: called-AE-qualifier (OCTET STRING)
                3 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, octets) = parse_octet_string(field_content)?;
                    called_ae_qualifier = Some(octets);
                }

                // A4: called-AP-invocation-id (INTEGER)
                4 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (field_content, int_tag) = nom_u8(field_content)?;
                    if int_tag == 0x02 {
                        let (field_content, int_len) = nom_u8(field_content)?;
                        if int_len <= 4 && field_content.len() >= int_len as usize {
                            let mut value: u32 = 0;
                            for &byte in field_content.iter().take(int_len as usize) {
                                value = (value << 8) | byte as u32;
                            }
                            called_ap_invocation_id = Some(value);
                        }
                    }
                }

                // A5: called-AE-invocation-id (INTEGER)
                5 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (field_content, int_tag) = nom_u8(field_content)?;
                    if int_tag == 0x02 {
                        let (field_content, int_len) = nom_u8(field_content)?;
                        if int_len <= 4 && field_content.len() >= int_len as usize {
                            let mut value: u32 = 0;
                            for &byte in field_content.iter().take(int_len as usize) {
                                value = (value << 8) | byte as u32;
                            }
                            called_ae_invocation_id = Some(value);
                        }
                    }
                }

                // A6: calling-AP-title (OCTET STRING) - typically system title
                6 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, octets) = parse_octet_string(field_content)?;
                    calling_ap_title = Some(octets);
                }

                // A7: calling-AE-qualifier (OCTET STRING)
                7 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, octets) = parse_octet_string(field_content)?;
                    calling_ae_qualifier = Some(octets);
                }

                // A8: calling-AP-invocation-id (INTEGER)
                8 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (field_content, int_tag) = nom_u8(field_content)?;
                    if int_tag == 0x02 {
                        let (field_content, int_len) = nom_u8(field_content)?;
                        if int_len <= 4 && field_content.len() >= int_len as usize {
                            let mut value: u32 = 0;
                            for &byte in field_content.iter().take(int_len as usize) {
                                value = (value << 8) | byte as u32;
                            }
                            calling_ap_invocation_id = Some(value);
                        }
                    }
                }

                // A9: calling-AE-invocation-id (INTEGER)
                9 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (field_content, int_tag) = nom_u8(field_content)?;
                    if int_tag == 0x02 {
                        let (field_content, int_len) = nom_u8(field_content)?;
                        if int_len <= 4 && field_content.len() >= int_len as usize {
                            let mut value: u32 = 0;
                            for &byte in field_content.iter().take(int_len as usize) {
                                value = (value << 8) | byte as u32;
                            }
                            calling_ae_invocation_id = Some(value);
                        }
                    }
                }

                // 8A (CONTEXT 10 PRIMITIVE): sender-acse-requirements (BIT STRING)
                10 => {
                    if tag_type != TagType::Primitive {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, (bits, _unused)) = parse_bit_string(field_content)?;
                    if !bits.is_empty() {
                        sender_acse_requirements = Some(bits[0]);
                    }
                }

                // 8B (CONTEXT 11 PRIMITIVE): mechanism-name (OBJECT IDENTIFIER)
                11 => {
                    if tag_type != TagType::Primitive {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, oid_bytes) = parse_object_identifier(field_content)?;
                    mechanism_name = MechanismName::from_oid_bytes(&oid_bytes);
                }

                // AC (CONTEXT 12): calling-authentication-value (CHOICE)
                12 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    // Parse CHOICE (CharString [0] or BitString [1])
                    let (field_content, (choice_class, choice_type, choice_tag)) =
                        parse_tag(field_content)?;
                    if choice_class != TagClass::ContextSpecific {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }

                    let (field_content, choice_len) = parse_length(field_content)?;
                    if field_content.len() < choice_len {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Eof)));
                    }

                    let choice_content = &field_content[..choice_len];

                    if choice_tag == 0 && choice_type == TagType::Primitive {
                        // CharString
                        calling_authentication_value =
                            Some(AuthenticationValue::CharString(choice_content.to_vec()));
                    } else if choice_tag == 1 && choice_type == TagType::Primitive {
                        // BitString
                        let (_remaining, (bits, _unused)) = parse_bit_string(choice_content)?;
                        calling_authentication_value = Some(AuthenticationValue::BitString(bits));
                    }
                }

                // BE (CONTEXT 30): user-information (OCTET STRING containing InitiateRequest)
                30 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, octets) = parse_octet_string(field_content)?;
                    // Parse InitiateRequest from the octet string
                    match InitiateRequest::parse(&octets) {
                        Ok((_, init_req)) => {
                            user_information = Some(init_req);
                        }
                        Err(_) => {
                            // If parsing fails, skip this field
                        }
                    }
                }

                _ => {
                    // Unknown tag, skip it
                }
            }
        }

        // Validate required fields
        let application_context_name = application_context_name
            .ok_or_else(|| nom::Err::Error(Error::new(aarq_content, ErrorKind::Tag)))?;

        Ok((
            remaining,
            Self {
                protocol_version,
                application_context_name,
                called_ap_title,
                called_ae_qualifier,
                called_ap_invocation_id,
                called_ae_invocation_id,
                calling_ap_title,
                calling_ae_qualifier,
                calling_ap_invocation_id,
                calling_ae_invocation_id,
                sender_acse_requirements,
                mechanism_name,
                calling_authentication_value,
                user_information,
            },
        ))
    }
}

impl fmt::Display for AarqApdu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AARQ(ctx={}, mech={:?})", self.application_context_name, self.mechanism_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aarq_new_simple() {
        let aarq = AarqApdu::new_simple_ln(0xFFFF);
        assert_eq!(aarq.protocol_version, PROTOCOL_VERSION);
        assert_eq!(aarq.application_context_name, ApplicationContextName::LogicalNameReferencing);
        assert_eq!(aarq.mechanism_name, Some(MechanismName::LowestLevelSecurity));
        assert!(aarq.user_information.is_some());
    }

    #[test]
    fn test_aarq_new_with_password() {
        let password = b"secret".to_vec();
        let aarq = AarqApdu::new_with_password(0xFFFF, password.clone());
        assert_eq!(aarq.mechanism_name, Some(MechanismName::LowLevelSecurity));
        assert!(matches!(
            aarq.calling_authentication_value,
            Some(AuthenticationValue::CharString(_))
        ));
    }

    #[test]
    fn test_aarq_new_with_ciphering() {
        let system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let aarq = AarqApdu::new_with_ciphering(0xFFFF, system_title);
        assert_eq!(
            aarq.application_context_name,
            ApplicationContextName::LogicalNameReferencingWithCiphering
        );
        assert_eq!(aarq.mechanism_name, Some(MechanismName::HighLevelSecurityGmac));
        assert_eq!(aarq.calling_ap_title, Some(system_title.to_vec()));
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_aarq_encode_simple() {
        let aarq = AarqApdu::new_simple_ln(0x0400);
        let encoded = aarq.encode();

        // Should start with APPLICATION[0] CONSTRUCTED tag
        assert_eq!(encoded[0], 0x60);

        // Should be non-empty
        assert!(!encoded.is_empty());

        // Should contain application context OID for LN
        let ln_oid = ApplicationContextName::LogicalNameReferencing.oid_bytes();
        let oid_present = encoded.windows(ln_oid.len()).any(|w| w == ln_oid);
        assert!(oid_present, "AARQ should contain LN application context OID");
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_aarq_encode_with_password() {
        let password = b"secret123".to_vec();
        let aarq = AarqApdu::new_with_password(0x0400, password.clone());
        let encoded = aarq.encode();

        // Should start with AARQ tag
        assert_eq!(encoded[0], 0x60);

        // Should contain the password
        let password_present = encoded.windows(password.len()).any(|w| w == password.as_slice());
        assert!(password_present, "AARQ should contain authentication password");
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_aarq_encode_with_ciphering() {
        let system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let aarq = AarqApdu::new_with_ciphering(0x0400, system_title);
        let encoded = aarq.encode();

        // Should start with AARQ tag
        assert_eq!(encoded[0], 0x60);

        // Should contain system title
        let title_present = encoded.windows(system_title.len()).any(|w| w == system_title);
        assert!(title_present, "AARQ should contain calling-AP-title (system title)");

        // Should use ciphered application context
        let cipher_oid = ApplicationContextName::LogicalNameReferencingWithCiphering.oid_bytes();
        let oid_present = encoded.windows(cipher_oid.len()).any(|w| w == cipher_oid);
        assert!(oid_present, "AARQ should contain ciphered application context OID");
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_aarq_encode_structure() {
        let aarq = AarqApdu::new_simple_ln(0x0400);
        let encoded = aarq.encode();

        // Verify BER structure
        assert_eq!(encoded[0], 0x60); // APPLICATION[0] CONSTRUCTED

        // Length should be in first few bytes
        let has_length = encoded.len() > 2;
        assert!(has_length);

        // Should contain context-specific tags
        let has_context_tags = encoded.iter().any(|&b| (b & 0xC0) == 0x80);
        assert!(has_context_tags, "AARQ should contain context-specific tags");
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_aarq_roundtrip_simple() {
        let original = AarqApdu::new_simple_ln(0x0400);

        let encoded = original.encode();
        let (remaining, parsed) = AarqApdu::parse(&encoded).expect("Failed to parse AARQ");

        assert!(remaining.is_empty(), "Should consume all input");
        assert_eq!(parsed.protocol_version, original.protocol_version);
        assert_eq!(parsed.application_context_name, original.application_context_name);
        assert_eq!(parsed.mechanism_name, original.mechanism_name);
        assert!(parsed.user_information.is_some());
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_aarq_roundtrip_with_password() {
        let password = b"secret123".to_vec();
        let original = AarqApdu::new_with_password(0x0400, password.clone());

        let encoded = original.encode();
        let (remaining, parsed) = AarqApdu::parse(&encoded).expect("Failed to parse AARQ");

        assert!(remaining.is_empty(), "Should consume all input");
        assert_eq!(parsed.protocol_version, original.protocol_version);
        assert_eq!(parsed.application_context_name, original.application_context_name);
        assert_eq!(parsed.mechanism_name, original.mechanism_name);

        if let Some(AuthenticationValue::CharString(parsed_pw)) =
            parsed.calling_authentication_value
        {
            assert_eq!(parsed_pw, password);
        } else {
            panic!("Expected CharString authentication value");
        }
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_aarq_roundtrip_with_ciphering() {
        let system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let original = AarqApdu::new_with_ciphering(0x0400, system_title);

        let encoded = original.encode();
        let (remaining, parsed) = AarqApdu::parse(&encoded).expect("Failed to parse AARQ");

        assert!(remaining.is_empty(), "Should consume all input");
        assert_eq!(
            parsed.application_context_name,
            ApplicationContextName::LogicalNameReferencingWithCiphering
        );
        assert_eq!(parsed.calling_ap_title, Some(system_title.to_vec()));
        assert_eq!(parsed.mechanism_name, Some(MechanismName::HighLevelSecurityGmac));
    }

    #[test]
    #[cfg(all(feature = "parse", feature = "encode"))]
    fn test_aarq_parse_minimal() {
        // Create a minimal AARQ with required fields
        let init_req = InitiateRequest::new_ln(0x0400);
        let init_encoded = init_req.encode();

        // Build AARQ manually
        let mut minimal_aarq = vec![
            0x60, // APPLICATION[0] CONSTRUCTED (AARQ tag)
        ];

        let mut content = vec![];

        // A0: protocol-version
        content.extend_from_slice(&[0xA0, 0x03, 0x03, 0x01, 0x00]); // BIT STRING: version 1

        // A1: application-context-name (LN)
        content
            .extend_from_slice(&[0xA1, 0x09, 0x06, 0x07, 0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01]);

        // BE: user-information (OCTET STRING containing InitiateRequest)
        content.push(0xBE); // Context-specific 30
        content.push(2 + init_encoded.len() as u8); // Length of OCTET STRING tag+len+data
        content.push(0x04); // OCTET STRING tag
        content.push(init_encoded.len() as u8); // OCTET STRING length
        content.extend_from_slice(&init_encoded);

        // Add total length to AARQ
        minimal_aarq.push(content.len() as u8);
        minimal_aarq.extend_from_slice(&content);

        let (remaining, parsed) =
            AarqApdu::parse(&minimal_aarq).expect("Failed to parse minimal AARQ");

        assert!(remaining.is_empty());
        assert_eq!(parsed.application_context_name, ApplicationContextName::LogicalNameReferencing);
        assert!(parsed.user_information.is_some());
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aarq_with_called_ap_title() {
        // Test encoding with called-AP-title (A2) - server AP title
        let mut aarq = AarqApdu::new_simple_ln(0x0400);
        aarq.called_ap_title = Some(vec![0x01, 0x02, 0x03, 0x04]);

        let encoded = aarq.encode();

        // Should contain A2 tag (0xA2) for called-AP-title
        assert!(encoded.windows(2).any(|w| w[0] == 0xA2));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aarq_with_called_ae_qualifier() {
        // Test encoding with called-AE-qualifier (A3)
        let mut aarq = AarqApdu::new_simple_ln(0x0400);
        aarq.called_ae_qualifier = Some(vec![0x05, 0x06, 0x07, 0x08]);

        let encoded = aarq.encode();

        // Should contain A3 tag (0xA3) for called-AE-qualifier
        assert!(encoded.windows(2).any(|w| w[0] == 0xA3));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aarq_with_calling_ae_qualifier() {
        // Test encoding with calling-AE-qualifier (A7)
        let mut aarq = AarqApdu::new_simple_ln(0x0400);
        aarq.calling_ae_qualifier = Some(vec![0x09, 0x0A, 0x0B, 0x0C]);

        let encoded = aarq.encode();

        // Should contain A7 tag (0xA7) for calling-AE-qualifier
        assert!(encoded.windows(2).any(|w| w[0] == 0xA7));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aarq_with_sender_acse_requirements() {
        // Test encoding with sender-acse-requirements (8A = context-specific 10)
        let mut aarq = AarqApdu::new_simple_ln(0x0400);
        aarq.sender_acse_requirements = Some(0x01); // Authentication required

        let encoded = aarq.encode();

        // Should contain 8A tag (context-specific 10) for sender-acse-requirements
        assert!(encoded.windows(2).any(|w| w[0] == 0x8A));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aarq_with_bitstring_authentication() {
        // Test encoding with BitString authentication value (AC context-specific 12)
        let mut aarq = AarqApdu::new_simple_ln(0x0400);
        aarq.mechanism_name = Some(MechanismName::HighLevelSecurityGmac);
        aarq.calling_authentication_value =
            Some(AuthenticationValue::BitString(vec![0xAA, 0xBB, 0xCC, 0xDD]));

        let encoded = aarq.encode();

        // Should contain AC tag (0xAC) for calling-authentication-value
        assert!(encoded.windows(2).any(|w| w[0] == 0xAC));
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "parse"))]
    fn test_aarq_roundtrip_all_optional_fields() {
        // Test roundtrip with ALL optional fields populated
        let mut original = AarqApdu::new_simple_ln(0x0400);
        original.called_ap_title = Some(vec![0x01, 0x02, 0x03]);
        original.called_ae_qualifier = Some(vec![0x04, 0x05, 0x06]);
        original.calling_ae_qualifier = Some(vec![0x07, 0x08, 0x09]);
        original.sender_acse_requirements = Some(0x02);
        original.mechanism_name = Some(MechanismName::LowLevelSecurity);
        original.calling_authentication_value =
            Some(AuthenticationValue::CharString(b"testpass".to_vec()));

        let encoded = original.encode();
        let (remaining, parsed) =
            AarqApdu::parse(&encoded).expect("Failed to parse AARQ with all fields");

        assert!(remaining.is_empty());
        assert_eq!(parsed.called_ap_title, original.called_ap_title);
        assert_eq!(parsed.called_ae_qualifier, original.called_ae_qualifier);
        assert_eq!(parsed.calling_ae_qualifier, original.calling_ae_qualifier);
        assert_eq!(parsed.sender_acse_requirements, original.sender_acse_requirements);
        assert_eq!(parsed.mechanism_name, original.mechanism_name);
        assert_eq!(parsed.calling_authentication_value, original.calling_authentication_value);
    }
}
