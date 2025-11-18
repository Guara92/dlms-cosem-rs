use core::fmt;

#[cfg(feature = "parse")]
use nom::{IResult, number::complete::u8};

/// Security Suite version for DLMS encryption
///
/// Defines the cryptographic suite used for encryption and authentication.
///
/// # DLMS Green Book Ed. 12 Reference
/// - Section 8.3.3: Security suites
/// - Suite V1 (AES-128-GCM) is the most commonly used
/// - Suite V2 (AES-256-GCM) is becoming more common in new deployments
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SecuritySuite {
    /// Suite V0 - Legacy suite (deprecated, not recommended)
    V0 = 0,
    /// Suite V1 - AES-128-GCM (most common, 16-byte key)
    V1 = 1,
    /// Suite V2 - AES-256-GCM (newer deployments, 32-byte key)
    V2 = 2,
}

impl SecuritySuite {
    /// Get the key size in bytes for this suite
    ///
    /// # Returns
    /// - Suite V0: 16 bytes (AES-128)
    /// - Suite V1: 16 bytes (AES-128)
    /// - Suite V2: 32 bytes (AES-256)
    pub fn key_size(&self) -> usize {
        match self {
            SecuritySuite::V0 => 16,
            SecuritySuite::V1 => 16,
            SecuritySuite::V2 => 32,
        }
    }

    /// Create a SecuritySuite from a suite ID (0-15)
    ///
    /// # Arguments
    /// * `id` - Suite ID from security control byte (lower 4 bits)
    ///
    /// # Returns
    /// - Some(SecuritySuite) if id is 0, 1, or 2
    /// - None if id is invalid (3-15 are reserved/undefined)
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(SecuritySuite::V0),
            1 => Some(SecuritySuite::V1),
            2 => Some(SecuritySuite::V2),
            _ => None, // 3-15 are reserved/undefined
        }
    }
}

/// Security Control byte for DLMS encryption
///
/// The security control byte defines the security mode and suite for encrypted messages.
///
/// # Bit Layout (DLMS Green Book Ed. 12, Section 8.3.2)
///
/// ```text
/// Bit 7 (0x80): Compression - Data compression enabled
/// Bit 6 (0x40): Broadcast   - Broadcast key used (not client-specific)
/// Bit 5 (0x20): Encryption  - Encryption enabled
/// Bit 4 (0x10): Authentication - Authentication enabled (MAC tag)
/// Bits 3-0 (0x0F): Suite ID - Security suite version (0=V0, 1=V1, 2=V2)
/// ```
///
/// # Common Values
/// - `0x00`: No security
/// - `0x10`: Authentication only (MAC, no encryption)
/// - `0x20`: Encryption only (no MAC)
/// - `0x30`: Authentication + Encryption (most secure)
/// - `0x31`: Authentication + Encryption with Suite V1 (AES-128-GCM)
/// - `0x32`: Authentication + Encryption with Suite V2 (AES-256-GCM)
///
/// # Examples
///
/// ```
/// use dlms_cosem::{SecurityControl, SecuritySuite};
///
/// // Create with authentication + encryption, Suite V1
/// let mut sc = SecurityControl::new(0x00);
/// sc.set_authentication(true);
/// sc.set_encryption(true);
/// sc.set_suite(SecuritySuite::V1);
/// assert_eq!(sc.encode(), 0x31);
///
/// // Parse existing security control
/// let (_, sc) = SecurityControl::parse(&[0x32]).unwrap();
/// assert!(sc.authentication());
/// assert!(sc.encryption());
/// assert_eq!(sc.suite(), Some(SecuritySuite::V2));
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
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
    const COMPRESSION_BIT:    u8 = 0b10000000; // Bit 7 (0x80)
    #[rustfmt::skip]
    const BROADCAST_BIT:      u8 = 0b01000000; // Bit 6 (0x40)
    #[rustfmt::skip]
    const ENCRYPTION_BIT:     u8 = 0b00100000; // Bit 5 (0x20)
    #[rustfmt::skip]
    const AUTHENTICATION_BIT: u8 = 0b00010000; // Bit 4 (0x10)
    #[rustfmt::skip]
    const SUITE_MASK:         u8 = 0b00001111; // Bits 3-0 (0x0F)

    /// Create a new SecurityControl from a raw byte value
    ///
    /// # Arguments
    /// * `security_control` - Raw security control byte (0-255)
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::SecurityControl;
    ///
    /// let sc = SecurityControl::new(0x30); // Auth + Encryption
    /// assert!(sc.authentication());
    /// assert!(sc.encryption());
    /// ```
    pub fn new(security_control: u8) -> Self {
        Self { security_control }
    }

    /// Create a new SecurityControl with specified security mode and suite
    ///
    /// # Arguments
    /// * `authentication` - Enable authentication (MAC tag)
    /// * `encryption` - Enable encryption
    /// * `suite` - Security suite (V0, V1, or V2)
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::{SecurityControl, SecuritySuite};
    ///
    /// // Authenticated encryption with AES-128-GCM
    /// let sc = SecurityControl::with_suite(true, true, SecuritySuite::V1);
    /// assert_eq!(sc.encode(), 0x31);
    ///
    /// // Authenticated encryption with AES-256-GCM
    /// let sc = SecurityControl::with_suite(true, true, SecuritySuite::V2);
    /// assert_eq!(sc.encode(), 0x32);
    /// ```
    pub fn with_suite(authentication: bool, encryption: bool, suite: SecuritySuite) -> Self {
        let mut byte = suite as u8;
        if authentication {
            byte |= Self::AUTHENTICATION_BIT;
        }
        if encryption {
            byte |= Self::ENCRYPTION_BIT;
        }
        Self { security_control: byte }
    }

    /// Encode the SecurityControl to a single byte
    #[cfg(feature = "encode")]
    pub fn encode(&self) -> u8 {
        self.security_control
    }

    #[cfg(feature = "parse")]
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, security_control) = u8(input)?;
        Ok((input, Self { security_control }))
    }

    /// Get the raw suite ID (0-15) from the security control byte
    ///
    /// This returns the lower 4 bits. Use `suite()` to get a typed SecuritySuite.
    ///
    /// # Returns
    /// Suite ID (0-15), where:
    /// - 0 = Suite V0 (legacy)
    /// - 1 = Suite V1 (AES-128-GCM)
    /// - 2 = Suite V2 (AES-256-GCM)
    /// - 3-15 = Reserved/undefined
    pub fn suite_id(&self) -> u8 {
        self.security_control & Self::SUITE_MASK
    }

    /// Get the security suite (if valid)
    ///
    /// # Returns
    /// - Some(SecuritySuite) if suite ID is 0, 1, or 2
    /// - None if suite ID is invalid (3-15 are reserved)
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::{SecurityControl, SecuritySuite};
    ///
    /// let sc = SecurityControl::new(0x31); // Suite V1
    /// assert_eq!(sc.suite(), Some(SecuritySuite::V1));
    ///
    /// let sc = SecurityControl::new(0x32); // Suite V2
    /// assert_eq!(sc.suite(), Some(SecuritySuite::V2));
    ///
    /// let sc = SecurityControl::new(0x3F); // Invalid suite (15)
    /// assert_eq!(sc.suite(), None);
    /// ```
    pub fn suite(&self) -> Option<SecuritySuite> {
        SecuritySuite::from_id(self.suite_id())
    }

    /// Set the security suite
    ///
    /// This updates the lower 4 bits while preserving other flags.
    ///
    /// # Arguments
    /// * `suite` - Security suite to set (V0, V1, or V2)
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::{SecurityControl, SecuritySuite};
    ///
    /// let mut sc = SecurityControl::new(0x30); // Auth + Enc, Suite V0
    /// sc.set_suite(SecuritySuite::V2);
    /// assert_eq!(sc.encode(), 0x32); // Auth + Enc, Suite V2
    /// ```
    pub fn set_suite(&mut self, suite: SecuritySuite) {
        // Clear lower 4 bits, then set suite
        self.security_control = (self.security_control & !Self::SUITE_MASK) | (suite as u8);
    }

    /// Get the key size in bytes required for this security control
    ///
    /// # Returns
    /// - 16 bytes for Suite V0 and V1 (AES-128)
    /// - 32 bytes for Suite V2 (AES-256)
    /// - 16 bytes for invalid suites (defaults to AES-128)
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::{SecurityControl, SecuritySuite};
    ///
    /// let sc = SecurityControl::with_suite(true, true, SecuritySuite::V1);
    /// assert_eq!(sc.key_size(), 16); // AES-128
    ///
    /// let sc = SecurityControl::with_suite(true, true, SecuritySuite::V2);
    /// assert_eq!(sc.key_size(), 32); // AES-256
    /// ```
    pub fn key_size(&self) -> usize {
        self.suite().map(|s| s.key_size()).unwrap_or(16) // Default to AES-128 for invalid suites
    }

    /// Check if this security control uses authentication (MAC tag)
    ///
    /// # Returns
    /// true if authentication bit (bit 4, 0x10) is set
    pub fn authentication(&self) -> bool {
        (self.security_control & Self::AUTHENTICATION_BIT) != 0
    }

    /// Set the authentication flag
    ///
    /// # Arguments
    /// * `authentication` - true to enable authentication (MAC tag)
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::SecurityControl;
    ///
    /// let mut sc = SecurityControl::new(0x00);
    /// sc.set_authentication(true);
    /// assert_eq!(sc.encode(), 0x10);
    /// ```
    pub fn set_authentication(&mut self, authentication: bool) {
        if authentication {
            self.security_control |= Self::AUTHENTICATION_BIT
        } else {
            self.security_control &= !Self::AUTHENTICATION_BIT
        }
    }

    /// Check if this security control uses encryption
    ///
    /// # Returns
    /// true if encryption bit (bit 5, 0x20) is set
    pub fn encryption(&self) -> bool {
        (self.security_control & Self::ENCRYPTION_BIT) != 0
    }

    /// Set the encryption flag
    ///
    /// # Arguments
    /// * `encryption` - true to enable encryption
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::SecurityControl;
    ///
    /// let mut sc = SecurityControl::new(0x00);
    /// sc.set_encryption(true);
    /// assert_eq!(sc.encode(), 0x20);
    /// ```
    pub fn set_encryption(&mut self, encryption: bool) {
        if encryption {
            self.security_control |= Self::ENCRYPTION_BIT
        } else {
            self.security_control &= !Self::ENCRYPTION_BIT
        }
    }

    /// Check if this security control uses broadcast key
    ///
    /// # Returns
    /// true if broadcast bit (bit 6, 0x40) is set
    ///
    /// # Note
    /// Broadcast keys are used for encrypting messages to multiple clients,
    /// as opposed to client-specific dedicated keys.
    pub fn broadcast(&self) -> bool {
        (self.security_control & Self::BROADCAST_BIT) != 0
    }

    /// Set the broadcast flag
    ///
    /// # Arguments
    /// * `broadcast` - true to use broadcast key
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::SecurityControl;
    ///
    /// let mut sc = SecurityControl::new(0x30);
    /// sc.set_broadcast(true);
    /// assert_eq!(sc.encode(), 0x70);
    /// ```
    pub fn set_broadcast(&mut self, broadcast: bool) {
        if broadcast {
            self.security_control |= Self::BROADCAST_BIT
        } else {
            self.security_control &= !Self::BROADCAST_BIT
        }
    }

    /// Check if this security control uses compression
    ///
    /// # Returns
    /// true if compression bit (bit 7, 0x80) is set
    pub fn compression(&self) -> bool {
        (self.security_control & Self::COMPRESSION_BIT) != 0
    }

    /// Set the compression flag
    ///
    /// # Arguments
    /// * `compression` - true to enable compression
    ///
    /// # Examples
    /// ```
    /// use dlms_cosem::SecurityControl;
    ///
    /// let mut sc = SecurityControl::new(0x30);
    /// sc.set_compression(true);
    /// assert_eq!(sc.encode(), 0xB0);
    /// ```
    pub fn set_compression(&mut self, compression: bool) {
        if compression {
            self.security_control |= Self::COMPRESSION_BIT
        } else {
            self.security_control &= !Self::COMPRESSION_BIT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sc = SecurityControl::new(0x30);
        assert_eq!(sc.security_control, 0x30);
        assert!(sc.authentication());
        assert!(sc.encryption());

        let sc = SecurityControl::new(0x00);
        assert_eq!(sc.security_control, 0x00);
        assert!(!sc.authentication());
        assert!(!sc.encryption());
    }

    #[cfg(feature = "encode")]
    #[test]
    fn test_encode() {
        let sc = SecurityControl::new(0x30);
        assert_eq!(sc.encode(), 0x30);

        let sc = SecurityControl::new(0xFF);
        assert_eq!(sc.encode(), 0xFF);

        let sc = SecurityControl::new(0x00);
        assert_eq!(sc.encode(), 0x00);
    }

    #[cfg(all(feature = "parse", feature = "encode"))]
    #[test]
    fn test_encode_decode_roundtrip() {
        let original = SecurityControl::new(0x3F);
        let encoded = original.encode();
        let (_, decoded) = SecurityControl::parse(&[encoded]).unwrap();
        assert_eq!(decoded.security_control, original.security_control);
    }

    #[cfg(feature = "parse")]
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

    // --- SecuritySuite Tests ---

    #[test]
    fn test_security_suite_key_size() {
        assert_eq!(SecuritySuite::V0.key_size(), 16);
        assert_eq!(SecuritySuite::V1.key_size(), 16);
        assert_eq!(SecuritySuite::V2.key_size(), 32);
    }

    #[test]
    fn test_security_suite_from_id() {
        assert_eq!(SecuritySuite::from_id(0), Some(SecuritySuite::V0));
        assert_eq!(SecuritySuite::from_id(1), Some(SecuritySuite::V1));
        assert_eq!(SecuritySuite::from_id(2), Some(SecuritySuite::V2));
        assert_eq!(SecuritySuite::from_id(3), None);
        assert_eq!(SecuritySuite::from_id(15), None);
        assert_eq!(SecuritySuite::from_id(255), None);
    }

    #[test]
    fn test_security_suite_values() {
        assert_eq!(SecuritySuite::V0 as u8, 0);
        assert_eq!(SecuritySuite::V1 as u8, 1);
        assert_eq!(SecuritySuite::V2 as u8, 2);
    }

    // --- SecurityControl Suite Methods Tests ---

    #[test]
    fn test_with_suite() {
        // Auth + Enc with Suite V1
        let sc = SecurityControl::with_suite(true, true, SecuritySuite::V1);
        assert_eq!(sc.security_control, 0x31);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V1));

        // Auth + Enc with Suite V2
        let sc = SecurityControl::with_suite(true, true, SecuritySuite::V2);
        assert_eq!(sc.security_control, 0x32);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V2));

        // Auth only with Suite V0
        let sc = SecurityControl::with_suite(true, false, SecuritySuite::V0);
        assert_eq!(sc.security_control, 0x10);
        assert!(sc.authentication());
        assert!(!sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V0));

        // Enc only with Suite V1
        let sc = SecurityControl::with_suite(false, true, SecuritySuite::V1);
        assert_eq!(sc.security_control, 0x21);
        assert!(!sc.authentication());
        assert!(sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V1));

        // No security with Suite V2
        let sc = SecurityControl::with_suite(false, false, SecuritySuite::V2);
        assert_eq!(sc.security_control, 0x02);
        assert!(!sc.authentication());
        assert!(!sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V2));
    }

    #[test]
    fn test_suite_getter() {
        let sc = SecurityControl::new(0x30); // Suite V0
        assert_eq!(sc.suite(), Some(SecuritySuite::V0));
        assert_eq!(sc.suite_id(), 0);

        let sc = SecurityControl::new(0x31); // Suite V1
        assert_eq!(sc.suite(), Some(SecuritySuite::V1));
        assert_eq!(sc.suite_id(), 1);

        let sc = SecurityControl::new(0x32); // Suite V2
        assert_eq!(sc.suite(), Some(SecuritySuite::V2));
        assert_eq!(sc.suite_id(), 2);

        let sc = SecurityControl::new(0x3F); // Invalid suite (15)
        assert_eq!(sc.suite(), None);
        assert_eq!(sc.suite_id(), 15);
    }

    #[test]
    fn test_set_suite() {
        let mut sc = SecurityControl::new(0x30); // Auth + Enc, Suite V0
        assert_eq!(sc.suite(), Some(SecuritySuite::V0));

        // Change to Suite V1
        sc.set_suite(SecuritySuite::V1);
        assert_eq!(sc.security_control, 0x31);
        assert_eq!(sc.suite(), Some(SecuritySuite::V1));
        assert!(sc.authentication()); // Preserved
        assert!(sc.encryption()); // Preserved

        // Change to Suite V2
        sc.set_suite(SecuritySuite::V2);
        assert_eq!(sc.security_control, 0x32);
        assert_eq!(sc.suite(), Some(SecuritySuite::V2));
        assert!(sc.authentication()); // Preserved
        assert!(sc.encryption()); // Preserved

        // Test that suite change doesn't affect other bits
        let mut sc = SecurityControl::new(0xF0); // All flags, Suite V0
        sc.set_suite(SecuritySuite::V2);
        assert_eq!(sc.security_control, 0xF2);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(sc.broadcast());
        assert!(sc.compression());
        assert_eq!(sc.suite(), Some(SecuritySuite::V2));
    }

    #[test]
    fn test_key_size() {
        let sc = SecurityControl::new(0x31); // Suite V1
        assert_eq!(sc.key_size(), 16);

        let sc = SecurityControl::new(0x32); // Suite V2
        assert_eq!(sc.key_size(), 32);

        let sc = SecurityControl::new(0x30); // Suite V0
        assert_eq!(sc.key_size(), 16);

        let sc = SecurityControl::new(0x3F); // Invalid suite
        assert_eq!(sc.key_size(), 16); // Default to AES-128
    }

    #[test]
    fn test_suite_with_all_flags() {
        // Test Suite V2 with all flags enabled
        let mut sc = SecurityControl::with_suite(true, true, SecuritySuite::V2);
        sc.set_broadcast(true);
        sc.set_compression(true);

        assert_eq!(sc.security_control, 0xF2);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert!(sc.broadcast());
        assert!(sc.compression());
        assert_eq!(sc.suite(), Some(SecuritySuite::V2));
        assert_eq!(sc.key_size(), 32);
    }

    #[test]
    fn test_suite_roundtrip() {
        // Test that we can create, encode, parse, and get the same suite
        #[cfg(feature = "encode")]
        {
            let original = SecurityControl::with_suite(true, true, SecuritySuite::V2);
            let encoded = original.encode();
            assert_eq!(encoded, 0x32);
        }

        #[cfg(feature = "parse")]
        {
            let (_, parsed) = SecurityControl::parse(&[0x32]).unwrap();
            assert_eq!(parsed.suite(), Some(SecuritySuite::V2));
            assert!(parsed.authentication());
            assert!(parsed.encryption());
        }
    }

    #[test]
    fn test_gurux_compatibility_suite_bits() {
        // Verify behavior matches Gurux:
        // tag = (broadcast ? 0x40 : 0) | security | suite;

        // Suite V1, Auth+Enc
        let sc = SecurityControl::with_suite(true, true, SecuritySuite::V1);
        assert_eq!(sc.security_control, 0x31);

        // Suite V2, Auth+Enc, Broadcast
        let mut sc = SecurityControl::with_suite(true, true, SecuritySuite::V2);
        sc.set_broadcast(true);
        assert_eq!(sc.security_control, 0x72); // 0x40 | 0x30 | 0x02
    }

    #[test]
    fn test_common_security_control_values() {
        // Test common real-world security control values

        // 0x30: Auth + Enc, Suite V0 (legacy)
        let sc = SecurityControl::new(0x30);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V0));

        // 0x31: Auth + Enc, Suite V1 (most common)
        let sc = SecurityControl::new(0x31);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V1));
        assert_eq!(sc.key_size(), 16);

        // 0x32: Auth + Enc, Suite V2 (newer)
        let sc = SecurityControl::new(0x32);
        assert!(sc.authentication());
        assert!(sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V2));
        assert_eq!(sc.key_size(), 32);

        // 0x20: Enc only, Suite V0
        let sc = SecurityControl::new(0x20);
        assert!(!sc.authentication());
        assert!(sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V0));

        // 0x10: Auth only, Suite V0
        let sc = SecurityControl::new(0x10);
        assert!(sc.authentication());
        assert!(!sc.encryption());
        assert_eq!(sc.suite(), Some(SecuritySuite::V0));
    }
}
