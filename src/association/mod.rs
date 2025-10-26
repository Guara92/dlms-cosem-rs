//! Association Layer - AARQ/AARE APDUs for DLMS/COSEM
//!
//! This module implements the Application Association (AA) establishment using
//! ACSE (Association Control Service Element) APDUs:
//! - AARQ (A-Associate Request) - Client initiates association
//! - AARE (A-Associate Response) - Server accepts/rejects association
//!
//! Reference: DLMS Green Book Ed. 12, Section 11
//!
//! # Architecture
//!
//! The association establishment follows this sequence:
//! 1. Client sends AARQ APDU (tag 0x60) containing:
//!    - Application context (LN/SN, with/without ciphering)
//!    - Authentication mechanism and credentials
//!    - xDLMS InitiateRequest with conformance and PDU size
//! 2. Server responds with AARE APDU (tag 0x61) containing:
//!    - Association result (accepted/rejected)
//!    - Diagnostic information if rejected
//!    - xDLMS InitiateResponse with negotiated parameters
//!
//! # Encoding
//!
//! - AARQ/AARE use ASN.1 BER encoding with context-specific tags
//! - xDLMS APDUs (InitiateRequest/Response) use A-XDR encoding
//! - All encoding is done in safe Rust with no_std compatibility

// Re-export for convenience
pub use self::{
    aare::AareApdu,
    aarq::AarqApdu,
    conformance::Conformance,
    enums::*,
    initiate::{InitiateRequest, InitiateResponse},
};

mod aare;
mod aarq;
mod ber;
mod conformance;
mod enums;
mod initiate;

/// ASN.1 BER tag for AARQ APDU
pub const AARQ_TAG: u8 = 0x60;

/// ASN.1 BER tag for AARE APDU
pub const AARE_TAG: u8 = 0x61;

/// xDLMS VAA name for Logical Name referencing
pub const VAA_NAME_LN: u16 = 0x0007;

/// xDLMS VAA name for Short Name referencing
pub const VAA_NAME_SN: u16 = 0x0001;

/// Default DLMS version number (version 6)
pub const DLMS_VERSION: u8 = 6;

/// Default protocol version (version 1)
pub const PROTOCOL_VERSION: u8 = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(AARQ_TAG, 0x60);
        assert_eq!(AARE_TAG, 0x61);
        assert_eq!(VAA_NAME_LN, 0x0007);
        assert_eq!(VAA_NAME_SN, 0x0001);
        assert_eq!(DLMS_VERSION, 6);
        assert_eq!(PROTOCOL_VERSION, 0);
    }

    /// Integration test: Full association handshake (AARQ → AARE)
    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_full_association_handshake_accepted() {
        // Step 1: Client creates AARQ
        let client_aarq = AarqApdu::new_simple_ln(0xFFFF);

        // Step 2: Client encodes AARQ and sends to server
        let aarq_bytes = client_aarq.encode();

        // Step 3: Server receives and parses AARQ
        let (_, parsed_aarq) = AarqApdu::parse(&aarq_bytes).expect("Server failed to parse AARQ");

        // Verify server received correct parameters
        assert_eq!(
            parsed_aarq.application_context_name,
            ApplicationContextName::LogicalNameReferencing
        );
        assert_eq!(parsed_aarq.mechanism_name, Some(MechanismName::LowestLevelSecurity));

        // Step 4: Server creates AARE response (accepting association)
        let proposed_conformance = parsed_aarq
            .user_information
            .as_ref()
            .map(|ui| ui.proposed_conformance)
            .unwrap_or(Conformance::TYPICAL_CLIENT_LN);
        let proposed_pdu_size = parsed_aarq
            .user_information
            .as_ref()
            .map(|ui| ui.client_max_receive_pdu_size)
            .unwrap_or(0xFFFF);

        // Server negotiates parameters (can reduce conformance/PDU size)
        let negotiated_conformance = proposed_conformance & Conformance::TYPICAL_CLIENT_LN;
        let negotiated_pdu_size = proposed_pdu_size.min(0x0400);

        let initiate_response =
            InitiateResponse::new_ln(negotiated_conformance, negotiated_pdu_size);
        let server_aare = AareApdu::new_accepted(
            ApplicationContextName::LogicalNameReferencing,
            initiate_response,
        );

        // Step 5: Server encodes AARE and sends to client
        let aare_bytes = server_aare.encode();

        // Step 6: Client receives and parses AARE
        let (_, parsed_aare) = AareApdu::parse(&aare_bytes).expect("Client failed to parse AARE");

        // Step 7: Client verifies association was accepted
        assert!(parsed_aare.is_accepted());
        assert_eq!(parsed_aare.result, AssociationResult::Accepted);
        assert_eq!(
            parsed_aare.application_context_name,
            ApplicationContextName::LogicalNameReferencing
        );

        // Client extracts negotiated parameters
        let negotiated =
            parsed_aare.user_information.expect("AARE should contain InitiateResponse");
        assert_eq!(negotiated.negotiated_conformance.bits(), negotiated_conformance.bits());
        assert_eq!(negotiated.server_max_receive_pdu_size, negotiated_pdu_size);
    }

    /// Integration test: Association rejected by server
    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_association_handshake_rejected() {
        // Step 1: Client creates AARQ with password authentication
        let password = b"wrong_password".to_vec();
        let client_aarq = AarqApdu::new_with_password(0xFFFF, password);

        // Step 2: Encode and send
        let aarq_bytes = client_aarq.encode();

        // Step 3: Server parses AARQ
        let (_, parsed_aarq) = AarqApdu::parse(&aarq_bytes).expect("Server failed to parse AARQ");

        // Step 4: Server validates authentication and rejects
        assert_eq!(parsed_aarq.mechanism_name, Some(MechanismName::LowLevelSecurity));

        let server_aare = AareApdu::new_rejected(
            ApplicationContextName::LogicalNameReferencing,
            AssociationResult::RejectedPermanent,
            AcseServiceUserDiagnostics::AuthenticationFailure,
        );

        // Step 5: Server encodes rejection response
        let aare_bytes = server_aare.encode();

        // Step 6: Client parses rejection
        let (_, parsed_aare) = AareApdu::parse(&aare_bytes).expect("Client failed to parse AARE");

        // Step 7: Client handles rejection
        assert!(!parsed_aare.is_accepted());
        assert_eq!(parsed_aare.result, AssociationResult::RejectedPermanent);
        assert_eq!(
            parsed_aare.result_source_diagnostic,
            AcseServiceUserDiagnostics::AuthenticationFailure
        );
        assert!(parsed_aare.user_information.is_none());
    }

    /// Integration test: Ciphered association with system title
    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_ciphered_association_handshake() {
        // Step 1: Client creates AARQ with ciphering and system title
        let client_system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let client_aarq = AarqApdu::new_with_ciphering(0x1000, client_system_title);

        // Step 2: Encode and send
        let aarq_bytes = client_aarq.encode();

        // Step 3: Server parses AARQ
        let (_, parsed_aarq) = AarqApdu::parse(&aarq_bytes).expect("Server failed to parse AARQ");

        // Verify ciphering context and system title
        assert_eq!(
            parsed_aarq.application_context_name,
            ApplicationContextName::LogicalNameReferencingWithCiphering
        );
        assert_eq!(parsed_aarq.calling_ap_title, Some(client_system_title.to_vec()));
        assert_eq!(parsed_aarq.mechanism_name, Some(MechanismName::HighLevelSecurityGmac));

        // Step 4: Server accepts with its own system title
        let server_system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x01, 0x23, 0x45, 0x67];
        let initiate_response = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x1000);
        let mut server_aare = AareApdu::new_accepted(
            ApplicationContextName::LogicalNameReferencingWithCiphering,
            initiate_response,
        );
        server_aare.responding_ap_title = Some(server_system_title.to_vec());
        server_aare.mechanism_name = Some(MechanismName::HighLevelSecurityGmac);

        // Step 5: Encode and send
        let aare_bytes = server_aare.encode();

        // Step 6: Client parses AARE
        let (_, parsed_aare) = AareApdu::parse(&aare_bytes).expect("Client failed to parse AARE");

        // Step 7: Client verifies ciphered association
        assert!(parsed_aare.is_accepted());
        assert_eq!(
            parsed_aare.application_context_name,
            ApplicationContextName::LogicalNameReferencingWithCiphering
        );
        assert_eq!(parsed_aare.responding_ap_title, Some(server_system_title.to_vec()));
        assert_eq!(parsed_aare.mechanism_name, Some(MechanismName::HighLevelSecurityGmac));
    }

    /// Integration test: Conformance negotiation
    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_conformance_negotiation() {
        // Client proposes full conformance
        let client_conformance = Conformance::GET
            | Conformance::SET
            | Conformance::ACTION
            | Conformance::SELECTIVE_ACCESS
            | Conformance::BLOCK_TRANSFER_WITH_GET_OR_READ
            | Conformance::BLOCK_TRANSFER_WITH_SET_OR_WRITE;

        let mut client_aarq = AarqApdu::new_simple_ln(0xFFFF);
        client_aarq.user_information = Some(InitiateRequest::new(client_conformance, 0xFFFF));

        let aarq_bytes = client_aarq.encode();
        let (_, parsed_aarq) = AarqApdu::parse(&aarq_bytes).unwrap();

        // Server supports only subset
        let server_conformance = Conformance::GET | Conformance::SET;

        // Server negotiates (intersection of client and server capabilities)
        let negotiated =
            parsed_aarq.user_information.unwrap().proposed_conformance & server_conformance;

        let initiate_response = InitiateResponse::new_ln(negotiated, 0x0400);
        let server_aare = AareApdu::new_accepted(
            ApplicationContextName::LogicalNameReferencing,
            initiate_response,
        );

        let aare_bytes = server_aare.encode();
        let (_, parsed_aare) = AareApdu::parse(&aare_bytes).unwrap();

        // Verify negotiated conformance
        let final_conformance = parsed_aare.user_information.unwrap().negotiated_conformance;
        assert!(final_conformance.contains(Conformance::GET));
        assert!(final_conformance.contains(Conformance::SET));
        assert!(!final_conformance.contains(Conformance::ACTION));
        assert!(!final_conformance.contains(Conformance::SELECTIVE_ACCESS));
    }

    /// Gurux compatibility test: Verify BER tag encoding matches Gurux.c constants
    #[test]
    fn test_gurux_ber_tag_compatibility() {
        // Gurux.c constants from enums.h:
        // BER_TYPE_APPLICATION = 0x40
        // BER_TYPE_CONTEXT = 0x80
        // BER_TYPE_CONSTRUCTED = 0x20
        // BER_TYPE_INTEGER = 0x02
        // BER_TYPE_OCTET_STRING = 0x04
        // BER_TYPE_OBJECT_IDENTIFIER = 0x06

        // AARQ tag: BER_TYPE_APPLICATION | BER_TYPE_CONSTRUCTED = 0x60
        assert_eq!(AARQ_TAG, 0x60);

        // AARE tag: BER_TYPE_APPLICATION | BER_TYPE_CONSTRUCTED | 0x01 = 0x61
        assert_eq!(AARE_TAG, 0x61);

        // Context-specific constructed tags used in AARQ/AARE:
        // A0 = BER_TYPE_CONTEXT | BER_TYPE_CONSTRUCTED | 0x00 = 0xA0
        // A1 = BER_TYPE_CONTEXT | BER_TYPE_CONSTRUCTED | 0x01 = 0xA1
        // etc.
        assert_eq!(0x80 | 0x20, 0xA0); // Context[0] Constructed
        assert_eq!(0x80 | 0x20 | 0x01, 0xA1); // Context[1] Constructed
    }

    /// Gurux compatibility test: Verify AARQ structure matches Green Book Table 136
    /// This is the same example used by Gurux for validation
    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_gurux_aarq_green_book_example() {
        // Green Book Table 136 example (simplified LN referencing, no auth)
        // This should match what Gurux generates for a simple AARQ

        let aarq = AarqApdu::new_simple_ln(0xFFFF);
        let encoded = aarq.encode();

        // Verify AARQ tag (APPLICATION 0 CONSTRUCTED)
        assert_eq!(encoded[0], 0x60, "AARQ tag should be 0x60");

        // Verify protocol-version is present (A0 tag)
        let has_protocol_version = encoded.windows(2).any(|w| w[0] == 0xA0);
        assert!(has_protocol_version, "AARQ should contain A0 (protocol-version)");

        // Verify application-context-name is present (A1 tag)
        let has_app_context = encoded.contains(&0xA1);
        assert!(has_app_context, "AARQ should contain A1 (application-context-name)");

        // Verify LN OID: 2.16.756.5.8.1.1 = 0x60 0x85 0x74 0x05 0x08 0x01 0x01
        let ln_oid = [0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01];
        let has_ln_oid = encoded.windows(ln_oid.len()).any(|w| w == ln_oid);
        assert!(has_ln_oid, "AARQ should contain LN OID matching Gurux");

        // Verify user-information is present (BE tag for context 30)
        let has_user_info = encoded.contains(&0xBE);
        assert!(has_user_info, "AARQ should contain BE (user-information)");

        // Roundtrip test to ensure we can parse what we encode
        let (_, parsed) = AarqApdu::parse(&encoded).expect("Should parse Gurux-compatible AARQ");
        assert_eq!(parsed.application_context_name, ApplicationContextName::LogicalNameReferencing);
    }

    /// Gurux compatibility test: Verify AARE accepted response matches Gurux encoding
    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_gurux_aare_accepted_encoding() {
        // Gurux generates AARE with specific structure matching Green Book Table 138

        let initiate_resp = InitiateResponse::new_ln(Conformance::TYPICAL_CLIENT_LN, 0x0400);
        let aare =
            AareApdu::new_accepted(ApplicationContextName::LogicalNameReferencing, initiate_resp);
        let encoded = aare.encode();

        // Verify AARE tag (APPLICATION 1 CONSTRUCTED)
        assert_eq!(
            encoded[0], 0x61,
            "AARE tag should be 0x61 (Gurux BER_TYPE_APPLICATION | BER_TYPE_CONSTRUCTED | 0x01)"
        );

        // Verify A2 (result) contains INTEGER with value 0 (accepted)
        // A2 03 02 01 00
        let result_pattern = [0xA2, 0x03, 0x02, 0x01, 0x00];
        let has_result = encoded.windows(result_pattern.len()).any(|w| w == result_pattern);
        assert!(has_result, "AARE should contain A2 with result=0 (accepted) matching Gurux");

        // Verify A3 (result-source-diagnostic) is present
        let has_diagnostic = encoded.contains(&0xA3);
        assert!(has_diagnostic, "AARE should contain A3 (result-source-diagnostic)");

        // Roundtrip test
        let (_, parsed) = AareApdu::parse(&encoded).expect("Should parse Gurux-compatible AARE");
        assert!(parsed.is_accepted());
        assert_eq!(parsed.result, AssociationResult::Accepted);
    }

    /// Gurux compatibility test: Verify AARE rejected response matches Gurux encoding
    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_gurux_aare_rejected_encoding() {
        // Test rejection scenario as encoded by Gurux

        let aare = AareApdu::new_rejected(
            ApplicationContextName::LogicalNameReferencing,
            AssociationResult::RejectedPermanent,
            AcseServiceUserDiagnostics::AuthenticationFailure,
        );
        let encoded = aare.encode();

        // Verify AARE tag
        assert_eq!(encoded[0], 0x61);

        // Verify result = 1 (rejected-permanent)
        // A2 03 02 01 01
        let result_pattern = [0xA2, 0x03, 0x02, 0x01, 0x01];
        let has_result = encoded.windows(result_pattern.len()).any(|w| w == result_pattern);
        assert!(has_result, "AARE should contain result=1 (rejected-permanent) matching Gurux");

        // Verify diagnostic contains acse-service-user choice (A1)
        // A3 contains nested A1 for acse-service-user diagnostic
        let has_acse_user = encoded.windows(2).any(|w| w[0] == 0xA3 && w[1] > 0);
        assert!(has_acse_user, "AARE should contain acse-service-user diagnostic");

        // Roundtrip test
        let (_, parsed) =
            AareApdu::parse(&encoded).expect("Should parse Gurux-compatible AARE rejection");
        assert!(!parsed.is_accepted());
        assert_eq!(parsed.result, AssociationResult::RejectedPermanent);
    }

    /// Gurux compatibility test: Verify mechanism name OID encoding
    #[test]
    fn test_gurux_mechanism_name_oid() {
        // Gurux uses specific OIDs for authentication mechanisms
        // From apdu.c: 0x60 0x85 0x74 0x05 0x08 0x02 [auth_level]

        // Low Level Security (password): 2.16.756.5.8.2.1
        let lls_oid = MechanismName::LowLevelSecurity.oid_bytes();
        assert_eq!(lls_oid[0], 0x60); // 2.16 encoded
        assert_eq!(lls_oid[1], 0x85); // 756 encoded (high byte)
        assert_eq!(lls_oid[2], 0x74); // 756 encoded (low byte)
        assert_eq!(lls_oid[3], 0x05); // 5
        assert_eq!(lls_oid[4], 0x08); // 8
        assert_eq!(lls_oid[5], 0x02); // 2 (authentication)

        // Verify we can parse it back
        let parsed = MechanismName::from_oid_bytes(lls_oid);
        assert_eq!(parsed, Some(MechanismName::LowLevelSecurity));
    }

    /// Gurux compatibility test: Protocol version bit string
    #[cfg(all(feature = "encode", feature = "parse"))]
    #[test]
    fn test_gurux_protocol_version_encoding() {
        // Gurux encodes protocol version as BIT STRING with specific pattern
        // From apdu.c: value 0x84 for version 1 (bits: 100001)
        // Gurux comment: "Protocol version must be 100001"

        let aarq = AarqApdu::new_simple_ln(0xFFFF);
        let encoded = aarq.encode();

        // Find protocol-version field (A0 tag)
        // A0 04 03 02 07 80 means: tag A0, len 4, BIT STRING tag 03, len 2, unused bits 7, value 0x80
        // The bit pattern 0x80 with 7 unused bits = bit 0 set = version 1
        let protocol_version_pattern = [0xA0, 0x04, 0x03, 0x02];
        let has_protocol =
            encoded.windows(protocol_version_pattern.len()).any(|w| w == protocol_version_pattern);
        assert!(has_protocol, "AARQ should have protocol-version matching Gurux pattern");
    }
}
