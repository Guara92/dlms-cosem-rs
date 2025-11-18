//! GLO (Global) encrypted APDU wrappers
//!
//! This module provides wrappers for encrypting xDLMS APDUs using the
//! general-glo-ciphering structure as defined in DLMS Green Book Ed. 12.
//!
//! GLO APDUs use the following tags:
//! - GLO-GET-Request: 0xC8
//! - GLO-SET-Request: 0xC9
//! - GLO-ACTION-Request: 0xCB
//! - GLO-GET-Response: 0xC4
//! - GLO-SET-Response: 0xC5
//! - GLO-ACTION-Response: 0xC7
//!
//! These wrappers encrypt the entire xDLMS APDU payload using AES-128-GCM
//! with a 12-byte IV constructed from the system title (8 bytes) and
//! invocation counter (4 bytes).

use alloc::vec::Vec;

use aes::Aes128;
use cipher::Key;

use crate::{GeneralGloCiphering, SecurityControl};

#[cfg(feature = "encode")]
use crate::ByteBuffer;

/// GLO-GET-Request (tag 0xC8)
///
/// Encrypted wrapper for GET-Request xDLMS APDU.
/// The plaintext GET-Request is encrypted using AES-128-GCM before transmission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GloGetRequest {
    inner: GeneralGloCiphering,
}

impl GloGetRequest {
    /// Create a new GLO-GET-Request by encrypting a plaintext GET-Request
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext GET-Request APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `GloGetRequest` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new GLO-GET-Request with authenticated encryption
    ///
    /// This performs AES-GCM encryption with authentication tag.
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext GET-Request APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `GloGetRequest` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the GLO-GET-Request to bytes
    ///
    /// Format: Tag (0xC8) + GeneralGloCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xC8); // GLO-GET-Request tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralGloCiphering structure
    pub fn inner(&self) -> &GeneralGloCiphering {
        &self.inner
    }
}

/// GLO-SET-Request (tag 0xC9)
///
/// Encrypted wrapper for SET-Request xDLMS APDU.
/// The plaintext SET-Request is encrypted using AES-128-GCM before transmission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GloSetRequest {
    inner: GeneralGloCiphering,
}

impl GloSetRequest {
    /// Create a new GLO-SET-Request by encrypting a plaintext SET-Request
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext SET-Request APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `GloSetRequest` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new GLO-SET-Request with authenticated encryption
    ///
    /// This performs AES-GCM encryption with authentication tag.
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext SET-Request APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `GloSetRequest` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the GLO-SET-Request to bytes
    ///
    /// Format: Tag (0xC9) + GeneralGloCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xC9); // GLO-SET-Request tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralGloCiphering structure
    pub fn inner(&self) -> &GeneralGloCiphering {
        &self.inner
    }
}

/// GLO-ACTION-Request (tag 0xCB)
///
/// Encrypted wrapper for ACTION-Request xDLMS APDU.
/// The plaintext ACTION-Request is encrypted using AES-128-GCM before transmission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GloActionRequest {
    inner: GeneralGloCiphering,
}

impl GloActionRequest {
    /// Create a new GLO-ACTION-Request by encrypting a plaintext ACTION-Request
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext ACTION-Request APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `GloActionRequest` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new GLO-ACTION-Request with authenticated encryption
    ///
    /// This performs AES-GCM encryption with authentication tag.
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext ACTION-Request APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `GloActionRequest` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the GLO-ACTION-Request to bytes
    ///
    /// Format: Tag (0xCB) + GeneralGloCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xCB); // GLO-ACTION-Request tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralGloCiphering structure
    pub fn inner(&self) -> &GeneralGloCiphering {
        &self.inner
    }
}

/// GLO-GET-Response (tag 0xC4)
///
/// Encrypted wrapper for GET-Response xDLMS APDU.
/// This is typically received from the server and decrypted by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GloGetResponse {
    inner: GeneralGloCiphering,
}

impl GloGetResponse {
    /// Create a new GLO-GET-Response by encrypting a plaintext GET-Response
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext GET-Response APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `GloGetResponse` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new GLO-GET-Response with authenticated encryption
    ///
    /// This performs AES-GCM encryption with authentication tag.
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext GET-Response APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `GloGetResponse` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the GLO-GET-Response to bytes
    ///
    /// Format: Tag (0xCC) + GeneralGloCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xCC); // GLO-GET-Response tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralGloCiphering structure
    pub fn inner(&self) -> &GeneralGloCiphering {
        &self.inner
    }
}

/// GLO-SET-Response (tag 0xC5)
///
/// Encrypted wrapper for SET-Response xDLMS APDU.
/// This is typically received from the server and decrypted by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GloSetResponse {
    inner: GeneralGloCiphering,
}

impl GloSetResponse {
    /// Create a new GLO-SET-Response by encrypting a plaintext SET-Response
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext SET-Response APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `GloSetResponse` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new GLO-SET-Response with authenticated encryption
    ///
    /// This performs AES-GCM encryption with authentication tag.
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext SET-Response APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `GloSetResponse` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the GLO-SET-Response to bytes
    ///
    /// Format: Tag (0xCD) + GeneralGloCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xCD); // GLO-SET-Response tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralGloCiphering structure
    pub fn inner(&self) -> &GeneralGloCiphering {
        &self.inner
    }
}

/// GLO-ACTION-Response (tag 0xC7)
///
/// Encrypted wrapper for ACTION-Response xDLMS APDU.
/// This is typically received from the server and decrypted by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GloActionResponse {
    inner: GeneralGloCiphering,
}

impl GloActionResponse {
    /// Create a new GLO-ACTION-Response by encrypting a plaintext ACTION-Response
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext ACTION-Response APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (encryption will be set automatically)
    ///
    /// # Returns
    /// A new `GloActionResponse` with encrypted payload
    pub fn new(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Create a new GLO-ACTION-Response with authenticated encryption
    ///
    /// This performs AES-GCM encryption with authentication tag.
    ///
    /// # Arguments
    /// * `plaintext_apdu` - The plaintext ACTION-Response APDU bytes
    /// * `key` - The AES-128 encryption key
    /// * `system_title` - 8-byte system title
    /// * `invocation_counter` - 4-byte invocation counter (must be unique per message)
    /// * `security_control` - Security control flags (both encryption and auth will be set)
    ///
    /// # Returns
    /// A new `GloActionResponse` with encrypted and authenticated payload
    pub fn new_authenticated(
        plaintext_apdu: &[u8],
        key: &Key<Aes128>,
        system_title: [u8; 8],
        invocation_counter: u32,
        security_control: SecurityControl,
    ) -> Result<Self, aes_gcm::Error> {
        let inner = GeneralGloCiphering::encrypt_authenticated(
            plaintext_apdu,
            key,
            system_title,
            invocation_counter,
            security_control,
        )?;
        Ok(Self { inner })
    }

    /// Encode the GLO-ACTION-Response to bytes
    ///
    /// Format: Tag (0xCF) + GeneralGloCiphering structure
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push_u8(0xCF); // GLO-ACTION-Response tag
        buffer.push_bytes(&self.inner.encode());
        buffer
    }

    /// Get a reference to the inner GeneralGloCiphering structure
    pub fn inner(&self) -> &GeneralGloCiphering {
        &self.inner
    }
}

#[cfg(all(test, feature = "encode"))]
mod tests {
    use super::*;

    #[test]
    fn test_glo_get_request_new() {
        let key = &[0u8; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"GET-Request payload";
        let security_control = SecurityControl::new(0x00);

        let result =
            GloGetRequest::new(plaintext, key, system_title, invocation_counter, security_control);

        assert!(result.is_ok());
        let glo_get = result.unwrap();

        // Check that the inner structure has encryption flag set
        assert!(glo_get.inner().security_control.encryption());
    }

    #[test]
    fn test_glo_get_request_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"GET";
        let security_control = SecurityControl::new(0x00);

        let glo_get =
            GloGetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();

        let encoded = glo_get.encode();

        // First byte should be the GLO-GET-Request tag (0xC8)
        assert_eq!(encoded[0], 0xC8);

        // Second byte should be the system title length tag (0x08)
        assert_eq!(encoded[1], 0x08);

        // Next 8 bytes should be the system title
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_glo_get_request_authenticated() {
        let key = &[0xFF; 16].into();
        let system_title = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let invocation_counter = 42;
        let plaintext = b"Authenticated GET";
        let security_control = SecurityControl::new(0x00);

        let glo_get = GloGetRequest::new_authenticated(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        // Check that both encryption and authentication flags are set
        assert!(glo_get.inner().security_control.encryption());
        assert!(glo_get.inner().security_control.authentication());
    }

    #[test]
    fn test_glo_set_request_new() {
        let key = &[0u8; 16].into();
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 100;
        let plaintext = b"SET-Request payload";
        let security_control = SecurityControl::new(0x00);

        let result =
            GloSetRequest::new(plaintext, key, system_title, invocation_counter, security_control);

        assert!(result.is_ok());
        let glo_set = result.unwrap();

        assert!(glo_set.inner().security_control.encryption());
    }

    #[test]
    fn test_glo_set_request_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 1;
        let plaintext = b"SET";
        let security_control = SecurityControl::new(0x00);

        let glo_set =
            GloSetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();

        let encoded = glo_set.encode();

        // First byte should be the GLO-SET-Request tag (0xC9)
        assert_eq!(encoded[0], 0xC9);

        // Second byte should be the system title length tag (0x08)
        assert_eq!(encoded[1], 0x08);

        // Next 8 bytes should be the system title
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_glo_set_request_authenticated() {
        let key = &[0xAA; 16].into();
        let system_title = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22];
        let invocation_counter = 999;
        let plaintext = b"Authenticated SET";
        let security_control = SecurityControl::new(0x00);

        let glo_set = GloSetRequest::new_authenticated(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        assert!(glo_set.inner().security_control.encryption());
        assert!(glo_set.inner().security_control.authentication());
    }

    #[test]
    fn test_glo_action_request_new() {
        let key = &[0u8; 16].into();
        let system_title = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let invocation_counter = 12345;
        let plaintext = b"ACTION-Request payload";
        let security_control = SecurityControl::new(0x00);

        let result = GloActionRequest::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let glo_action = result.unwrap();

        assert!(glo_action.inner().security_control.encryption());
    }

    #[test]
    fn test_glo_action_request_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let invocation_counter = 1;
        let plaintext = b"ACTION";
        let security_control = SecurityControl::new(0x00);

        let glo_action = GloActionRequest::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let encoded = glo_action.encode();

        // First byte should be the GLO-ACTION-Request tag (0xCB)
        assert_eq!(encoded[0], 0xCB);

        // Second byte should be the system title length tag (0x08)
        assert_eq!(encoded[1], 0x08);

        // Next 8 bytes should be the system title
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_glo_action_request_authenticated() {
        let key = &[0x55; 16].into();
        let system_title = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        let invocation_counter = 777;
        let plaintext = b"Authenticated ACTION";
        let security_control = SecurityControl::new(0x00);

        let glo_action = GloActionRequest::new_authenticated(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        assert!(glo_action.inner().security_control.encryption());
        assert!(glo_action.inner().security_control.authentication());
    }

    #[test]
    fn test_different_glo_types_different_tags() {
        let key = &[0u8; 16].into();
        let system_title = [0; 8];
        let invocation_counter = 1;
        let plaintext = b"test";
        let security_control = SecurityControl::new(0x00);

        let glo_get =
            GloGetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let glo_set =
            GloSetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let glo_action = GloActionRequest::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let get_encoded = glo_get.encode();
        let set_encoded = glo_set.encode();
        let action_encoded = glo_action.encode();

        // Verify different tags
        assert_eq!(get_encoded[0], 0xC8);
        assert_eq!(set_encoded[0], 0xC9);
        assert_eq!(action_encoded[0], 0xCB);

        // Rest of the structure should be similar (same key, IC, system title)
        assert_eq!(&get_encoded[1..], &set_encoded[1..]);
        assert_eq!(&get_encoded[1..], &action_encoded[1..]);
    }

    #[test]
    fn test_clone_and_equality() {
        let key = &[0u8; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"Test";
        let security_control = SecurityControl::new(0x00);

        let glo_get1 =
            GloGetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let glo_get2 = glo_get1.clone();

        assert_eq!(glo_get1, glo_get2);
    }

    #[test]
    fn test_debug_format() {
        let key = &[0u8; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"Test";
        let security_control = SecurityControl::new(0x00);

        let glo_get =
            GloGetRequest::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();

        let debug_str = format!("{:?}", glo_get);
        assert!(debug_str.contains("GloGetRequest"));
    }

    #[test]
    fn test_glo_get_response_new() {
        let key = &[0u8; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"GET-Response payload";
        let security_control = SecurityControl::new(0x00);

        let result =
            GloGetResponse::new(plaintext, key, system_title, invocation_counter, security_control);

        assert!(result.is_ok());
        let glo_get_resp = result.unwrap();

        // Check that the inner structure has encryption flag set
        assert!(glo_get_resp.inner().security_control.encryption());
    }

    #[test]
    fn test_glo_get_response_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0x4b, 0x46, 0x4d, 0x10, 0x20, 0x01, 0x12, 0xa9];
        let invocation_counter = 1;
        let plaintext = b"GET-RESP";
        let security_control = SecurityControl::new(0x00);

        let glo_get_resp =
            GloGetResponse::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();

        let encoded = glo_get_resp.encode();

        // First byte should be the GLO-GET-Response tag (0xCC)
        assert_eq!(encoded[0], 0xCC);

        // Second byte should be the system title length tag (0x08)
        assert_eq!(encoded[1], 0x08);

        // Next 8 bytes should be the system title
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_glo_set_response_new() {
        let key = &[0u8; 16].into();
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 100;
        let plaintext = b"SET-Response payload";
        let security_control = SecurityControl::new(0x00);

        let result =
            GloSetResponse::new(plaintext, key, system_title, invocation_counter, security_control);

        assert!(result.is_ok());
        let glo_set_resp = result.unwrap();
        assert!(glo_set_resp.inner().security_control.encryption());
    }

    #[test]
    fn test_glo_set_response_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let invocation_counter = 1;
        let plaintext = b"SET-RESP";
        let security_control = SecurityControl::new(0x00);

        let glo_set_resp =
            GloSetResponse::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();

        let encoded = glo_set_resp.encode();

        // First byte should be the GLO-SET-Response tag (0xCD)
        assert_eq!(encoded[0], 0xCD);

        // Second byte should be the system title length tag (0x08)
        assert_eq!(encoded[1], 0x08);

        // Next 8 bytes should be the system title
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_glo_action_response_new() {
        let key = &[0u8; 16].into();
        let system_title = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let invocation_counter = 12345;
        let plaintext = b"ACTION-Response payload";
        let security_control = SecurityControl::new(0x00);

        let result = GloActionResponse::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        );

        assert!(result.is_ok());
        let glo_action_resp = result.unwrap();
        assert!(glo_action_resp.inner().security_control.encryption());
    }

    #[test]
    fn test_glo_action_response_encode() {
        let key = &[0u8; 16].into();
        let system_title = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE];
        let invocation_counter = 1;
        let plaintext = b"ACTION-RESP";
        let security_control = SecurityControl::new(0x00);

        let glo_action_resp = GloActionResponse::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let encoded = glo_action_resp.encode();

        // First byte should be the GLO-ACTION-Response tag (0xCF)
        assert_eq!(encoded[0], 0xCF);

        // Second byte should be the system title length tag (0x08)
        assert_eq!(encoded[1], 0x08);

        // Next 8 bytes should be the system title
        assert_eq!(&encoded[2..10], &system_title);
    }

    #[test]
    fn test_glo_response_types_different_tags() {
        let key = &[0u8; 16].into();
        let system_title = [0; 8];
        let invocation_counter = 1;
        let plaintext = b"test";
        let security_control = SecurityControl::new(0x00);

        let glo_get_resp =
            GloGetResponse::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let glo_set_resp =
            GloSetResponse::new(plaintext, key, system_title, invocation_counter, security_control)
                .unwrap();
        let glo_action_resp = GloActionResponse::new(
            plaintext,
            key,
            system_title,
            invocation_counter,
            security_control,
        )
        .unwrap();

        let encoded_get = glo_get_resp.encode();
        let encoded_set = glo_set_resp.encode();
        let encoded_action = glo_action_resp.encode();

        // Each should have a different tag
        assert_eq!(encoded_get[0], 0xCC); // GLO-GET-Response
        assert_eq!(encoded_set[0], 0xCD); // GLO-SET-Response
        assert_eq!(encoded_action[0], 0xCF); // GLO-ACTION-Response

        // Tags should be different from each other
        assert_ne!(encoded_get[0], encoded_set[0]);
        assert_ne!(encoded_set[0], encoded_action[0]);
        assert_ne!(encoded_get[0], encoded_action[0]);
    }
}
