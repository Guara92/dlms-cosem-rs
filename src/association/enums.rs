//! Enumerations for AARQ/AARE APDUs
//!
//! Reference: DLMS Green Book Ed. 12, Section 11

use core::fmt;

/// Association result returned in AARE
///
/// Reference: Green Book Table 138
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AssociationResult {
    /// Association accepted
    Accepted = 0,
    /// Association rejected permanently
    RejectedPermanent = 1,
    /// Association rejected transiently
    RejectedTransient = 2,
}

impl AssociationResult {
    /// Create from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Accepted),
            1 => Some(Self::RejectedPermanent),
            2 => Some(Self::RejectedTransient),
            _ => None,
        }
    }

    /// Convert to u8 value
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for AssociationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accepted => write!(f, "Accepted"),
            Self::RejectedPermanent => write!(f, "Rejected (Permanent)"),
            Self::RejectedTransient => write!(f, "Rejected (Transient)"),
        }
    }
}

/// ACSE service user diagnostics - reason for association rejection
///
/// Reference: Green Book Table 138
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AcseServiceUserDiagnostics {
    /// No diagnostic information
    Null = 0,
    /// No reason given
    NoReasonGiven = 1,
    /// Application context name not supported
    ApplicationContextNameNotSupported = 2,
    /// Authentication mechanism name not recognized
    AuthenticationMechanismNameNotRecognised = 11,
    /// Authentication mechanism name required
    AuthenticationMechanismNameRequired = 12,
    /// Authentication failure
    AuthenticationFailure = 13,
    /// Authentication required
    AuthenticationRequired = 14,
}

impl AcseServiceUserDiagnostics {
    /// Create from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Null),
            1 => Some(Self::NoReasonGiven),
            2 => Some(Self::ApplicationContextNameNotSupported),
            11 => Some(Self::AuthenticationMechanismNameNotRecognised),
            12 => Some(Self::AuthenticationMechanismNameRequired),
            13 => Some(Self::AuthenticationFailure),
            14 => Some(Self::AuthenticationRequired),
            _ => None,
        }
    }

    /// Convert to u8 value
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for AcseServiceUserDiagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "Null"),
            Self::NoReasonGiven => write!(f, "No reason given"),
            Self::ApplicationContextNameNotSupported => {
                write!(f, "Application context name not supported")
            }
            Self::AuthenticationMechanismNameNotRecognised => {
                write!(f, "Authentication mechanism name not recognised")
            }
            Self::AuthenticationMechanismNameRequired => {
                write!(f, "Authentication mechanism name required")
            }
            Self::AuthenticationFailure => write!(f, "Authentication failure"),
            Self::AuthenticationRequired => write!(f, "Authentication required"),
        }
    }
}

/// Application context name - identifies the type of association
///
/// These are encoded as ASN.1 OBJECT IDENTIFIERs
///
/// Reference: Green Book Section 9.3.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationContextName {
    /// Logical Name referencing without ciphering
    /// OID: 2.16.756.5.8.1.1
    LogicalNameReferencing,
    /// Short Name referencing without ciphering
    /// OID: 2.16.756.5.8.1.2
    ShortNameReferencing,
    /// Logical Name referencing with ciphering
    /// OID: 2.16.756.5.8.1.3
    LogicalNameReferencingWithCiphering,
    /// Short Name referencing with ciphering
    /// OID: 2.16.756.5.8.1.4
    ShortNameReferencingWithCiphering,
}

impl ApplicationContextName {
    /// Get the OID bytes for this application context
    ///
    /// Returns the BER-encoded OBJECT IDENTIFIER
    pub fn oid_bytes(&self) -> &'static [u8] {
        match self {
            // 2.16.756.5.8.1.1
            Self::LogicalNameReferencing => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01],
            // 2.16.756.5.8.1.2
            Self::ShortNameReferencing => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x02],
            // 2.16.756.5.8.1.3
            Self::LogicalNameReferencingWithCiphering => {
                &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x03]
            }
            // 2.16.756.5.8.1.4
            Self::ShortNameReferencingWithCiphering => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x04],
        }
    }

    /// Parse from OID bytes
    #[cfg(feature = "parse")]
    pub fn from_oid_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01] => Some(Self::LogicalNameReferencing),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x02] => Some(Self::ShortNameReferencing),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x03] => {
                Some(Self::LogicalNameReferencingWithCiphering)
            }
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x04] => {
                Some(Self::ShortNameReferencingWithCiphering)
            }
            _ => None,
        }
    }

    /// Check if this context uses ciphering
    pub const fn uses_ciphering(&self) -> bool {
        matches!(
            self,
            Self::LogicalNameReferencingWithCiphering | Self::ShortNameReferencingWithCiphering
        )
    }

    /// Check if this context uses logical name referencing
    pub const fn uses_logical_name(&self) -> bool {
        matches!(self, Self::LogicalNameReferencing | Self::LogicalNameReferencingWithCiphering)
    }
}

impl fmt::Display for ApplicationContextName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LogicalNameReferencing => write!(f, "LN"),
            Self::ShortNameReferencing => write!(f, "SN"),
            Self::LogicalNameReferencingWithCiphering => write!(f, "LN with ciphering"),
            Self::ShortNameReferencingWithCiphering => write!(f, "SN with ciphering"),
        }
    }
}

/// Authentication mechanism name
///
/// These are encoded as ASN.1 OBJECT IDENTIFIERs
///
/// Reference: Green Book Section 9.3.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MechanismName {
    /// No authentication (lowest level security)
    /// OID: 2.16.756.5.8.2.0
    LowestLevelSecurity,
    /// Password authentication (low level security)
    /// OID: 2.16.756.5.8.2.1
    LowLevelSecurity,
    /// HLS with manufacturer-specific algorithm
    /// OID: 2.16.756.5.8.2.2
    HighLevelSecurity,
    /// HLS with MD5
    /// OID: 2.16.756.5.8.2.3
    HighLevelSecurityMd5,
    /// HLS with SHA-1
    /// OID: 2.16.756.5.8.2.4
    HighLevelSecuritySha1,
    /// HLS with AES-GCM (GMAC)
    /// OID: 2.16.756.5.8.2.5
    HighLevelSecurityGmac,
    /// HLS with SHA-256
    /// OID: 2.16.756.5.8.2.6
    HighLevelSecuritySha256,
    /// HLS with ECDSA
    /// OID: 2.16.756.5.8.2.7
    HighLevelSecurityEcdsa,
}

impl MechanismName {
    /// Get the OID bytes for this mechanism
    ///
    /// Returns the BER-encoded OBJECT IDENTIFIER
    pub fn oid_bytes(&self) -> &'static [u8] {
        match self {
            // 2.16.756.5.8.2.0
            Self::LowestLevelSecurity => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x00],
            // 2.16.756.5.8.2.1
            Self::LowLevelSecurity => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x01],
            // 2.16.756.5.8.2.2
            Self::HighLevelSecurity => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x02],
            // 2.16.756.5.8.2.3
            Self::HighLevelSecurityMd5 => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x03],
            // 2.16.756.5.8.2.4
            Self::HighLevelSecuritySha1 => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x04],
            // 2.16.756.5.8.2.5
            Self::HighLevelSecurityGmac => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x05],
            // 2.16.756.5.8.2.6
            Self::HighLevelSecuritySha256 => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x06],
            // 2.16.756.5.8.2.7
            Self::HighLevelSecurityEcdsa => &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x07],
        }
    }

    /// Parse from OID bytes
    #[cfg(feature = "parse")]
    pub fn from_oid_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x00] => Some(Self::LowestLevelSecurity),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x01] => Some(Self::LowLevelSecurity),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x02] => Some(Self::HighLevelSecurity),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x03] => Some(Self::HighLevelSecurityMd5),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x04] => Some(Self::HighLevelSecuritySha1),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x05] => Some(Self::HighLevelSecurityGmac),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x06] => Some(Self::HighLevelSecuritySha256),
            [0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x07] => Some(Self::HighLevelSecurityEcdsa),
            _ => None,
        }
    }
}

impl fmt::Display for MechanismName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LowestLevelSecurity => write!(f, "No authentication"),
            Self::LowLevelSecurity => write!(f, "Low level (password)"),
            Self::HighLevelSecurity => write!(f, "High level"),
            Self::HighLevelSecurityMd5 => write!(f, "HLS-MD5"),
            Self::HighLevelSecuritySha1 => write!(f, "HLS-SHA1"),
            Self::HighLevelSecurityGmac => write!(f, "HLS-GMAC"),
            Self::HighLevelSecuritySha256 => write!(f, "HLS-SHA256"),
            Self::HighLevelSecurityEcdsa => write!(f, "HLS-ECDSA"),
        }
    }
}

/// Authentication value for AARQ/AARE
///
/// Reference: Green Book Section 11
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationValue {
    /// Character string (password) - for low level security
    CharString(alloc::vec::Vec<u8>),
    /// Bit string (challenge/response) - for high level security
    BitString(alloc::vec::Vec<u8>),
}

impl AuthenticationValue {
    /// Create a character string authentication value
    pub fn char_string(data: alloc::vec::Vec<u8>) -> Self {
        Self::CharString(data)
    }

    /// Create a bit string authentication value
    pub fn bit_string(data: alloc::vec::Vec<u8>) -> Self {
        Self::BitString(data)
    }

    /// Get the inner bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::CharString(bytes) | Self::BitString(bytes) => bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    #[test]
    fn test_association_result() {
        assert_eq!(AssociationResult::Accepted.as_u8(), 0);
        assert_eq!(AssociationResult::RejectedPermanent.as_u8(), 1);
        assert_eq!(AssociationResult::RejectedTransient.as_u8(), 2);

        assert_eq!(AssociationResult::from_u8(0), Some(AssociationResult::Accepted));
        assert_eq!(AssociationResult::from_u8(1), Some(AssociationResult::RejectedPermanent));
        assert_eq!(AssociationResult::from_u8(2), Some(AssociationResult::RejectedTransient));
        assert_eq!(AssociationResult::from_u8(3), None);
    }

    #[test]
    fn test_application_context_name() {
        // Check OID encoding
        let ln = ApplicationContextName::LogicalNameReferencing;
        assert_eq!(ln.oid_bytes(), &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01]);
        assert!(!ln.uses_ciphering());
        assert!(ln.uses_logical_name());

        let ln_cipher = ApplicationContextName::LogicalNameReferencingWithCiphering;
        assert_eq!(ln_cipher.oid_bytes(), &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x03]);
        assert!(ln_cipher.uses_ciphering());
        assert!(ln_cipher.uses_logical_name());

        let sn = ApplicationContextName::ShortNameReferencing;
        assert_eq!(sn.oid_bytes(), &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x02]);
        assert!(!sn.uses_ciphering());
        assert!(!sn.uses_logical_name());
    }

    #[test]
    #[cfg(feature = "parse")]
    fn test_application_context_name_parse() {
        assert_eq!(
            ApplicationContextName::from_oid_bytes(&[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01]),
            Some(ApplicationContextName::LogicalNameReferencing)
        );
        assert_eq!(
            ApplicationContextName::from_oid_bytes(&[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x03]),
            Some(ApplicationContextName::LogicalNameReferencingWithCiphering)
        );
        assert_eq!(ApplicationContextName::from_oid_bytes(&[0x00, 0x00, 0x00]), None);
    }

    #[test]
    fn test_mechanism_name() {
        assert_eq!(
            MechanismName::LowestLevelSecurity.oid_bytes(),
            &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x00]
        );
        assert_eq!(
            MechanismName::LowLevelSecurity.oid_bytes(),
            &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x01]
        );
        assert_eq!(
            MechanismName::HighLevelSecurityGmac.oid_bytes(),
            &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x05]
        );
    }

    #[test]
    #[cfg(feature = "parse")]
    fn test_mechanism_name_parse() {
        assert_eq!(
            MechanismName::from_oid_bytes(&[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x00]),
            Some(MechanismName::LowestLevelSecurity)
        );
        assert_eq!(
            MechanismName::from_oid_bytes(&[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x05]),
            Some(MechanismName::HighLevelSecurityGmac)
        );
        assert_eq!(MechanismName::from_oid_bytes(&[0x00, 0x00]), None);
    }

    #[test]
    fn test_mechanism_name_all_variants() {
        // Test all 8 mechanism name variants per Green Book Table 138
        let mechanisms = vec![
            (MechanismName::LowestLevelSecurity, &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x00][..]),
            (MechanismName::LowLevelSecurity, &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x01][..]),
            (MechanismName::HighLevelSecurity, &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x02][..]),
            (MechanismName::HighLevelSecurityMd5, &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x03][..]),
            (MechanismName::HighLevelSecuritySha1, &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x04][..]),
            (MechanismName::HighLevelSecurityGmac, &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x05][..]),
            (
                MechanismName::HighLevelSecuritySha256,
                &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x06][..],
            ),
            (
                MechanismName::HighLevelSecurityEcdsa,
                &[0x60, 0x85, 0x74, 0x05, 0x08, 0x02, 0x07][..],
            ),
        ];

        for (mechanism, oid) in mechanisms {
            assert_eq!(mechanism.oid_bytes(), oid);
            assert_eq!(MechanismName::from_oid_bytes(oid), Some(mechanism));
        }
    }

    #[test]
    fn test_application_context_all_variants() {
        // Test all 4 application context variants per Green Book Table 137
        let contexts = vec![
            (
                ApplicationContextName::LogicalNameReferencing,
                &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x01][..],
            ),
            (
                ApplicationContextName::ShortNameReferencing,
                &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x02][..],
            ),
            (
                ApplicationContextName::LogicalNameReferencingWithCiphering,
                &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x03][..],
            ),
            (
                ApplicationContextName::ShortNameReferencingWithCiphering,
                &[0x60, 0x85, 0x74, 0x05, 0x08, 0x01, 0x04][..],
            ),
        ];

        for (context, oid) in contexts {
            assert_eq!(context.oid_bytes(), oid);
            assert_eq!(ApplicationContextName::from_oid_bytes(oid), Some(context));

            // Test uses_ciphering() method
            let should_use_ciphering = matches!(
                context,
                ApplicationContextName::LogicalNameReferencingWithCiphering
                    | ApplicationContextName::ShortNameReferencingWithCiphering
            );
            assert_eq!(context.uses_ciphering(), should_use_ciphering);

            // Test uses_logical_name() method
            let should_use_ln = matches!(
                context,
                ApplicationContextName::LogicalNameReferencing
                    | ApplicationContextName::LogicalNameReferencingWithCiphering
            );
            assert_eq!(context.uses_logical_name(), should_use_ln);
        }
    }

    #[test]
    fn test_association_result_all_variants() {
        // Test all 3 result variants per Green Book Table 139
        assert_eq!(AssociationResult::Accepted.as_u8(), 0);
        assert_eq!(AssociationResult::RejectedPermanent.as_u8(), 1);
        assert_eq!(AssociationResult::RejectedTransient.as_u8(), 2);

        assert_eq!(AssociationResult::from_u8(0), Some(AssociationResult::Accepted));
        assert_eq!(AssociationResult::from_u8(1), Some(AssociationResult::RejectedPermanent));
        assert_eq!(AssociationResult::from_u8(2), Some(AssociationResult::RejectedTransient));
        assert_eq!(AssociationResult::from_u8(3), None);
    }

    #[test]
    fn test_diagnostics_all_variants() {
        // Test all 7 diagnostic variants implemented (subset of Green Book Table 139)
        let diagnostics = vec![
            (AcseServiceUserDiagnostics::Null, 0),
            (AcseServiceUserDiagnostics::NoReasonGiven, 1),
            (AcseServiceUserDiagnostics::ApplicationContextNameNotSupported, 2),
            (AcseServiceUserDiagnostics::AuthenticationMechanismNameNotRecognised, 11),
            (AcseServiceUserDiagnostics::AuthenticationMechanismNameRequired, 12),
            (AcseServiceUserDiagnostics::AuthenticationFailure, 13),
            (AcseServiceUserDiagnostics::AuthenticationRequired, 14),
        ];

        for (diag, value) in diagnostics {
            assert_eq!(diag.as_u8(), value);
            assert_eq!(AcseServiceUserDiagnostics::from_u8(value), Some(diag));
        }

        // Test invalid values (including unused diagnostic codes 3-10)
        assert_eq!(AcseServiceUserDiagnostics::from_u8(3), None);
        assert_eq!(AcseServiceUserDiagnostics::from_u8(10), None);
        assert_eq!(AcseServiceUserDiagnostics::from_u8(99), None);
    }

    #[test]
    fn test_authentication_value() {
        let password = AuthenticationValue::char_string(alloc::vec![0x01, 0x02, 0x03]);
        assert_eq!(password.as_bytes(), &[0x01, 0x02, 0x03]);

        let challenge = AuthenticationValue::bit_string(alloc::vec![0xAA, 0xBB]);
        assert_eq!(challenge.as_bytes(), &[0xAA, 0xBB]);
    }

    #[test]
    fn test_diagnostics() {
        assert_eq!(AcseServiceUserDiagnostics::Null.as_u8(), 0);
        assert_eq!(AcseServiceUserDiagnostics::AuthenticationFailure.as_u8(), 13);

        assert_eq!(
            AcseServiceUserDiagnostics::from_u8(13),
            Some(AcseServiceUserDiagnostics::AuthenticationFailure)
        );
    }
}
