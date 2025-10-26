//! AARE APDU (A-Associate Response)
//!
//! This module implements encoding and parsing of AARE APDUs sent by the server
//! in response to an AARQ to accept or reject an application association.
//!
//! Reference: DLMS Green Book Ed. 12, Section 11.4 and Table 138-139

use alloc::vec::Vec;
use core::fmt;

use super::{
    AcseServiceUserDiagnostics, ApplicationContextName, AssociationResult, AuthenticationValue,
    InitiateResponse, MechanismName, PROTOCOL_VERSION,
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

/// AARE APDU (A-Associate Response) - Tag 0x61
///
/// Sent by the server to accept or reject an application association.
/// Uses ASN.1 BER encoding with context-specific tags.
///
/// Reference: Green Book Table 138-139
#[derive(Debug, Clone, PartialEq)]
pub struct AareApdu {
    /// Protocol version (echoed from AARQ)
    pub protocol_version: u8,
    /// Application context name (negotiated)
    pub application_context_name: ApplicationContextName,
    /// Association result (accepted/rejected)
    pub result: AssociationResult,
    /// Diagnostic information (reason for rejection)
    pub result_source_diagnostic: AcseServiceUserDiagnostics,
    /// Responding AP Title (server) - optional
    pub responding_ap_title: Option<Vec<u8>>,
    /// Responding AE Qualifier (server) - optional
    pub responding_ae_qualifier: Option<Vec<u8>>,
    /// Responding AP invocation ID (server) - optional
    pub responding_ap_invocation_id: Option<u32>,
    /// Responding AE invocation ID (server) - optional
    pub responding_ae_invocation_id: Option<u32>,
    /// Responder ACSE requirements - optional
    pub responder_acse_requirements: Option<u8>,
    /// Authentication mechanism (confirmed)
    pub mechanism_name: Option<MechanismName>,
    /// Authentication value (server challenge for HLS)
    pub responding_authentication_value: Option<AuthenticationValue>,
    /// xDLMS InitiateResponse APDU
    pub user_information: Option<InitiateResponse>,
}

impl AareApdu {
    /// Create a new AARE APDU accepting the association
    ///
    /// # Arguments
    ///
    /// * `context` - Application context name (from AARQ)
    /// * `initiate_response` - xDLMS InitiateResponse with negotiated parameters
    pub fn new_accepted(
        context: ApplicationContextName,
        initiate_response: InitiateResponse,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            application_context_name: context,
            result: AssociationResult::Accepted,
            result_source_diagnostic: AcseServiceUserDiagnostics::Null,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: Some(initiate_response),
        }
    }

    /// Create a new AARE APDU rejecting the association
    ///
    /// # Arguments
    ///
    /// * `context` - Application context name (from AARQ)
    /// * `result` - Rejection type (permanent/transient)
    /// * `diagnostic` - Reason for rejection
    pub fn new_rejected(
        context: ApplicationContextName,
        result: AssociationResult,
        diagnostic: AcseServiceUserDiagnostics,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            application_context_name: context,
            result,
            result_source_diagnostic: diagnostic,
            responding_ap_title: None,
            responding_ae_qualifier: None,
            responding_ap_invocation_id: None,
            responding_ae_invocation_id: None,
            responder_acse_requirements: None,
            mechanism_name: None,
            responding_authentication_value: None,
            user_information: None,
        }
    }

    /// Check if the association was accepted
    pub fn is_accepted(&self) -> bool {
        self.result == AssociationResult::Accepted
    }

    /// Encode to ASN.1 BER format
    ///
    /// Returns the complete AARE APDU including the tag and length
    ///
    /// # BER Structure
    ///
    /// ```text
    /// 61 (APPLICATION 1 CONSTRUCTED) - AARE
    ///   A0 (CONTEXT 0) - protocol-version [OPTIONAL]
    ///   A1 (CONTEXT 1) - application-context-name
    ///   A2 (CONTEXT 2) - result
    ///   A3 (CONTEXT 3) - result-source-diagnostic
    ///   A4 (CONTEXT 4) - responding-AP-title [OPTIONAL]
    ///   A5 (CONTEXT 5) - responding-AE-qualifier [OPTIONAL]
    ///   A6 (CONTEXT 6) - responding-AP-invocation-id [OPTIONAL]
    ///   A7 (CONTEXT 7) - responding-AE-invocation-id [OPTIONAL]
    ///   88 (CONTEXT 8 PRIMITIVE) - responder-acse-requirements [OPTIONAL]
    ///   89 (CONTEXT 9 PRIMITIVE) - mechanism-name [OPTIONAL]
    ///   AA (CONTEXT 10) - responding-authentication-value [OPTIONAL]
    ///   BE (CONTEXT 30) - user-information [OPTIONAL]
    /// ```
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut content = Vec::new();

        // A0: protocol-version (BIT STRING, default 0 = version 1)
        let version_bits = if self.protocol_version == 0 { &[0x80] } else { &[0x00] };
        let protocol_version_encoded = encode_bit_string(version_bits, 7);
        content.extend(encode_context_specific(0, TagType::Constructed, &protocol_version_encoded));

        // A1: application-context-name (OBJECT IDENTIFIER)
        let app_context_oid = encode_object_identifier(self.application_context_name.oid_bytes());
        content.extend(encode_context_specific(1, TagType::Constructed, &app_context_oid));

        // A2: result (INTEGER)
        // Encode as primitive INTEGER: tag 0x02, length 0x01, value
        let result_encoded = alloc::vec![0x02, 0x01, self.result.as_u8()];
        content.extend(encode_context_specific(2, TagType::Constructed, &result_encoded));

        // A3: result-source-diagnostic (CHOICE)
        // CHOICE[1] acse-service-user: INTEGER
        let diagnostic_value = alloc::vec![0x02, 0x01, self.result_source_diagnostic.as_u8()];
        let diagnostic_choice = encode_context_specific(1, TagType::Constructed, &diagnostic_value);
        content.extend(encode_context_specific(3, TagType::Constructed, &diagnostic_choice));

        // A4: responding-AP-title (OPTIONAL)
        if let Some(ref title) = self.responding_ap_title {
            let title_encoded = encode_octet_string(title);
            content.extend(encode_context_specific(4, TagType::Constructed, &title_encoded));
        }

        // A5: responding-AE-qualifier (OPTIONAL)
        if let Some(ref qualifier) = self.responding_ae_qualifier {
            let qualifier_encoded = encode_octet_string(qualifier);
            content.extend(encode_context_specific(5, TagType::Constructed, &qualifier_encoded));
        }

        // A6-A7: invocation IDs (OPTIONAL) - not commonly used, skip for now

        // 88: responder-acse-requirements (OPTIONAL)
        if let Some(acse_req) = self.responder_acse_requirements {
            let acse_bits = encode_bit_string(&[acse_req], 0);
            content.extend(encode_context_specific(8, TagType::Primitive, &acse_bits));
        }

        // 89: mechanism-name (OPTIONAL)
        if let Some(ref mechanism) = self.mechanism_name {
            let mechanism_oid = encode_object_identifier(mechanism.oid_bytes());
            content.extend(encode_context_specific(9, TagType::Primitive, &mechanism_oid));
        }

        // AA: responding-authentication-value (OPTIONAL)
        if let Some(ref auth_value) = self.responding_authentication_value {
            let auth_encoded = match auth_value {
                AuthenticationValue::CharString(password) => {
                    let mut auth_content = Vec::new();
                    auth_content.extend(encode_context_specific(0, TagType::Primitive, password));
                    auth_content
                }
                AuthenticationValue::BitString(bits) => {
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
            content.extend(encode_context_specific(10, TagType::Constructed, &auth_encoded));
        }

        // BE: user-information (OPTIONAL) - contains xDLMS InitiateResponse
        if let Some(ref user_info) = self.user_information {
            let initiate_encoded = user_info.encode();
            let user_info_octets = encode_octet_string(&initiate_encoded);
            content.extend(encode_context_specific(30, TagType::Constructed, &user_info_octets));
        }

        // Wrap in APPLICATION[1] CONSTRUCTED tag (0x61)
        encode_application(1, TagType::Constructed, &content)
    }

    /// Parse from ASN.1 BER format
    ///
    /// # BER Structure
    ///
    /// ```text
    /// 61 (APPLICATION 1 CONSTRUCTED) - AARE
    ///   A0 (CONTEXT 0) - protocol-version [OPTIONAL]
    ///   A1 (CONTEXT 1) - application-context-name
    ///   A2 (CONTEXT 2) - result
    ///   A3 (CONTEXT 3) - result-source-diagnostic
    ///   A4 (CONTEXT 4) - responding-AP-title [OPTIONAL]
    ///   A5 (CONTEXT 5) - responding-AE-qualifier [OPTIONAL]
    ///   88 (CONTEXT 8 PRIMITIVE) - responder-acse-requirements [OPTIONAL]
    ///   89 (CONTEXT 9 PRIMITIVE) - mechanism-name [OPTIONAL]
    ///   AA (CONTEXT 10) - responding-authentication-value [OPTIONAL]
    ///   BE (CONTEXT 30) - user-information [OPTIONAL]
    /// ```
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        // Parse APPLICATION[1] CONSTRUCTED tag (0x61)
        let (input, (class, tag_type, tag_number)) = parse_tag(input)?;
        if class != TagClass::Application || tag_number != 1 {
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

        // Extract AARE content
        let aare_content = &input[..length];
        let remaining = &input[length..];

        // Parse AARE fields from content
        let mut content = aare_content;

        let mut protocol_version = PROTOCOL_VERSION;
        let mut application_context_name = None;
        let mut result = None;
        let mut result_source_diagnostic = None;
        let mut responding_ap_title = None;
        let mut responding_ae_qualifier = None;
        let mut responding_ap_invocation_id = None;
        let mut responding_ae_invocation_id = None;
        let mut responder_acse_requirements = None;
        let mut mechanism_name = None;
        let mut responding_authentication_value = None;
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

                // A2: result (INTEGER)
                2 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    // Parse INTEGER tag
                    let (field_content, int_tag) = nom_u8(field_content)?;
                    if int_tag != 0x02 {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (field_content, int_len) = nom_u8(field_content)?;
                    if int_len != 1 || field_content.is_empty() {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Eof)));
                    }
                    let (_, result_value) = nom_u8(field_content)?;
                    result = AssociationResult::from_u8(result_value);
                }

                // A3: result-source-diagnostic (CHOICE)
                3 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    // Parse nested CHOICE[1] acse-service-user
                    let (field_content, (choice_class, choice_type, choice_tag)) =
                        parse_tag(field_content)?;
                    if choice_class != TagClass::ContextSpecific || choice_tag != 1 {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    if choice_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }

                    let (field_content, choice_len) = parse_length(field_content)?;
                    if field_content.len() < choice_len {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Eof)));
                    }

                    // Parse INTEGER inside choice
                    let (field_content, int_tag) = nom_u8(field_content)?;
                    if int_tag != 0x02 {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (field_content, int_len) = nom_u8(field_content)?;
                    if int_len != 1 || field_content.is_empty() {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Eof)));
                    }
                    let (_, diag_value) = nom_u8(field_content)?;
                    result_source_diagnostic = AcseServiceUserDiagnostics::from_u8(diag_value);
                }

                // A4: responding-AP-title (OCTET STRING)
                4 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, octets) = parse_octet_string(field_content)?;
                    responding_ap_title = Some(octets);
                }

                // A5: responding-AE-qualifier (OCTET STRING)
                5 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, octets) = parse_octet_string(field_content)?;
                    responding_ae_qualifier = Some(octets);
                }

                // A6: responding-AP-invocation-id (INTEGER)
                6 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    // Parse INTEGER - simplified for now
                    let (field_content, int_tag) = nom_u8(field_content)?;
                    if int_tag == 0x02 {
                        let (field_content, int_len) = nom_u8(field_content)?;
                        if int_len <= 4 && field_content.len() >= int_len as usize {
                            let mut value: u32 = 0;
                            for &byte in field_content.iter().take(int_len as usize) {
                                value = (value << 8) | byte as u32;
                            }
                            responding_ap_invocation_id = Some(value);
                        }
                    }
                }

                // A7: responding-AE-invocation-id (INTEGER)
                7 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    // Parse INTEGER - simplified for now
                    let (field_content, int_tag) = nom_u8(field_content)?;
                    if int_tag == 0x02 {
                        let (field_content, int_len) = nom_u8(field_content)?;
                        if int_len <= 4 && field_content.len() >= int_len as usize {
                            let mut value: u32 = 0;
                            for &byte in field_content.iter().take(int_len as usize) {
                                value = (value << 8) | byte as u32;
                            }
                            responding_ae_invocation_id = Some(value);
                        }
                    }
                }

                // 88 (CONTEXT 8 PRIMITIVE): responder-acse-requirements (BIT STRING)
                8 => {
                    if tag_type != TagType::Primitive {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, (bits, _unused)) = parse_bit_string(field_content)?;
                    if !bits.is_empty() {
                        responder_acse_requirements = Some(bits[0]);
                    }
                }

                // 89 (CONTEXT 9 PRIMITIVE): mechanism-name (OBJECT IDENTIFIER)
                9 => {
                    if tag_type != TagType::Primitive {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, oid_bytes) = parse_object_identifier(field_content)?;
                    mechanism_name = MechanismName::from_oid_bytes(&oid_bytes);
                }

                // AA (CONTEXT 10): responding-authentication-value (CHOICE)
                10 => {
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
                        responding_authentication_value =
                            Some(AuthenticationValue::CharString(choice_content.to_vec()));
                    } else if choice_tag == 1 && choice_type == TagType::Primitive {
                        // BitString
                        let (_remaining, (bits, _unused)) = parse_bit_string(choice_content)?;
                        responding_authentication_value =
                            Some(AuthenticationValue::BitString(bits));
                    }
                }

                // BE (CONTEXT 30): user-information (OCTET STRING containing InitiateResponse)
                30 => {
                    if tag_type != TagType::Constructed {
                        return Err(nom::Err::Error(Error::new(field_content, ErrorKind::Tag)));
                    }
                    let (_remaining, octets) = parse_octet_string(field_content)?;
                    // Parse InitiateResponse from the octet string
                    match InitiateResponse::parse(&octets) {
                        Ok((_, init_resp)) => {
                            user_information = Some(init_resp);
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
            .ok_or_else(|| nom::Err::Error(Error::new(aare_content, ErrorKind::Tag)))?;
        let result =
            result.ok_or_else(|| nom::Err::Error(Error::new(aare_content, ErrorKind::Tag)))?;
        let result_source_diagnostic = result_source_diagnostic
            .ok_or_else(|| nom::Err::Error(Error::new(aare_content, ErrorKind::Tag)))?;

        Ok((
            remaining,
            Self {
                protocol_version,
                application_context_name,
                result,
                result_source_diagnostic,
                responding_ap_title,
                responding_ae_qualifier,
                responding_ap_invocation_id,
                responding_ae_invocation_id,
                responder_acse_requirements,
                mechanism_name,
                responding_authentication_value,
                user_information,
            },
        ))
    }
}

impl fmt::Display for AareApdu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AARE(result={}, ctx={}, diag={})",
            self.result, self.application_context_name, self.result_source_diagnostic
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::association::Conformance;

    #[test]
    fn test_aare_new_accepted() {
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let aare =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);

        assert_eq!(aare.protocol_version, PROTOCOL_VERSION);
        assert_eq!(aare.result, AssociationResult::Accepted);
        assert!(aare.is_accepted());
        assert_eq!(aare.result_source_diagnostic, AcseServiceUserDiagnostics::Null);
        assert!(aare.user_information.is_some());
    }

    #[test]
    fn test_aare_new_rejected() {
        let aare = AareApdu::new_rejected(
            ApplicationContextName::LogicalNameReferencing,
            AssociationResult::RejectedPermanent,
            AcseServiceUserDiagnostics::AuthenticationFailure,
        );

        assert_eq!(aare.result, AssociationResult::RejectedPermanent);
        assert!(!aare.is_accepted());
        assert_eq!(
            aare.result_source_diagnostic,
            AcseServiceUserDiagnostics::AuthenticationFailure
        );
        assert!(aare.user_information.is_none());
    }

    #[test]
    fn test_aare_display() {
        let aare = AareApdu::new_rejected(
            ApplicationContextName::LogicalNameReferencing,
            AssociationResult::RejectedPermanent,
            AcseServiceUserDiagnostics::AuthenticationRequired,
        );

        let display = format!("{}", aare);
        assert!(display.contains("AARE"));
        assert!(display.contains("Rejected"));
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_aare_encode_accepted() {
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let aare =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);
        let encoded = aare.encode();

        // Should start with APPLICATION[1] CONSTRUCTED tag
        assert_eq!(encoded[0], 0x61);

        // Should be non-empty
        assert!(!encoded.is_empty());

        // Should contain application context OID for LN
        let ln_oid = ApplicationContextName::LogicalNameReferencing.oid_bytes();
        let oid_present = encoded.windows(ln_oid.len()).any(|w| w == ln_oid);
        assert!(oid_present, "AARE should contain LN application context OID");

        // Should contain result = 0 (accepted)
        let result_present = encoded.windows(3).any(|w| w == [0x02, 0x01, 0x00]);
        assert!(result_present, "AARE should contain result=0 (accepted)");
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_aare_encode_rejected() {
        let aare = AareApdu::new_rejected(
            ApplicationContextName::LogicalNameReferencing,
            AssociationResult::RejectedPermanent,
            AcseServiceUserDiagnostics::AuthenticationFailure,
        );
        let encoded = aare.encode();

        // Should start with AARE tag
        assert_eq!(encoded[0], 0x61);

        // Should contain result = 1 (rejected permanent)
        let result_present = encoded.windows(3).any(|w| w == [0x02, 0x01, 0x01]);
        assert!(result_present, "AARE should contain result=1 (rejected permanent)");
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_aare_encode_structure() {
        let initiate_resp = InitiateResponse::new_ln(Conformance::GET | Conformance::SET, 0x0400);
        let aare =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);
        let encoded = aare.encode();

        // Verify BER structure
        assert_eq!(encoded[0], 0x61); // APPLICATION[1] CONSTRUCTED

        // Length should be in first few bytes
        let has_length = encoded.len() > 2;
        assert!(has_length);

        // Should contain context-specific tags
        let has_context_tags = encoded.iter().any(|&b| (b & 0xC0) == 0x80);
        assert!(has_context_tags, "AARE should contain context-specific tags");
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_aare_roundtrip_accepted() {
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let original =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);

        let encoded = original.encode();
        let (remaining, parsed) = AareApdu::parse(&encoded).expect("Failed to parse AARE");

        assert!(remaining.is_empty(), "Should consume all input");
        assert_eq!(parsed.protocol_version, original.protocol_version);
        assert_eq!(parsed.application_context_name, original.application_context_name);
        assert_eq!(parsed.result, original.result);
        assert_eq!(parsed.result_source_diagnostic, original.result_source_diagnostic);
        assert!(parsed.is_accepted());
        assert!(parsed.user_information.is_some());
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_aare_roundtrip_rejected() {
        let original = AareApdu::new_rejected(
            ApplicationContextName::LogicalNameReferencing,
            AssociationResult::RejectedPermanent,
            AcseServiceUserDiagnostics::AuthenticationFailure,
        );

        let encoded = original.encode();
        let (remaining, parsed) = AareApdu::parse(&encoded).expect("Failed to parse AARE");

        assert!(remaining.is_empty(), "Should consume all input");
        assert_eq!(parsed.protocol_version, original.protocol_version);
        assert_eq!(parsed.application_context_name, original.application_context_name);
        assert_eq!(parsed.result, original.result);
        assert_eq!(parsed.result_source_diagnostic, original.result_source_diagnostic);
        assert!(!parsed.is_accepted());
        assert!(parsed.user_information.is_none());
    }

    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_aare_roundtrip_with_ciphering() {
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x1000);
        let mut aare = AareApdu::new_accepted(
            ApplicationContextName::LogicalNameReferencingWithCiphering,
            initiate_resp,
        );

        // Add server system title
        aare.responding_ap_title = Some(vec![0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E]);
        aare.mechanism_name = Some(MechanismName::LowLevelSecurity);

        let encoded = aare.encode();
        let (remaining, parsed) = AareApdu::parse(&encoded).expect("Failed to parse AARE");

        assert!(remaining.is_empty(), "Should consume all input");
        assert_eq!(
            parsed.application_context_name,
            ApplicationContextName::LogicalNameReferencingWithCiphering
        );
        assert_eq!(parsed.responding_ap_title, aare.responding_ap_title);
        assert_eq!(parsed.mechanism_name, aare.mechanism_name);
    }

    #[test]
    #[cfg(all(feature = "parse", feature = "encode"))]
    fn test_aare_parse_minimal() {
        // Minimal AARE: only required fields (protocol version, context, result, diagnostic)
        // First, create a valid InitiateResponse and encode it
        let init_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let init_encoded = init_resp.encode();

        // Build AARE manually
        let mut minimal_aare = vec![
            0x61, // APPLICATION[1] CONSTRUCTED (AARE tag)
        ];

        // Calculate total length (we'll update this)
        let mut content = vec![];

        // A0: protocol-version
        content.extend_from_slice(&[0xA0, 0x03, 0x03, 0x01, 0x00]); // BIT STRING: version 1

        // A1: application-context-name (LN)
        content
            .extend_from_slice(&[0xA1, 0x09, 0x06, 0x07, 0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01]);

        // A2: result = 0 (accepted)
        content.extend_from_slice(&[0xA2, 0x03, 0x02, 0x01, 0x00]);

        // A3: result-source-diagnostic (acse-service-user = 0)
        content.extend_from_slice(&[0xA3, 0x05, 0xA1, 0x03, 0x02, 0x01, 0x00]);

        // BE: user-information (OCTET STRING containing InitiateResponse)
        content.push(0xBE); // Context-specific 30
        content.push(2 + init_encoded.len() as u8); // Length of OCTET STRING tag+len+data
        content.push(0x04); // OCTET STRING tag
        content.push(init_encoded.len() as u8); // OCTET STRING length
        content.extend_from_slice(&init_encoded);

        // Add total length to AARE
        minimal_aare.push(content.len() as u8);
        minimal_aare.extend_from_slice(&content);

        let (remaining, parsed) =
            AareApdu::parse(&minimal_aare).expect("Failed to parse minimal AARE");

        assert!(remaining.is_empty());
        assert_eq!(parsed.result, AssociationResult::Accepted);
        assert_eq!(parsed.application_context_name, ApplicationContextName::LogicalNameReferencing);
        assert!(parsed.is_accepted());
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aare_with_responding_ap_title() {
        // Test encoding with responding-AP-title (A4) - server AP title
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let mut aare =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);
        aare.responding_ap_title = Some(vec![0x01, 0x02, 0x03, 0x04]);

        let encoded = aare.encode();

        // Should contain A4 tag (0xA4) for responding-AP-title
        assert!(encoded.windows(2).any(|w| w[0] == 0xA4));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aare_with_responding_ae_qualifier() {
        // Test encoding with responding-AE-qualifier (A5)
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let mut aare =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);
        aare.responding_ae_qualifier = Some(vec![0x05, 0x06, 0x07, 0x08]);

        let encoded = aare.encode();

        // Should contain A5 tag (0xA5) for responding-AE-qualifier
        assert!(encoded.windows(2).any(|w| w[0] == 0xA5));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aare_with_responder_acse_requirements() {
        // Test encoding with responder-acse-requirements (88 = context-specific 8)
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let mut aare =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);
        aare.responder_acse_requirements = Some(0x01); // Authentication required

        let encoded = aare.encode();

        // Should contain 88 tag (context-specific 8) for responder-acse-requirements
        assert!(encoded.windows(2).any(|w| w[0] == 0x88));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_aare_with_responding_authentication() {
        // Test encoding with responding-authentication-value (AA = context-specific 10)
        // Used for HLS - server challenge in response
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let mut aare = AareApdu::new_accepted(
            ApplicationContextName::LogicalNameReferencingWithCiphering,
            initiate_resp,
        );
        aare.mechanism_name = Some(MechanismName::HighLevelSecurityGmac);
        aare.responding_authentication_value =
            Some(AuthenticationValue::BitString(vec![0xAA, 0xBB, 0xCC, 0xDD]));

        let encoded = aare.encode();

        // Should contain AA tag (0xAA) for responding-authentication-value
        assert!(encoded.windows(2).any(|w| w[0] == 0xAA));
    }

    #[test]
    #[cfg(all(feature = "encode", feature = "parse"))]
    fn test_aare_roundtrip_all_optional_fields() {
        // Test roundtrip with ALL optional fields populated
        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let mut original =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);
        original.responding_ap_title = Some(vec![0x01, 0x02, 0x03]);
        original.responding_ae_qualifier = Some(vec![0x04, 0x05, 0x06]);
        original.responder_acse_requirements = Some(0x02);
        original.mechanism_name = Some(MechanismName::LowLevelSecurity);
        original.responding_authentication_value =
            Some(AuthenticationValue::CharString(b"challenge".to_vec()));

        let encoded = original.encode();
        let (remaining, parsed) =
            AareApdu::parse(&encoded).expect("Failed to parse AARE with all fields");

        assert!(remaining.is_empty());
        assert_eq!(parsed.responding_ap_title, original.responding_ap_title);
        assert_eq!(parsed.responding_ae_qualifier, original.responding_ae_qualifier);
        assert_eq!(parsed.responder_acse_requirements, original.responder_acse_requirements);
        assert_eq!(parsed.mechanism_name, original.mechanism_name);
        assert_eq!(
            parsed.responding_authentication_value,
            original.responding_authentication_value
        );
    }

    #[test]
    fn test_aare_rejected_diagnostics() {
        // Test all diagnostic variants per Green Book Table 139
        let diagnostics = vec![
            AcseServiceUserDiagnostics::Null,
            AcseServiceUserDiagnostics::NoReasonGiven,
            AcseServiceUserDiagnostics::ApplicationContextNameNotSupported,
            AcseServiceUserDiagnostics::AuthenticationMechanismNameNotRecognised,
            AcseServiceUserDiagnostics::AuthenticationMechanismNameRequired,
            AcseServiceUserDiagnostics::AuthenticationFailure,
            AcseServiceUserDiagnostics::AuthenticationRequired,
        ];

        for diag in diagnostics {
            let aare = AareApdu::new_rejected(
                ApplicationContextName::LogicalNameReferencing,
                AssociationResult::RejectedPermanent,
                diag,
            );
            assert_eq!(aare.result, AssociationResult::RejectedPermanent);
            assert_eq!(aare.result_source_diagnostic, diag);
            assert!(!aare.is_accepted());
        }
    }
}
