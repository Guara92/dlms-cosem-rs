use core::fmt::{self, Debug, Display};

use nom::{IResult, Parser, number::complete::u8};
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

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
