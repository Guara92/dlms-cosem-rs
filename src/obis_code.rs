use core::fmt::{self, Debug, Display};

use nom::{IResult, Parser, number::complete::u8};
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

#[cfg(feature = "encode")]
extern crate alloc;

/// An OBIS code.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObisCode {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
}

impl ObisCode {
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, (a, b, c, d, e, f)) = (u8, u8, u8, u8, u8, u8).parse(input)?;
        Ok((input, Self::new(a, b, c, d, e, f)))
    }

    /// Encode OBIS code as 6 raw bytes (A-B-C-D-E-F)
    ///
    /// Returns a fixed-size array of 6 bytes representing the OBIS code.
    /// This is the raw binary format without any A-XDR type tags.
    ///
    /// # Example
    /// ```
    /// use dlms_cosem::ObisCode;
    ///
    /// let code = ObisCode::new(1, 0, 1, 8, 0, 255);
    /// let encoded = code.encode();
    /// assert_eq!(encoded, [1, 0, 1, 8, 0, 255]);
    /// ```
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> [u8; 6] {
        [self.a, self.b, self.c, self.d, self.e, self.f]
    }

    /// Encode OBIS code with A-XDR type tag: 09 06 A B C D E F
    ///
    /// Returns a Vec containing:
    /// - Tag 09 = octet-string (per DLMS Green Book Section 4.1.6.1)
    /// - Length 06 = 6 bytes
    /// - 6 bytes of the OBIS code (A-B-C-D-E-F)
    ///
    /// Total length: 8 bytes (tag + length + 6 data bytes)
    ///
    /// # Example
    /// ```
    /// use dlms_cosem::ObisCode;
    ///
    /// let code = ObisCode::new(1, 0, 1, 8, 0, 255);
    /// let encoded = code.encode_with_type();
    /// assert_eq!(encoded, vec![0x09, 0x06, 1, 0, 1, 8, 0, 255]);
    /// ```
    #[cfg(feature = "encode")]
    pub fn encode_with_type(&self) -> alloc::vec::Vec<u8> {
        alloc::vec![0x09, 0x06, self.a, self.b, self.c, self.d, self.e, self.f]
    }
}

impl Display for ObisCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}:{}.{}.{}*{}", self.a, self.b, self.c, self.d, self.e, self.f)
    }
}

impl Debug for ObisCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ObisCode({})", self)
    }
}

#[cfg(feature = "serde")]
impl Serialize for ObisCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ENCODING TESTS ====================
    // Following TDD approach: write tests first (RED phase)

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_basic() {
        // Test basic OBIS code encoding to 6 raw bytes
        let code = ObisCode::new(1, 0, 1, 8, 0, 255);
        let encoded = code.encode();

        assert_eq!(encoded, [1, 0, 1, 8, 0, 255]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_all_zeros() {
        let code = ObisCode::new(0, 0, 0, 0, 0, 0);
        let encoded = code.encode();

        assert_eq!(encoded, [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_all_max() {
        let code = ObisCode::new(255, 255, 255, 255, 255, 255);
        let encoded = code.encode();

        assert_eq!(encoded, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_with_type_basic() {
        // Test A-XDR encoding: tag (0x09) + length (0x06) + 6 bytes
        let code = ObisCode::new(1, 0, 1, 8, 0, 255);
        let encoded = code.encode_with_type();

        // Expected: 0x09 (octet-string tag), 0x06 (length=6), then 6 bytes
        assert_eq!(encoded, vec![0x09, 0x06, 1, 0, 1, 8, 0, 255]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_with_type_all_zeros() {
        let code = ObisCode::new(0, 0, 0, 0, 0, 0);
        let encoded = code.encode_with_type();

        assert_eq!(encoded, vec![0x09, 0x06, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_with_type_all_max() {
        let code = ObisCode::new(255, 255, 255, 255, 255, 255);
        let encoded = code.encode_with_type();

        assert_eq!(encoded, vec![0x09, 0x06, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_roundtrip_basic() {
        // Test encode → parse roundtrip
        let original = ObisCode::new(1, 0, 1, 8, 0, 255);
        let encoded = original.encode();
        let (remaining, parsed) = ObisCode::parse(&encoded).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_roundtrip_all_zeros() {
        let original = ObisCode::new(0, 0, 0, 0, 0, 0);
        let encoded = original.encode();
        let (_, parsed) = ObisCode::parse(&encoded).unwrap();

        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_roundtrip_all_max() {
        let original = ObisCode::new(255, 255, 255, 255, 255, 255);
        let encoded = original.encode();
        let (_, parsed) = ObisCode::parse(&encoded).unwrap();

        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_roundtrip_real_world_codes() {
        // Test real OBIS codes

        // Active energy total import (1-0:1.8.0*255)
        let code1 = ObisCode::new(1, 0, 1, 8, 0, 255);
        let encoded1 = code1.encode();
        let (_, parsed1) = ObisCode::parse(&encoded1).unwrap();
        assert_eq!(parsed1, code1);

        // Active power total (1-0:1.7.0*255)
        let code2 = ObisCode::new(1, 0, 1, 7, 0, 255);
        let encoded2 = code2.encode();
        let (_, parsed2) = ObisCode::parse(&encoded2).unwrap();
        assert_eq!(parsed2, code2);

        // Clock (0-0:1.0.0*255)
        let code3 = ObisCode::new(0, 0, 1, 0, 0, 255);
        let encoded3 = code3.encode();
        let (_, parsed3) = ObisCode::parse(&encoded3).unwrap();
        assert_eq!(parsed3, code3);

        // Voltage L1 (1-0:32.7.0*255)
        let code4 = ObisCode::new(1, 0, 32, 7, 0, 255);
        let encoded4 = code4.encode();
        let (_, parsed4) = ObisCode::parse(&encoded4).unwrap();
        assert_eq!(parsed4, code4);

        // Current L1 (1-0:31.7.0*255)
        let code5 = ObisCode::new(1, 0, 31, 7, 0, 255);
        let encoded5 = code5.encode();
        let (_, parsed5) = ObisCode::parse(&encoded5).unwrap();
        assert_eq!(parsed5, code5);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_with_type_roundtrip() {
        // Test encode_with_type → parse roundtrip
        // Note: parse expects raw 6 bytes, so we skip the A-XDR header
        let original = ObisCode::new(1, 0, 1, 8, 0, 255);
        let encoded = original.encode_with_type();

        // Skip tag (0x09) and length (0x06), parse the 6 bytes
        let (_, parsed) = ObisCode::parse(&encoded[2..]).unwrap();

        assert_eq!(parsed, original);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_with_type_header() {
        // Verify the A-XDR header is always correct
        let codes = vec![
            ObisCode::new(1, 0, 1, 8, 0, 255),
            ObisCode::new(0, 0, 0, 0, 0, 0),
            ObisCode::new(255, 255, 255, 255, 255, 255),
            ObisCode::new(1, 0, 32, 7, 0, 255),
        ];

        for code in codes {
            let encoded = code.encode_with_type();

            // All encodings should start with 0x09 0x06
            assert_eq!(encoded[0], 0x09, "Tag should be 0x09 (octet-string)");
            assert_eq!(encoded[1], 0x06, "Length should be 0x06 (6 bytes)");
            assert_eq!(encoded.len(), 8, "Total length should be 8 bytes");
        }
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_encode_matches_parse_format() {
        // Verify that encoded output matches what parse() expects
        let code = ObisCode::new(10, 20, 30, 40, 50, 60);
        let encoded = code.encode();

        // Create test input that parse() expects
        let expected = [10, 20, 30, 40, 50, 60];

        assert_eq!(encoded, expected);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_green_book_compliance_get_request_example() {
        // Verify compliance with DLMS Green Book Ed. 12, Line 1458
        // GET-Request example: C0 01 00 03 01 01 01 08 00 FF 02
        // Where: 01 01 01 08 00 FF is the OBIS code for Register (1-0:1.8.0*255)

        let code = ObisCode::new(1, 0, 1, 8, 0, 255);
        let encoded = code.encode();

        // OBIS code in GET-Request is encoded as raw 6 bytes (no A-XDR tag)
        assert_eq!(encoded, [0x01, 0x00, 0x01, 0x08, 0x00, 0xFF]);
        assert_eq!(encoded.len(), 6);

        // Verify it matches the Green Book example exactly
        let green_book_obis_bytes = [0x01, 0x00, 0x01, 0x08, 0x00, 0xFF];
        assert_eq!(encoded, green_book_obis_bytes);
    }

    #[test]
    #[cfg(feature = "encode")]
    fn test_green_book_compliance_data_octet_string() {
        // When OBIS code is used as Data::OctetString (e.g., in structures),
        // it must be encoded with A-XDR tag: 0x09 (octet-string) + 0x06 (length)

        let code = ObisCode::new(1, 0, 1, 8, 0, 255);
        let encoded_with_type = code.encode_with_type();

        // First byte: 0x09 = octet-string tag (per DataType enum)
        assert_eq!(encoded_with_type[0], 0x09);

        // Second byte: 0x06 = length (6 bytes for OBIS code)
        assert_eq!(encoded_with_type[1], 0x06);

        // Remaining 6 bytes: OBIS code
        assert_eq!(&encoded_with_type[2..8], &[0x01, 0x00, 0x01, 0x08, 0x00, 0xFF]);

        // Total length: 8 bytes (tag + length + 6 data bytes)
        assert_eq!(encoded_with_type.len(), 8);
    }

    // ==================== PARSING TESTS ====================

    #[test]
    fn test_parse_basic() {
        let input = [1, 2, 3, 4, 5, 6];
        let (remaining, code) = ObisCode::parse(&input).unwrap();

        assert_eq!(remaining, &[]);
        assert_eq!(code.a, 1);
        assert_eq!(code.b, 2);
        assert_eq!(code.c, 3);
        assert_eq!(code.d, 4);
        assert_eq!(code.e, 5);
        assert_eq!(code.f, 6);
    }

    #[test]
    fn test_parse_with_remaining() {
        let input = [10, 20, 30, 40, 50, 60, 0xFF, 0xAA];
        let (remaining, code) = ObisCode::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF, 0xAA]);
        assert_eq!(code.a, 10);
        assert_eq!(code.b, 20);
        assert_eq!(code.c, 30);
        assert_eq!(code.d, 40);
        assert_eq!(code.e, 50);
        assert_eq!(code.f, 60);
    }

    #[test]
    fn test_parse_all_zeros() {
        let input = [0, 0, 0, 0, 0, 0];
        let (_, code) = ObisCode::parse(&input).unwrap();

        assert_eq!(code, ObisCode::new(0, 0, 0, 0, 0, 0));
    }

    #[test]
    fn test_parse_all_max() {
        let input = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let (_, code) = ObisCode::parse(&input).unwrap();

        assert_eq!(code, ObisCode::new(255, 255, 255, 255, 255, 255));
    }

    #[test]
    fn test_parse_insufficient_input() {
        // Less than 6 bytes should fail
        let input = [1, 2, 3, 4, 5];
        let result = ObisCode::parse(&input);

        assert!(result.is_err());
    }

    #[test]
    fn test_new() {
        let code = ObisCode::new(1, 0, 1, 8, 0, 255);

        assert_eq!(code.a, 1);
        assert_eq!(code.b, 0);
        assert_eq!(code.c, 1);
        assert_eq!(code.d, 8);
        assert_eq!(code.e, 0);
        assert_eq!(code.f, 255);
    }

    #[test]
    fn test_display_format() {
        // Standard OBIS code format: A-B:C.D.E*F
        let code = ObisCode::new(1, 0, 1, 8, 0, 255);
        let display = format!("{}", code);

        assert_eq!(display, "1-0:1.8.0*255");
    }

    #[test]
    fn test_display_various_codes() {
        // Test common OBIS codes

        // Active energy import: 1-0:1.8.0*255
        let code = ObisCode::new(1, 0, 1, 8, 0, 255);
        assert_eq!(format!("{}", code), "1-0:1.8.0*255");

        // Active energy export: 1-0:2.8.0*255
        let code = ObisCode::new(1, 0, 2, 8, 0, 255);
        assert_eq!(format!("{}", code), "1-0:2.8.0*255");

        // Voltage L1: 1-0:32.7.0*255
        let code = ObisCode::new(1, 0, 32, 7, 0, 255);
        assert_eq!(format!("{}", code), "1-0:32.7.0*255");

        // Current L1: 1-0:31.7.0*255
        let code = ObisCode::new(1, 0, 31, 7, 0, 255);
        assert_eq!(format!("{}", code), "1-0:31.7.0*255");
    }

    #[test]
    fn test_debug_format() {
        let code = ObisCode::new(1, 0, 1, 8, 0, 255);
        let debug = format!("{:?}", code);

        assert!(debug.contains("ObisCode"));
        assert!(debug.contains("1-0:1.8.0*255"));
    }

    #[test]
    fn test_equality() {
        let code1 = ObisCode::new(1, 0, 1, 8, 0, 255);
        let code2 = ObisCode::new(1, 0, 1, 8, 0, 255);
        let code3 = ObisCode::new(1, 0, 2, 8, 0, 255);

        assert_eq!(code1, code2);
        assert_ne!(code1, code3);
    }

    #[test]
    fn test_clone() {
        let code1 = ObisCode::new(1, 0, 1, 8, 0, 255);
        let code2 = code1.clone();

        assert_eq!(code1, code2);
        assert_eq!(code1.a, code2.a);
        assert_eq!(code1.f, code2.f);
    }

    #[test]
    fn test_ordering() {
        let code1 = ObisCode::new(1, 0, 1, 8, 0, 255);
        let code2 = ObisCode::new(1, 0, 1, 8, 1, 255);
        let code3 = ObisCode::new(1, 0, 2, 8, 0, 255);

        assert!(code1 < code2);
        assert!(code2 < code3);
        assert!(code1 < code3);
    }

    #[test]
    fn test_parse_real_world_codes() {
        // Test parsing of actual OBIS codes that might be encountered

        // Active energy total import (1-0:1.8.0*255)
        let input = [1, 0, 1, 8, 0, 255];
        let (_, code) = ObisCode::parse(&input).unwrap();
        assert_eq!(format!("{}", code), "1-0:1.8.0*255");

        // Active power total (1-0:1.7.0*255)
        let input = [1, 0, 1, 7, 0, 255];
        let (_, code) = ObisCode::parse(&input).unwrap();
        assert_eq!(format!("{}", code), "1-0:1.7.0*255");

        // Clock (0-0:1.0.0*255)
        let input = [0, 0, 1, 0, 0, 255];
        let (_, code) = ObisCode::parse(&input).unwrap();
        assert_eq!(format!("{}", code), "0-0:1.0.0*255");
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize() {
        use serde::Serialize;
        let code = ObisCode::new(1, 0, 1, 8, 0, 255);

        // Verify Display output which is used by Serialize
        let display = format!("{}", code);
        assert_eq!(display, "1-0:1.8.0*255");

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&code);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_various_codes() {
        use serde::Serialize;

        // Active energy
        let code1 = ObisCode::new(1, 0, 1, 8, 0, 255);
        assert_eq!(format!("{}", code1), "1-0:1.8.0*255");

        // Voltage
        let code2 = ObisCode::new(1, 0, 32, 7, 0, 255);
        assert_eq!(format!("{}", code2), "1-0:32.7.0*255");

        // All zeros
        let code3 = ObisCode::new(0, 0, 0, 0, 0, 0);
        assert_eq!(format!("{}", code3), "0-0:0.0.0*0");

        // Verify the trait is implemented (compile-time check)
        fn assert_serialize<T: Serialize>(_: &T) {}
        assert_serialize(&code1);
        assert_serialize(&code2);
        assert_serialize(&code3);
    }
}
