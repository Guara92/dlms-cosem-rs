use alloc::vec::Vec;

use aes::{Aes128, Aes256};
use aes_gcm::aead::{AeadInPlace, KeyInit};
use aes_gcm::{Aes128Gcm, Aes256Gcm};
use cipher::Key;
#[cfg(feature = "parse")]
use nom::{
    IResult, Parser,
    bytes::streaming::tag,
    combinator::cond,
    multi::{count, fill},
    number::streaming::{be_u16, be_u32, u8},
};

use crate::{SecurityControl, SecuritySuite};

#[cfg(feature = "encode")]
use crate::ByteBuffer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralGloCiphering {
    pub(crate) system_title: [u8; 8],
    pub(crate) security_control: SecurityControl,
    pub(crate) invocation_counter: Option<u32>,
    pub(crate) payload: Vec<u8>,
}

impl GeneralGloCiphering {
    /// Create a new GeneralGloCiphering structure
    pub fn new(
        system_title: [u8; 8],
        security_control: SecurityControl,
        invocation_counter: Option<u32>,
        payload: Vec<u8>,
    ) -> Self {
        Self { system_title, security_control, invocation_counter, payload }
    }

    /// Encrypt a payload using AES-128-GCM
    ///
    /// # Arguments
    /// * `payload` - The plaintext data to encrypt
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title (part of IV)
    /// * `invocation_counter` - 4-byte invocation counter (part of IV)
    /// * `security_control` - Security control flags
    ///
    /// # Returns
    /// A new `GeneralGloCiphering` with encrypted payload
    pub fn encrypt(
        payload: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        mut security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let cipher = Aes128Gcm::new(key);

        // Build IV: system_title (8 bytes) + invocation_counter (4 bytes) = 12 bytes
        let mut iv = [0u8; 12];
        iv[0..8].copy_from_slice(&system_title);
        iv[8..12].copy_from_slice(&invocation_counter.to_be_bytes());

        // Encrypt the payload in-place
        let mut encrypted_payload = payload.to_vec();
        cipher.encrypt_in_place_detached(&iv.into(), &[], &mut encrypted_payload)?;

        // Set encryption flag and Suite V1
        security_control.set_encryption(true);
        security_control.set_suite(SecuritySuite::V1);

        Ok(Self {
            system_title,
            security_control,
            invocation_counter: Some(invocation_counter),
            payload: encrypted_payload,
        })
    }

    /// Encrypt a payload with authentication (authenticated encryption)
    ///
    /// This performs AES-GCM encryption with authentication tag.
    ///
    /// # Arguments
    /// * `payload` - The plaintext data to encrypt
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title (part of IV)
    /// * `invocation_counter` - 4-byte invocation counter (part of IV)
    /// * `security_control` - Security control flags
    ///
    /// # Returns
    /// A new `GeneralGloCiphering` with encrypted and authenticated payload
    pub fn encrypt_authenticated(
        payload: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        mut security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let cipher = Aes128Gcm::new(key);

        // Build IV: system_title (8 bytes) + invocation_counter (4 bytes) = 12 bytes
        let mut iv = [0u8; 12];
        iv[0..8].copy_from_slice(&system_title);
        iv[8..12].copy_from_slice(&invocation_counter.to_be_bytes());

        // Encrypt the payload in-place
        let mut encrypted_payload = payload.to_vec();
        cipher.encrypt_in_place_detached(&iv.into(), &[], &mut encrypted_payload)?;

        // Set both encryption and authentication flags, and Suite V1
        security_control.set_encryption(true);
        security_control.set_authentication(true);
        security_control.set_suite(SecuritySuite::V1);

        Ok(Self {
            system_title,
            security_control,
            invocation_counter: Some(invocation_counter),
            payload: encrypted_payload,
        })
    }

    /// Encrypt a payload using AES-256-GCM (Suite V2)
    ///
    /// # Arguments
    /// * `payload` - The plaintext data to encrypt
    /// * `key` - The AES-256 encryption key (32 bytes)
    /// * `system_title` - 8-byte system title (part of IV)
    /// * `invocation_counter` - 4-byte invocation counter (part of IV)
    /// * `security_control` - Security control flags
    ///
    /// # Returns
    /// A new `GeneralGloCiphering` with encrypted payload
    ///
    /// # Note
    /// This function automatically sets the suite to V2 and enables encryption flag.
    pub fn encrypt_v2(
        payload: &[u8],
        key: &Key<Aes256>,
        system_title: [u8; 8],
        invocation_counter: u32,
        mut security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let cipher = Aes256Gcm::new(key);

        // Build IV: system_title (8 bytes) + invocation_counter (4 bytes) = 12 bytes
        let mut iv = [0u8; 12];
        iv[0..8].copy_from_slice(&system_title);
        iv[8..12].copy_from_slice(&invocation_counter.to_be_bytes());

        // Encrypt the payload in-place
        let mut encrypted_payload = payload.to_vec();
        cipher.encrypt_in_place_detached(&iv.into(), &[], &mut encrypted_payload)?;

        // Set encryption flag and Suite V2
        security_control.set_encryption(true);
        security_control.set_suite(SecuritySuite::V2);

        Ok(Self {
            system_title,
            security_control,
            invocation_counter: Some(invocation_counter),
            payload: encrypted_payload,
        })
    }

    /// Encrypt a payload with authentication using AES-256-GCM (Suite V2)
    ///
    /// This performs AES-GCM encryption with authentication tag.
    ///
    /// # Arguments
    /// * `payload` - The plaintext data to encrypt
    /// * `key` - The AES-256 encryption key (32 bytes)
    /// * `system_title` - 8-byte system title (part of IV)
    /// * `invocation_counter` - 4-byte invocation counter (part of IV)
    /// * `security_control` - Security control flags
    ///
    /// # Returns
    /// A new `GeneralGloCiphering` with encrypted and authenticated payload
    ///
    /// # Note
    /// This function automatically sets the suite to V2 and enables both encryption and authentication flags.
    pub fn encrypt_authenticated_v2(
        payload: &[u8],
        key: &Key<Aes256>,
        system_title: [u8; 8],
        invocation_counter: u32,
        mut security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let cipher = Aes256Gcm::new(key);

        // Build IV: system_title (8 bytes) + invocation_counter (4 bytes) = 12 bytes
        let mut iv = [0u8; 12];
        iv[0..8].copy_from_slice(&system_title);
        iv[8..12].copy_from_slice(&invocation_counter.to_be_bytes());

        // Encrypt the payload in-place
        let mut encrypted_payload = payload.to_vec();
        cipher.encrypt_in_place_detached(&iv.into(), &[], &mut encrypted_payload)?;

        // Set both encryption and authentication flags, and Suite V2
        security_control.set_encryption(true);
        security_control.set_authentication(true);
        security_control.set_suite(SecuritySuite::V2);

        Ok(Self {
            system_title,
            security_control,
            invocation_counter: Some(invocation_counter),
            payload: encrypted_payload,
        })
    }

    /// Encode the GeneralGloCiphering structure to bytes
    ///
    /// # Format
    /// - Tag [8] (system title length)
    /// - System title (8 bytes)
    /// - Length of remaining data
    /// - Security control (1 byte)
    /// - Invocation counter (4 bytes, if auth or encryption is set)
    /// - Encrypted payload
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Tag: length of system title (always 8)
        buffer.push_u8(0x08);

        // System title (8 bytes)
        buffer.push_bytes(&self.system_title);

        // Calculate payload length: security_control (1) + invocation_counter (4 if present) + payload
        let has_invocation_counter =
            self.security_control.authentication() || self.security_control.encryption();

        let ic_len = if has_invocation_counter { 4 } else { 0 };
        let total_len = 1 + ic_len + self.payload.len();

        // Encode length
        if total_len <= 127 {
            buffer.push_u8(total_len as u8);
        } else {
            buffer.push_u8(0x82);
            buffer.push_u16(total_len as u16);
        }

        // Security control
        buffer.push_bytes(&[self.security_control.encode()]);

        // Invocation counter (if present)
        if let Some(ic) = self.invocation_counter {
            buffer.push_u32(ic);
        }

        // Encrypted payload
        buffer.push_bytes(&self.payload);

        buffer
    }

    pub fn decrypt(mut self, key: &Key<Aes128>) -> Result<Vec<u8>, aes_gcm::Error> {
        if self.security_control.encryption() {
            let cipher = Aes128Gcm::new(key);

            let mut iv = [0u8; 12];
            iv[0..8].copy_from_slice(&self.system_title);
            iv[8..].copy_from_slice(&self.invocation_counter.unwrap().to_be_bytes());

            cipher.encrypt_in_place_detached(&iv.into(), &[], &mut self.payload)?;
            self.security_control.set_encryption(false);
        }

        Ok(self.payload)
    }

    /// Decrypt a payload using AES-256-GCM (Suite V2)
    ///
    /// # Arguments
    /// * `key` - The AES-256 decryption key (32 bytes)
    ///
    /// # Returns
    /// Decrypted plaintext payload
    ///
    /// # Note
    /// This consumes self and returns the decrypted payload.
    pub fn decrypt_v2(mut self, key: &Key<Aes256>) -> Result<Vec<u8>, aes_gcm::Error> {
        if self.security_control.encryption() {
            let cipher = Aes256Gcm::new(key);

            let mut iv = [0u8; 12];
            iv[0..8].copy_from_slice(&self.system_title);
            iv[8..].copy_from_slice(&self.invocation_counter.unwrap().to_be_bytes());

            cipher.encrypt_in_place_detached(&iv.into(), &[], &mut self.payload)?;
            self.security_control.set_encryption(false);
        }

        Ok(self.payload)
    }

    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, _) = tag(&[8u8][..]).parse(input)?;
        let mut system_title = [0u8; 8];
        let (input, _) = fill(u8, &mut system_title).parse(input)?;

        let (input, mut payload_len) = match u8(input)? {
            (input, 0x82) => {
                let (input, len) = be_u16(input)?;
                (input, len as usize)
            }
            (input, len) => (input, len as usize),
        };
        payload_len -= 5;

        // Green Book 9.2.7.2.4.1
        let (input, security_control) = SecurityControl::parse(input)?;

        let (input, invocation_counter) =
            cond(security_control.authentication() || security_control.encryption(), be_u32)
                .parse(input)?;

        let (input, payload) = count(u8, payload_len).parse(input)?;

        Ok((input, Self { system_title, security_control, invocation_counter, payload }))
    }
}

#[cfg(all(test, feature = "parse"))]
mod parse_tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        // Simple GeneralGloCiphering with no encryption/authentication
        // Tag [8] + SystemTitle [8 bytes] + Length + SecurityControl + Payload
        #[rustfmt::skip]
        let input = [
            0x08,                                           // Tag (length = 8 bytes system title)
            0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9, // System Title (8 bytes)
            0x05,                                           // Payload length (5 bytes total including SC and IC)
            0x00,                                           // Security Control (no auth/enc)
            // No invocation counter (auth/enc not set)
            // Payload would be empty since length - 5 = 0
        ];

        let (remaining, ggc) = GeneralGloCiphering::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(ggc.system_title, [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9]);
        assert!(!ggc.security_control.authentication());
        assert!(!ggc.security_control.encryption());
        assert_eq!(ggc.invocation_counter, None);
        assert_eq!(ggc.payload, vec![]);
    }

    #[test]
    fn test_parse_with_authentication() {
        // With authentication flag set, invocation counter is included
        #[rustfmt::skip]
        let input = [
            0x08,                                           // Tag
            0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9, // System Title
            0x09,                                           // Payload length (9 = 1 SC + 4 IC + 4 payload)
            0x10,                                           // Security Control (authentication bit set)
            0x00, 0x00, 0x00, 0x01,                        // Invocation Counter
            0xAA, 0xBB, 0xCC, 0xDD,                        // Payload (4 bytes)
        ];

        let (remaining, ggc) = GeneralGloCiphering::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert!(ggc.security_control.authentication());
        assert!(!ggc.security_control.encryption());
        assert_eq!(ggc.invocation_counter, Some(1));
        assert_eq!(ggc.payload, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn test_parse_with_encryption() {
        // With encryption flag set, invocation counter is included
        #[rustfmt::skip]
        let input = [
            0x08,                                           // Tag
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, // System Title
            0x0A,                                           // Payload length (10 = 1 SC + 4 IC + 5 payload)
            0x20,                                           // Security Control (encryption bit set)
            0x00, 0x00, 0x12, 0x34,                        // Invocation Counter (0x1234)
            0x01, 0x02, 0x03, 0x04, 0x05,                  // Payload (5 bytes)
        ];

        let (remaining, ggc) = GeneralGloCiphering::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(ggc.system_title, [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]);
        assert!(!ggc.security_control.authentication());
        assert!(ggc.security_control.encryption());
        assert_eq!(ggc.invocation_counter, Some(0x1234));
        assert_eq!(ggc.payload, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_parse_with_auth_and_encryption() {
        // Both authentication and encryption flags set
        #[rustfmt::skip]
        let input = [
            0x08,                                           // Tag
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22, // System Title
            0x08,                                           // Payload length (8 = 1 SC + 4 IC + 3 payload)
            0x30,                                           // Security Control (0x30 = auth + enc)
            0xFF, 0xFF, 0xFF, 0xFF,                        // Invocation Counter (max value)
            0xDE, 0xAD, 0xBE,                              // Payload (3 bytes)
        ];

        let (remaining, ggc) = GeneralGloCiphering::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert!(ggc.security_control.authentication());
        assert!(ggc.security_control.encryption());
        assert_eq!(ggc.invocation_counter, Some(0xFFFFFFFF));
        assert_eq!(ggc.payload, vec![0xDE, 0xAD, 0xBE]);
    }

    #[test]
    fn test_parse_with_extended_length() {
        // Test with 0x82 extended length encoding (for payloads > 127 bytes)
        #[rustfmt::skip]
        let input = [
            0x08,                                           // Tag
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // System Title
            0x82, 0x00, 0x0C,                              // Extended length: 0x000C = 12 bytes
            0x30,                                           // Security Control
            0x00, 0x00, 0x00, 0x01,                        // Invocation Counter
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11,      // Payload (7 bytes = 12 - 5)
        ];

        let (remaining, ggc) = GeneralGloCiphering::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(ggc.invocation_counter, Some(1));
        assert_eq!(ggc.payload.len(), 7);
        assert_eq!(ggc.payload, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11]);
    }

    #[test]
    fn test_parse_with_remaining_input() {
        // Test that parser correctly handles remaining input
        #[rustfmt::skip]
        let input = [
            0x08,                                           // Tag
            0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9, // System Title
            0x06,                                           // Payload length (6 = 1 SC + 4 IC + 1 payload)
            0x10,                                           // Security Control
            0x00, 0x00, 0x00, 0x01,                        // Invocation Counter
            0xAA,                                           // Payload (1 byte)
            0xFF, 0xFF,                                     // Extra bytes that should remain
        ];

        let (remaining, ggc) = GeneralGloCiphering::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF, 0xFF]);
        assert_eq!(ggc.payload, vec![0xAA]);
    }

    #[test]
    fn test_parse_empty_payload() {
        // Test with minimal payload (only security control and invocation counter)
        #[rustfmt::skip]
        let input = [
            0x08,                                           // Tag
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // System Title
            0x05,                                           // Payload length (5 = 1 SC + 4 IC + 0 payload)
            0x10,                                           // Security Control (auth)
            0x00, 0x00, 0x00, 0x00,                        // Invocation Counter
        ];

        let (remaining, ggc) = GeneralGloCiphering::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(ggc.payload, vec![]);
    }

    #[test]
    fn test_clone_and_equality() {
        #[rustfmt::skip]
        let input = [
            0x08,
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x07,
            0x30,
            0x00, 0x00, 0x00, 0x42,
            0xAA, 0xBB,
        ];

        let (_, ggc1) = GeneralGloCiphering::parse(&input).unwrap();
        let ggc2 = ggc1.clone();

        assert_eq!(ggc1, ggc2);
        assert_eq!(ggc1.system_title, ggc2.system_title);
        assert_eq!(ggc1.payload, ggc2.payload);
    }

    #[test]
    fn test_debug_format() {
        #[rustfmt::skip]
        let input = [
            0x08,
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x06,
            0x10,
            0x00, 0x00, 0x00, 0x01,
            0xAA,
        ];

        let (_, ggc) = GeneralGloCiphering::parse(&input).unwrap();
        let debug_str = format!("{:?}", ggc);

        assert!(debug_str.contains("GeneralGloCiphering"));
        assert!(debug_str.contains("system_title"));
        assert!(debug_str.contains("security_control"));
        assert!(debug_str.contains("invocation_counter"));
        assert!(debug_str.contains("payload"));
    }
}

#[cfg(all(test, feature = "encode"))]
mod encode_tests {
    use super::*;

    #[test]
    fn test_encode_basic() {
        // Create a basic GeneralGloCiphering with no encryption/authentication
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let security_control = SecurityControl::new(0x00);
        let payload = vec![];

        let ggc = GeneralGloCiphering::new(system_title, security_control, None, payload);

        let encoded = ggc.encode();

        #[rustfmt::skip]
        let expected = vec![
            0x08,                                           // Tag
            0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9, // System Title
            0x01,                                           // Length (1 byte for SC only)
            0x00,                                           // Security Control
        ];

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_with_authentication() {
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let mut security_control = SecurityControl::new(0x00);
        security_control.set_authentication(true);
        let payload = vec![0xAA, 0xBB, 0xCC, 0xDD];

        let ggc = GeneralGloCiphering::new(system_title, security_control, Some(1), payload);

        let encoded = ggc.encode();

        #[rustfmt::skip]
        let expected = vec![
            0x08,                                           // Tag
            0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9, // System Title
            0x09,                                           // Length (1 SC + 4 IC + 4 payload)
            0x10,                                           // Security Control (auth)
            0x00, 0x00, 0x00, 0x01,                        // Invocation Counter
            0xAA, 0xBB, 0xCC, 0xDD,                        // Payload
        ];

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_with_encryption() {
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let mut security_control = SecurityControl::new(0x00);
        security_control.set_encryption(true);
        let payload = vec![0x01, 0x02, 0x03, 0x04, 0x05];

        let ggc = GeneralGloCiphering::new(system_title, security_control, Some(0x1234), payload);

        let encoded = ggc.encode();

        #[rustfmt::skip]
        let expected = vec![
            0x08,                                           // Tag
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, // System Title
            0x0A,                                           // Length (1 SC + 4 IC + 5 payload)
            0x20,                                           // Security Control (encryption)
            0x00, 0x00, 0x12, 0x34,                        // Invocation Counter
            0x01, 0x02, 0x03, 0x04, 0x05,                  // Payload
        ];

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_with_extended_length() {
        let system_title = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut security_control = SecurityControl::new(0x00);
        security_control.set_authentication(true);
        security_control.set_encryption(true);

        // Create payload that results in total length > 127
        let payload = vec![0xAA; 200];

        let ggc =
            GeneralGloCiphering::new(system_title, security_control, Some(1), payload.clone());

        let encoded = ggc.encode();

        // Total length = 1 (SC) + 4 (IC) + 200 (payload) = 205
        assert_eq!(encoded[0], 0x08); // Tag
        assert_eq!(&encoded[1..9], &system_title); // System title
        assert_eq!(encoded[9], 0x82); // Extended length marker
        assert_eq!(encoded[10], 0x00); // Length high byte
        assert_eq!(encoded[11], 205); // Length low byte (205)
        assert_eq!(encoded[12], 0x30); // Security control (auth + enc)
        assert_eq!(&encoded[13..17], &[0x00, 0x00, 0x00, 0x01]); // IC
        assert_eq!(&encoded[17..], &payload[..]); // Payload
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let mut security_control = SecurityControl::new(0x00);
        security_control.set_authentication(true);
        let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];

        let original = GeneralGloCiphering::new(system_title, security_control, Some(42), payload);

        let encoded = original.encode();
        let (remaining, decoded) = GeneralGloCiphering::parse(&encoded).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(decoded, original);
    }
}

#[cfg(all(test, feature = "encode"))]
mod encryption_tests {
    use super::*;

    #[test]
    fn test_encrypt_basic() {
        let key = &[0u8; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"Hello, DLMS!";
        let security_control = SecurityControl::new(0x00);

        let result = GeneralGloCiphering::encrypt(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ggc = result.unwrap();

        // Check that encryption flag is set
        assert!(ggc.security_control.encryption());
        assert_eq!(ggc.system_title, system_title);
        assert_eq!(ggc.invocation_counter, Some(invocation_counter));

        // Encrypted payload should be different from plaintext
        assert_ne!(ggc.payload, plaintext);
        assert_eq!(ggc.payload.len(), plaintext.len());
    }

    #[test]
    fn test_encrypt_authenticated() {
        let key = &[0u8; 16].into();
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 42;
        let plaintext = b"Authenticated message";
        let security_control = SecurityControl::new(0x00);

        let result = GeneralGloCiphering::encrypt_authenticated(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ggc = result.unwrap();

        // Check that both flags are set
        assert!(ggc.security_control.encryption());
        assert!(ggc.security_control.authentication());
        assert_eq!(ggc.system_title, system_title);
        assert_eq!(ggc.invocation_counter, Some(invocation_counter));

        // Encrypted payload should be different from plaintext
        assert_ne!(ggc.payload, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key: &Key<Aes128> = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ]
        .into();
        let system_title = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22];
        let invocation_counter = 12345;
        let plaintext = b"Test message for encryption";
        let security_control = SecurityControl::new(0x00);

        // Encrypt
        let encrypted = GeneralGloCiphering::encrypt(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        // Decrypt
        let decrypted = encrypted.decrypt(key).unwrap();

        // Should match original plaintext
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_authenticated_decrypt_roundtrip() {
        let key: &Key<Aes128> = &[0xFF; 16].into();
        let system_title = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let invocation_counter = 999;
        let plaintext = b"Authenticated and encrypted";
        let security_control = SecurityControl::new(0x00);

        // Encrypt with authentication
        let encrypted = GeneralGloCiphering::encrypt_authenticated(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        // Decrypt
        let decrypted = encrypted.decrypt(key).unwrap();

        // Should match original plaintext
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty_payload() {
        let key = &[0u8; 16].into();
        let system_title = [0; 8];
        let invocation_counter = 0;
        let plaintext = b"";
        let security_control = SecurityControl::new(0x00);

        let result = GeneralGloCiphering::encrypt(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ggc = result.unwrap();
        assert_eq!(ggc.payload.len(), 0);
    }

    #[test]
    fn test_encrypt_different_keys_produce_different_ciphertext() {
        let key1 = [0u8; 16].into();
        let key2 = [0xFF; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"Same plaintext";
        let security_control = SecurityControl::new(0x00);

        let encrypted1 = GeneralGloCiphering::encrypt(
            plaintext,
            &key1,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let encrypted2 = GeneralGloCiphering::encrypt(
            plaintext,
            &key2,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        // Same plaintext with different keys should produce different ciphertext
        assert_ne!(encrypted1.payload, encrypted2.payload);
    }

    #[test]
    fn test_encrypt_different_iv_produce_different_ciphertext() {
        let key = &[0u8; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter1 = 1;
        let invocation_counter2 = 2;
        let plaintext = b"Same plaintext";
        let security_control = SecurityControl::new(0x00);

        let encrypted1 = GeneralGloCiphering::encrypt(
            plaintext,
            key,
            system_title,
            invocation_counter1,
            security_control,
        )
        .unwrap();

        let encrypted2 = GeneralGloCiphering::encrypt(
            plaintext,
            key,
            system_title,
            invocation_counter2,
            security_control,
        )
        .unwrap();

        // Same plaintext with different IV should produce different ciphertext
        assert_ne!(encrypted1.payload, encrypted2.payload);
    }

    // --- Suite V2 (AES-256-GCM) Tests ---

    #[test]
    fn test_encrypt_v2_basic() {
        let key = &[0u8; 32].into(); // 32-byte key for AES-256
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"Hello, DLMS Suite V2!";
        let security_control = SecurityControl::new(0x00);

        let result = GeneralGloCiphering::encrypt_v2(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ggc = result.unwrap();

        // Check that encryption flag is set and suite is V2
        assert!(ggc.security_control.encryption());
        assert_eq!(ggc.security_control.suite(), Some(SecuritySuite::V2));
        assert_eq!(ggc.security_control.suite_id(), 2);
        assert_eq!(ggc.system_title, system_title);
        assert_eq!(ggc.invocation_counter, Some(invocation_counter));

        // Encrypted payload should be different from plaintext
        assert_ne!(ggc.payload, plaintext);
        assert_eq!(ggc.payload.len(), plaintext.len());
    }

    #[test]
    fn test_encrypt_authenticated_v2() {
        let key = &[0u8; 32].into(); // 32-byte key for AES-256
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 42;
        let plaintext = b"Authenticated message V2";
        let security_control = SecurityControl::new(0x00);

        let result = GeneralGloCiphering::encrypt_authenticated_v2(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ggc = result.unwrap();

        // Check that both flags are set and suite is V2
        assert!(ggc.security_control.encryption());
        assert!(ggc.security_control.authentication());
        assert_eq!(ggc.security_control.suite(), Some(SecuritySuite::V2));
        assert_eq!(ggc.security_control.suite_id(), 2);
        assert_eq!(ggc.system_title, system_title);
        assert_eq!(ggc.invocation_counter, Some(invocation_counter));

        // Encrypted payload should be different from plaintext
        assert_ne!(ggc.payload, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_v2_roundtrip() {
        let key: &Key<Aes256> = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
            0x1D, 0x1E, 0x1F, 0x20,
        ]
        .into();
        let system_title = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22];
        let invocation_counter = 12345;
        let plaintext = b"Test message for Suite V2 encryption";
        let security_control = SecurityControl::new(0x00);

        // Encrypt
        let encrypted = GeneralGloCiphering::encrypt_v2(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        assert!(encrypted.security_control.encryption());
        assert_eq!(encrypted.security_control.suite(), Some(SecuritySuite::V2));
        assert_ne!(encrypted.payload, plaintext);

        // Decrypt
        let decrypted = encrypted.decrypt_v2(key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_authenticated_decrypt_v2_roundtrip() {
        let key: &Key<Aes256> = &[0xAB; 32].into();
        let system_title = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let invocation_counter = 999;
        let plaintext = b"Authenticated V2 roundtrip test";
        let security_control = SecurityControl::new(0x00);

        // Encrypt with authentication
        let encrypted = GeneralGloCiphering::encrypt_authenticated_v2(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        assert!(encrypted.security_control.encryption());
        assert!(encrypted.security_control.authentication());
        assert_eq!(encrypted.security_control.suite(), Some(SecuritySuite::V2));

        // Decrypt
        let decrypted = encrypted.decrypt_v2(key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_v1_v2_produce_different_ciphertext() {
        let key_v1 = &[0x42; 16].into(); // AES-128
        let key_v2 = &[0x42; 32].into(); // AES-256
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"Same plaintext, different suite";
        let security_control = SecurityControl::new(0x00);

        let encrypted_v1 = GeneralGloCiphering::encrypt(
            plaintext,
            key_v1,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let encrypted_v2 = GeneralGloCiphering::encrypt_v2(
            plaintext,
            key_v2,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        // Different suites should produce different ciphertext
        assert_ne!(encrypted_v1.payload, encrypted_v2.payload);
        assert_eq!(encrypted_v1.security_control.suite(), Some(SecuritySuite::V1));
        assert_eq!(encrypted_v2.security_control.suite(), Some(SecuritySuite::V2));
    }

    #[test]
    fn test_encrypt_v2_empty_payload() {
        let key = &[0u8; 32].into();
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 1;
        let plaintext = b"";
        let security_control = SecurityControl::new(0x00);

        let result = GeneralGloCiphering::encrypt_v2(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ggc = result.unwrap();
        assert_eq!(ggc.payload.len(), 0);
        assert_eq!(ggc.security_control.suite(), Some(SecuritySuite::V2));
    }

    #[test]
    fn test_v2_key_size_matches_security_control() {
        let security_control = SecurityControl::with_suite(true, true, SecuritySuite::V2);
        assert_eq!(security_control.key_size(), 32);
        assert_eq!(security_control.suite(), Some(SecuritySuite::V2));

        let security_control = SecurityControl::with_suite(true, true, SecuritySuite::V1);
        assert_eq!(security_control.key_size(), 16);
        assert_eq!(security_control.suite(), Some(SecuritySuite::V1));
    }
}
