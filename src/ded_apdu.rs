//! DED (Dedicated) encrypted APDU wrappers
//!
//! This module provides wrappers for encrypting xDLMS APDUs using the
//! general-ded-ciphering structure as defined in DLMS Green Book Ed. 12.
//!
//! DED APDUs use dedicated (per-client) encryption with the following tags:
//! - DED-GET-Request: 0xD0
//! - DED-SET-Request: 0xD1
//! - DED-ACTION-Request: 0xD3
//! - DED-GET-Response: 0xD4
//! - DED-SET-Response: 0xD5
//! - DED-ACTION-Response: 0xD7
//!
//! The difference between GLO (Global) and DED (Dedicated):
//! - GLO: Same key shared across all clients (broadcast scenarios)
//! - DED: Different key per client (point-to-point security)
//!
//! DED ciphering uses AES-128-GCM with a 12-byte IV constructed from
//! the system title (8 bytes) and invocation counter (4 bytes).

use alloc::vec::Vec;

use aes::Aes128;
use aes_gcm::Aes128Gcm;
use aes_gcm::aead::{AeadInPlace, KeyInit};
use cipher::Key;
#[cfg(feature = "parse")]
use nom::{
    IResult, Parser,
    bytes::streaming::tag,
    combinator::cond,
    multi::{count, fill},
    number::streaming::{be_u16, be_u32, u8},
};

use crate::SecurityControl;

#[cfg(feature = "encode")]
use crate::ByteBuffer;

/// General-ded-ciphering structure for dedicated (per-client) encryption
///
/// This structure is similar to GeneralGloCiphering but uses dedicated keys
/// for point-to-point communication rather than broadcast keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralDedCiphering {
    pub(crate) system_title: [u8; 8],
    pub(crate) security_control: SecurityControl,
    pub(crate) invocation_counter: Option<u32>,
    pub(crate) payload: Vec<u8>,
}

impl GeneralDedCiphering {
    /// Create a new GeneralDedCiphering structure
    pub fn new(
        system_title: [u8; 8],
        security_control: SecurityControl,
        invocation_counter: Option<u32>,
        payload: Vec<u8>,
    ) -> Self {
        Self { system_title, security_control, invocation_counter, payload }
    }

    /// Encrypt a payload using AES-128-GCM with a dedicated key
    ///
    /// # Arguments
    /// * `payload` - The plaintext data to encrypt
    /// * `key` - The AES-128 dedicated encryption key (unique per client)
    /// * `system_title` - 8-byte system title (part of IV)
    /// * `invocation_counter` - 4-byte invocation counter (part of IV)
    /// * `security_control` - Security control flags
    ///
    /// # Returns
    /// A new `GeneralDedCiphering` with encrypted payload
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

        // Set encryption flag
        security_control.set_encryption(true);

        Ok(Self {
            system_title,
            security_control,
            invocation_counter: Some(invocation_counter),
            payload: encrypted_payload,
        })
    }

    /// Encrypt a payload with authentication (authenticated encryption)
    ///
    /// This performs AES-GCM encryption with authentication tag using a dedicated key.
    ///
    /// # Arguments
    /// * `payload` - The plaintext data to encrypt
    /// * `key` - The AES-128 dedicated encryption key (unique per client)
    /// * `system_title` - 8-byte system title (part of IV)
    /// * `invocation_counter` - 4-byte invocation counter (part of IV)
    /// * `security_control` - Security control flags
    ///
    /// # Returns
    /// A new `GeneralDedCiphering` with encrypted and authenticated payload
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

        // Set both encryption and authentication flags
        security_control.set_encryption(true);
        security_control.set_authentication(true);

        Ok(Self {
            system_title,
            security_control,
            invocation_counter: Some(invocation_counter),
            payload: encrypted_payload,
        })
    }

    /// Encode the GeneralDedCiphering structure to bytes
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
        } else if total_len <= 255 {
            buffer.push_u8(0x81); // Extended length, 1 byte follows
            buffer.push_u8(total_len as u8);
        } else {
            buffer.push_u8(0x82); // Extended length, 2 bytes follow
            buffer.push_u16(total_len as u16);
        }

        // Security control (1 byte)
        buffer.push_u8(self.security_control.encode());

        // Invocation counter (4 bytes, if present)
        if let Some(ic) = self.invocation_counter {
            buffer.push_u32(ic);
        }

        // Encrypted payload
        buffer.push_bytes(&self.payload);

        buffer
    }

    /// Decrypt the payload using the dedicated key
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

    /// Parse a GeneralDedCiphering structure from bytes
    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        // System title length tag (should be 8)
        let (input, _st_len) = tag(&[0x08][..]).parse(input)?;

        // System title (8 bytes)
        let mut system_title = [0u8; 8];
        let (input, _) = fill(u8, &mut system_title).parse(input)?;

        // Length of remaining data
        let (input, remaining_len) = u8(input)?;
        let (input, remaining_len) = if remaining_len == 0x81 {
            let (input, len) = u8(input)?;
            (input, len as usize)
        } else if remaining_len == 0x82 {
            let (input, len) = be_u16(input)?;
            (input, len as usize)
        } else {
            (input, remaining_len as usize)
        };

        // Security control (1 byte)
        let (input, security_control_byte) = u8(input)?;
        let security_control = SecurityControl::new(security_control_byte);

        // Invocation counter (4 bytes if auth or encryption is set)
        let has_invocation_counter =
            security_control.authentication() || security_control.encryption();
        let (input, invocation_counter) = cond(has_invocation_counter, be_u32).parse(input)?;

        // Calculate payload length
        let ic_len = if has_invocation_counter { 4 } else { 0 };
        let payload_len = remaining_len - 1 - ic_len;

        // Encrypted payload
        let (input, payload) = count(u8, payload_len).parse(input)?;

        Ok((input, Self { system_title, security_control, invocation_counter, payload }))
    }
}

/// DED-GET-Request (tag 0xD0)
///
/// Encrypted wrapper for GET-Request xDLMS APDU using dedicated key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedGetRequest {
    inner: GeneralDedCiphering,
}

impl DedGetRequest {
    /// Create a new DED-GET-Request by encrypting a plaintext GET-Request
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext GET-Request APDU bytes
    /// * `key` - The AES-128 dedicated encryption key (unique per client)
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `DedGetRequest` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new DED-GET-Request with authenticated encryption
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext GET-Request APDU bytes
    /// * `key` - The AES-128 dedicated encryption key (unique per client)
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `DedGetRequest` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the DED-GET-Request to bytes
    ///
    /// Format: Tag (0xD0) + GeneralDedCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xD0); // DED-GET-Request tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralDedCiphering structure
    pub fn inner(&self) -> &GeneralDedCiphering {
        &self.inner
    }
}

/// DED-SET-Request (tag 0xD1)
///
/// Encrypted wrapper for SET-Request xDLMS APDU using dedicated key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedSetRequest {
    inner: GeneralDedCiphering,
}

impl DedSetRequest {
    /// Create a new DED-SET-Request by encrypting a plaintext SET-Request
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext SET-Request APDU bytes
    /// * `key` - The AES-128 dedicated encryption key (unique per client)
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `DedSetRequest` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new DED-SET-Request with authenticated encryption
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext SET-Request APDU bytes
    /// * `key` - The AES-128 dedicated encryption key (unique per client)
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `DedSetRequest` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the DED-SET-Request to bytes
    ///
    /// Format: Tag (0xD1) + GeneralDedCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xD1); // DED-SET-Request tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralDedCiphering structure
    pub fn inner(&self) -> &GeneralDedCiphering {
        &self.inner
    }
}

/// DED-ACTION-Request (tag 0xD3)
///
/// Encrypted wrapper for ACTION-Request xDLMS APDU using dedicated key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedActionRequest {
    inner: GeneralDedCiphering,
}

impl DedActionRequest {
    /// Create a new DED-ACTION-Request by encrypting a plaintext ACTION-Request
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext ACTION-Request APDU bytes
    /// * `key` - The AES-128 dedicated encryption key (unique per client)
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `DedActionRequest` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new DED-ACTION-Request with authenticated encryption
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext ACTION-Request APDU bytes
    /// * `key` - The AES-128 dedicated encryption key (unique per client)
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `DedActionRequest` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the DED-ACTION-Request to bytes
    ///
    /// Format: Tag (0xD3) + GeneralDedCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xD3); // DED-ACTION-Request tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralDedCiphering structure
    pub fn inner(&self) -> &GeneralDedCiphering {
        &self.inner
    }
}

/// DED-GET-Response (tag 0xD4)
///
/// Encrypted wrapper for GET-Response xDLMS APDU using dedicated key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedGetResponse {
    inner: GeneralDedCiphering,
}

impl DedGetResponse {
    /// Create a new DED-GET-Response by encrypting a plaintext GET-Response
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new DED-GET-Response with authenticated encryption
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the DED-GET-Response to bytes
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xD4); // DED-GET-Response tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralDedCiphering structure
    pub fn inner(&self) -> &GeneralDedCiphering {
        &self.inner
    }
}

/// DED-SET-Response (tag 0xD5)
///
/// Encrypted wrapper for SET-Response xDLMS APDU using dedicated key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedSetResponse {
    inner: GeneralDedCiphering,
}

impl DedSetResponse {
    /// Create a new DED-SET-Response by encrypting a plaintext SET-Response
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new DED-SET-Response with authenticated encryption
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the DED-SET-Response to bytes
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xD5); // DED-SET-Response tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralDedCiphering structure
    pub fn inner(&self) -> &GeneralDedCiphering {
        &self.inner
    }
}

/// DED-ACTION-Response (tag 0xD7)
///
/// Encrypted wrapper for ACTION-Response xDLMS APDU using dedicated key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedActionResponse {
    inner: GeneralDedCiphering,
}

impl DedActionResponse {
    /// Create a new DED-ACTION-Response by encrypting a plaintext ACTION-Response
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new DED-ACTION-Response with authenticated encryption
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralDedCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the DED-ACTION-Response to bytes
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xD7); // DED-ACTION-Response tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralDedCiphering structure
    pub fn inner(&self) -> &GeneralDedCiphering {
        &self.inner
    }
}

#[cfg(all(test, feature = "encode"))]
mod tests {
    use super::*;

    #[test]
    fn test_general_ded_ciphering_encrypt() {
        let key: &Key<Aes128> = &[
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ]
        .into();
        let system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let invocation_counter = 0x00112233;
        let plaintext = b"Dedicated encryption test";
        let security_control = SecurityControl::new(0x00);

        let result = GeneralDedCiphering::encrypt(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ded = result.unwrap();
        assert_eq!(ded.system_title, system_title);
        assert!(ded.security_control.encryption());
        assert_eq!(ded.invocation_counter, Some(invocation_counter));
        assert_ne!(ded.payload, plaintext); // Should be encrypted
    }

    #[test]
    fn test_general_ded_ciphering_encrypt_decrypt_roundtrip() {
        let key: &Key<Aes128> = &[0xAA; 16].into();
        let system_title = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let invocation_counter = 999;
        let plaintext = b"Test roundtrip encryption";
        let security_control = SecurityControl::new(0x00);

        // Encrypt
        let encrypted = GeneralDedCiphering::encrypt(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        // Decrypt
        let decrypted = encrypted.decrypt(key).unwrap();

        // Should match original
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_general_ded_ciphering_authenticated() {
        let key: &Key<Aes128> = &[0xFF; 16].into();
        let system_title = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22];
        let invocation_counter = 12345;
        let plaintext = b"Authenticated DED";
        let security_control = SecurityControl::new(0x00);

        let result = GeneralDedCiphering::encrypt_authenticated(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ded = result.unwrap();
        assert!(ded.security_control.encryption());
        assert!(ded.security_control.authentication());
    }

    #[test]
    fn test_ded_get_request_new() {
        let key = &[0u8; 16].into();
        let system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let invocation_counter = 1;
        let plaintext = b"DED GET-Request payload";
        let security_control = SecurityControl::new(0x00);

        let result =
            DedGetRequest::new(plaintext, key, system_title, invocation_counter, security_control);

        assert!(result.is_ok());
        let ded_get = result.unwrap();
        assert!(ded_get.inner().security_control.encryption());
    }

    #[test]
    fn test_ded_get_request_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let invocation_counter = 1;
        let plaintext = b"DED-GET";
        let security_control = SecurityControl::new(0x00);

        let ded_get =
            DedGetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();

        let encoded = ded_get.encode();

        // First byte should be DED-GET-Request tag (0xD0)
        assert_eq!(encoded[0], 0xD0);

        // Second byte should be system title length (0x08)
        assert_eq!(encoded[1], 0x08);

        // Next 8 bytes should be system title
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_ded_set_request_new() {
        let key = &[0u8; 16].into();
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 100;
        let plaintext = b"DED SET-Request payload";
        let security_control = SecurityControl::new(0x00);

        let result =
            DedSetRequest::new(plaintext, key, system_title, invocation_counter, security_control);

        assert!(result.is_ok());
        let ded_set = result.unwrap();
        assert!(ded_set.inner().security_control.encryption());
    }

    #[test]
    fn test_ded_set_request_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 1;
        let plaintext = b"DED-SET";
        let security_control = SecurityControl::new(0x00);

        let ded_set =
            DedSetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();

        let encoded = ded_set.encode();

        // First byte should be DED-SET-Request tag (0xD1)
        assert_eq!(encoded[0], 0xD1);
        assert_eq!(encoded[1], 0x08);
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_ded_action_request_new() {
        let key = &[0u8; 16].into();
        let system_title = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let invocation_counter = 12345;
        let plaintext = b"DED ACTION-Request payload";
        let security_control = SecurityControl::new(0x00);

        let result = DedActionRequest::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let ded_action = result.unwrap();
        assert!(ded_action.inner().security_control.encryption());
    }

    #[test]
    fn test_ded_action_request_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let invocation_counter = 1;
        let plaintext = b"DED-ACTION";
        let security_control = SecurityControl::new(0x00);

        let ded_action = DedActionRequest::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let encoded = ded_action.encode();

        // First byte should be DED-ACTION-Request tag (0xD3)
        assert_eq!(encoded[0], 0xD3);
        assert_eq!(encoded[1], 0x08);
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_ded_request_types_different_tags() {
        let key = &[0u8; 16].into();
        let system_title = [0; 8];
        let invocation_counter = 1;
        let plaintext = b"test";
        let security_control = SecurityControl::new(0x00);

        let ded_get =
            DedGetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let ded_set =
            DedSetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let ded_action = DedActionRequest::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let encoded_get = ded_get.encode();
        let encoded_set = ded_set.encode();
        let encoded_action = ded_action.encode();

        // Each should have different tag
        assert_eq!(encoded_get[0], 0xD0);
        assert_eq!(encoded_set[0], 0xD1);
        assert_eq!(encoded_action[0], 0xD3);

        // All different from each other
        assert_ne!(encoded_get[0], encoded_set[0]);
        assert_ne!(encoded_set[0], encoded_action[0]);
        assert_ne!(encoded_get[0], encoded_action[0]);
    }

    #[test]
    fn test_ded_response_types() {
        let key = &[0u8; 16].into();
        let system_title = [0; 8];
        let invocation_counter = 1;
        let plaintext = b"response";
        let security_control = SecurityControl::new(0x00);

        let ded_get_resp =
            DedGetResponse::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let ded_set_resp =
            DedSetResponse::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let ded_action_resp = DedActionResponse::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let encoded_get = ded_get_resp.encode();
        let encoded_set = ded_set_resp.encode();
        let encoded_action = ded_action_resp.encode();

        // Response tags should be 0xD4, 0xD5, 0xD7
        assert_eq!(encoded_get[0], 0xD4);
        assert_eq!(encoded_set[0], 0xD5);
        assert_eq!(encoded_action[0], 0xD7);
    }

    #[test]
    fn test_clone_and_equality() {
        let key = &[0u8; 16].into();
        let system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let invocation_counter = 1;
        let plaintext = b"Test";
        let security_control = SecurityControl::new(0x00);

        let ded_get1 =
            DedGetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let ded_get2 = ded_get1.clone();

        assert_eq!(ded_get1, ded_get2);
    }

    #[test]
    fn test_debug_format() {
        let key = &[0u8; 16].into();
        let system_title = [0x4D, 0x4D, 0x4D, 0x00, 0x00, 0xBC, 0x61, 0x4E];
        let invocation_counter = 1;
        let plaintext = b"Test";
        let security_control = SecurityControl::new(0x00);

        let ded_get =
            DedGetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();

        let debug_str = format!("{:?}", ded_get);
        assert!(debug_str.contains("DedGetRequest"));
    }
}
