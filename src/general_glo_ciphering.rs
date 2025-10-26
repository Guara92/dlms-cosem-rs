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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralGloCiphering {
    system_title: [u8; 8],
    security_control: SecurityControl,
    invocation_counter: Option<u32>,
    payload: Vec<u8>,
}

impl GeneralGloCiphering {
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

#[cfg(test)]
mod tests {
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
