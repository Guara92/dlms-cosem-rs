use core::fmt;

#[cfg(feature = "parse")]
use nom::{IResult, number::complete::u8};

#[derive(Clone, PartialEq, Eq)]
pub struct SecurityControl {
    security_control: u8,
}

impl fmt::Debug for SecurityControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecurityControl")
            .field("suite_id", &self.suite_id())
            .field("authentication", &self.authentication())
            .field("encryption", &self.encryption())
            .field("broadcast", &self.broadcast())
            .field("compression", &self.compression())
            .finish()
    }
}

impl SecurityControl {
    #[rustfmt::skip]
    const COMPRESSION_BIT:    u8 = 0b10000000;
  #[rustfmt::skip]
    const BROADCAST_BIT:      u8 = 0b01000000;
  #[rustfmt::skip]
    const ENCRYPTION_BIT:     u8 = 0b00100000;
  #[rustfmt::skip]
    const AUTHENTICATION_BIT: u8 = 0b00010000;

    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, security_control) = u8(input)?;
        Ok((input, Self { security_control }))
    }

    pub fn suite_id(&self) -> u8 {
        self.security_control & 0b00001111
    }

    pub fn authentication(&self) -> bool {
        (self.security_control & Self::AUTHENTICATION_BIT) != 0
    }

    pub fn set_authentication(&mut self, authentication: bool) {
        if authentication {
            self.security_control |= Self::AUTHENTICATION_BIT
        } else {
            self.security_control &= !Self::AUTHENTICATION_BIT
        }
    }

    pub fn encryption(&self) -> bool {
        (self.security_control & Self::ENCRYPTION_BIT) != 0
    }

    pub fn set_encryption(&mut self, encryption: bool) {
        if encryption {
            self.security_control |= Self::ENCRYPTION_BIT
        } else {
            self.security_control &= !Self::ENCRYPTION_BIT
        }
    }

    pub fn broadcast(&self) -> bool {
        (self.security_control & Self::BROADCAST_BIT) != 0
    }

    pub fn set_broadcast(&mut self, broadcast: bool) {
        if broadcast {
            self.security_control |= Self::BROADCAST_BIT
        } else {
            self.security_control &= !Self::BROADCAST_BIT
        }
    }

    pub fn compression(&self) -> bool {
        (self.security_control & Self::COMPRESSION_BIT) != 0
    }

    pub fn set_compression(&mut self, compression: bool) {
        if compression {
            self.security_control |= Self::COMPRESSION_BIT
        } else {
            self.security_control &= !Self::COMPRESSION_BIT
        }
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_security_control() {
        // Test parsing basic security control byte
        let input = [0x30, 0xFF];
        let (remaining, sc) = SecurityControl::parse(&input).unwrap();

        assert_eq!(remaining, &[0xFF]);
        assert_eq!(sc.security_control, 0x30);
    }

    #[test]
    fn test_suite_id() {
        // Suite ID is the lower 4 bits (0-15)
        let input = [0x00];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert_eq!(sc.suite_id(), 0);

        let input = [0x0F];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert_eq!(sc.suite_id(), 15);

        // Suite ID should only use lower 4 bits, ignoring upper bits
        let input = [0xFF];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert_eq!(sc.suite_id(), 15);

        let input = [0xF0];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert_eq!(sc.suite_id(), 0);
    }

    #[test]
    fn test_authentication_bit() {
        // Bit 4 (0x10) = Authentication
        let input = [0x00];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(!sc.authentication());

        let input = [0x10];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(sc.authentication());

        let input = [0xFF];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(sc.authentication());
    }

    #[test]
    fn test_set_authentication() {
        let input = [0x00];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();

        // Initially false
        assert!(!sc.authentication());

        // Set to true
        sc.set_authentication(true);
        assert!(sc.authentication());
        assert_eq!(sc.security_control, 0x10);

        // Set to false
        sc.set_authentication(false);
        assert!(!sc.authentication());
        assert_eq!(sc.security_control, 0x00);

        // Test that setting authentication doesn't affect other bits
        let input = [0xFF];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();
        sc.set_authentication(false);
        assert_eq!(sc.security_control, 0xEF);
        assert!(sc.encryption());
        assert!(sc.broadcast());
        assert!(sc.compression());
    }

    #[test]
    fn test_encryption_bit() {
        // Bit 5 (0x20) = Encryption
        let input = [0x00];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(!sc.encryption());

        let input = [0x20];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(sc.encryption());

        let input = [0xFF];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(sc.encryption());
    }

    #[test]
    fn test_set_encryption() {
        let input = [0x00];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();

        // Initially false
        assert!(!sc.encryption());

        // Set to true
        sc.set_encryption(true);
        assert!(sc.encryption());
        assert_eq!(sc.security_control, 0x20);

        // Set to false
        sc.set_encryption(false);
        assert!(!sc.encryption());
        assert_eq!(sc.security_control, 0x00);

        // Test that setting encryption doesn't affect other bits
        let input = [0xFF];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();
        sc.set_encryption(false);
        assert_eq!(sc.security_control, 0xDF);
        assert!(sc.authentication());
        assert!(sc.broadcast());
        assert!(sc.compression());
    }

    #[test]
    fn test_broadcast_bit() {
        // Bit 6 (0x40) = Broadcast
        let input = [0x00];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(!sc.broadcast());

        let input = [0x40];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(sc.broadcast());

        let input = [0xFF];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(sc.broadcast());
    }

    #[test]
    fn test_set_broadcast() {
        let input = [0x00];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();

        // Initially false
        assert!(!sc.broadcast());

        // Set to true
        sc.set_broadcast(true);
        assert!(sc.broadcast());
        assert_eq!(sc.security_control, 0x40);

        // Set to false
        sc.set_broadcast(false);
        assert!(!sc.broadcast());
        assert_eq!(sc.security_control, 0x00);

        // Test that setting broadcast doesn't affect other bits
        let input = [0xFF];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();
        sc.set_broadcast(false);
        assert_eq!(sc.security_control, 0xBF);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(sc.compression());
    }

    #[test]
    fn test_compression_bit() {
        // Bit 7 (0x80) = Compression
        let input = [0x00];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(!sc.compression());

        let input = [0x80];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(sc.compression());

        let input = [0xFF];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert!(sc.compression());
    }

    #[test]
    fn test_set_compression() {
        let input = [0x00];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();

        // Initially false
        assert!(!sc.compression());

        // Set to true
        sc.set_compression(true);
        assert!(sc.compression());
        assert_eq!(sc.security_control, 0x80);

        // Set to false
        sc.set_compression(false);
        assert!(!sc.compression());
        assert_eq!(sc.security_control, 0x00);

        // Test that setting compression doesn't affect other bits
        let input = [0xFF];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();
        sc.set_compression(false);
        assert_eq!(sc.security_control, 0x7F);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(sc.broadcast());
    }

    #[test]
    fn test_all_bits_combined() {
        // Test all combinations of flags

        // No flags set (0x00)
        let input = [0x00];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert_eq!(sc.suite_id(), 0);
        assert!(!sc.authentication());
        assert!(!sc.encryption());
        assert!(!sc.broadcast());
        assert!(!sc.compression());

        // All flags set (0xF0) with suite_id = 0
        let input = [0xF0];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert_eq!(sc.suite_id(), 0);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(sc.broadcast());
        assert!(sc.compression());

        // All flags set (0xFF) with suite_id = 15
        let input = [0xFF];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert_eq!(sc.suite_id(), 15);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(sc.broadcast());
        assert!(sc.compression());

        // Authentication + Encryption (0x30) - common case
        let input = [0x30];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        assert_eq!(sc.suite_id(), 0);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(!sc.broadcast());
        assert!(!sc.compression());
    }

    #[test]
    fn test_modify_multiple_bits() {
        let input = [0x00];
        let (_, mut sc) = SecurityControl::parse(&input).unwrap();

        // Set authentication and encryption
        sc.set_authentication(true);
        sc.set_encryption(true);
        assert_eq!(sc.security_control, 0x30);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(!sc.broadcast());
        assert!(!sc.compression());

        // Add broadcast
        sc.set_broadcast(true);
        assert_eq!(sc.security_control, 0x70);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(sc.broadcast());
        assert!(!sc.compression());

        // Remove encryption
        sc.set_encryption(false);
        assert_eq!(sc.security_control, 0x50);
        assert!(sc.authentication());
        assert!(!sc.encryption());
        assert!(sc.broadcast());
        assert!(!sc.compression());
    }

    #[test]
    fn test_debug_format() {
        // Test that Debug implementation includes all fields
        let input = [0x3F];
        let (_, sc) = SecurityControl::parse(&input).unwrap();
        let debug_str = format!("{:?}", sc);

        assert!(debug_str.contains("SecurityControl"));
        assert!(debug_str.contains("suite_id"));
        assert!(debug_str.contains("authentication"));
        assert!(debug_str.contains("encryption"));
        assert!(debug_str.contains("broadcast"));
        assert!(debug_str.contains("compression"));
    }
}
