//! Conformance bitflags for DLMS/COSEM
//!
//! Conformance bits define which DLMS services are supported by client and server.
//! During association establishment, the client proposes its conformance, and the
//! server responds with the negotiated conformance (bitwise AND of client and server).
//!
//! Reference: DLMS Green Book Ed. 12, Table 133

use core::fmt;

/// Conformance bits indicating supported DLMS services
///
/// This is a 24-bit bitfield (3 bytes) that defines which services
/// are available in the association.
///
/// # Examples
///
/// ```
/// use dlms_cosem::association::Conformance;
///
/// // Typical client conformance
/// let client = Conformance::READ | Conformance::WRITE |
///              Conformance::GET | Conformance::SET | Conformance::ACTION |
///              Conformance::SELECTIVE_ACCESS | Conformance::BLOCK_TRANSFER_WITH_GET_OR_READ;
///
/// // Check if a service is supported
/// assert!(client.contains(Conformance::GET));
/// assert!(!client.contains(Conformance::MULTIPLE_REFERENCES));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Conformance {
    bits: u32,
}

impl Conformance {
    // Bit positions as defined in Green Book Table 133

    /// General protection (bit 0)
    pub const GENERAL_PROTECTION: Self = Self { bits: 0x00000001 };

    /// General block transfer (bit 1)
    pub const GENERAL_BLOCK_TRANSFER: Self = Self { bits: 0x00000002 };

    /// Read (bit 2) - Short Name referencing
    pub const READ: Self = Self { bits: 0x00000004 };

    /// Write (bit 3) - Short Name referencing
    pub const WRITE: Self = Self { bits: 0x00000008 };

    /// Unconfirmed write (bit 4)
    pub const UNCONFIRMED_WRITE: Self = Self { bits: 0x00000010 };

    /// Attribute 0 supported with SET (bit 5)
    pub const ATTRIBUTE_0_SUPPORTED_WITH_SET: Self = Self { bits: 0x00000020 };

    /// Priority management supported (bit 6)
    pub const PRIORITY_MGMT_SUPPORTED: Self = Self { bits: 0x00000040 };

    /// Attribute 0 supported with GET (bit 7)
    pub const ATTRIBUTE_0_SUPPORTED_WITH_GET: Self = Self { bits: 0x00000080 };

    /// Block transfer with GET or READ (bit 8)
    pub const BLOCK_TRANSFER_WITH_GET_OR_READ: Self = Self { bits: 0x00000100 };

    /// Block transfer with SET or WRITE (bit 9)
    pub const BLOCK_TRANSFER_WITH_SET_OR_WRITE: Self = Self { bits: 0x00000200 };

    /// Block transfer with ACTION (bit 10)
    pub const BLOCK_TRANSFER_WITH_ACTION: Self = Self { bits: 0x00000400 };

    /// Multiple references (bit 11)
    pub const MULTIPLE_REFERENCES: Self = Self { bits: 0x00000800 };

    /// Information report (bit 12)
    pub const INFORMATION_REPORT: Self = Self { bits: 0x00001000 };

    /// Data notification (bit 13)
    pub const DATA_NOTIFICATION: Self = Self { bits: 0x00002000 };

    /// Parameterized access (bit 14)
    pub const PARAMETERIZED_ACCESS: Self = Self { bits: 0x00004000 };

    /// GET (bit 15) - Logical Name referencing
    pub const GET: Self = Self { bits: 0x00008000 };

    /// SET (bit 16) - Logical Name referencing
    pub const SET: Self = Self { bits: 0x00010000 };

    /// Selective access (bit 17)
    pub const SELECTIVE_ACCESS: Self = Self { bits: 0x00020000 };

    /// Event notification (bit 18)
    pub const EVENT_NOTIFICATION: Self = Self { bits: 0x00040000 };

    /// ACTION (bit 19) - Logical Name referencing
    pub const ACTION: Self = Self { bits: 0x00080000 };

    // Reserved bits 20-23 (not used)

    /// Empty conformance (no services)
    pub const EMPTY: Self = Self { bits: 0 };

    /// All conformance bits set (for testing)
    pub const ALL: Self = Self { bits: 0x00FFFFFF }; // 24 bits (3 bytes)

    /// Typical client conformance for Logical Name referencing
    pub const TYPICAL_CLIENT_LN: Self = Self {
        bits: Self::GET.bits
            | Self::SET.bits
            | Self::ACTION.bits
            | Self::SELECTIVE_ACCESS.bits
            | Self::BLOCK_TRANSFER_WITH_GET_OR_READ.bits
            | Self::BLOCK_TRANSFER_WITH_SET_OR_WRITE.bits
            | Self::BLOCK_TRANSFER_WITH_ACTION.bits,
    };

    /// Typical client conformance for Short Name referencing
    pub const TYPICAL_CLIENT_SN: Self = Self {
        bits: Self::READ.bits
            | Self::WRITE.bits
            | Self::BLOCK_TRANSFER_WITH_GET_OR_READ.bits
            | Self::BLOCK_TRANSFER_WITH_SET_OR_WRITE.bits,
    };

    /// Create conformance from raw bits
    pub const fn from_bits(bits: u32) -> Self {
        Self { bits: bits & 0x00FFFFFF } // Mask to 24 bits (3 bytes)
    }

    /// Create conformance from raw bits without masking
    pub const fn from_bits_truncate(bits: u32) -> Self {
        Self { bits }
    }

    /// Get raw bits value
    pub const fn bits(&self) -> u32 {
        self.bits
    }

    /// Create conformance from 3-byte array (big-endian)
    ///
    /// The conformance is transmitted as 3 bytes in A-XDR encoding.
    pub const fn from_bytes(bytes: [u8; 3]) -> Self {
        let bits = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32);
        Self::from_bits(bits)
    }

    /// Convert to 3-byte array (big-endian)
    pub const fn to_bytes(self) -> [u8; 3] {
        [
            ((self.bits >> 16) & 0xFF) as u8,
            ((self.bits >> 8) & 0xFF) as u8,
            (self.bits & 0xFF) as u8,
        ]
    }

    /// Check if this conformance contains a specific flag
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Check if this conformance is empty
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    /// Bitwise OR of two conformance values
    pub const fn union(self, other: Self) -> Self {
        Self { bits: self.bits | other.bits }
    }

    /// Bitwise AND of two conformance values (intersection)
    ///
    /// This is used during association negotiation to find common services.
    pub const fn intersection(self, other: Self) -> Self {
        Self { bits: self.bits & other.bits }
    }

    /// Bitwise XOR of two conformance values
    pub const fn symmetric_difference(self, other: Self) -> Self {
        Self { bits: self.bits ^ other.bits }
    }

    /// Remove flags from conformance
    pub const fn difference(self, other: Self) -> Self {
        Self { bits: self.bits & !other.bits }
    }
}

impl core::ops::BitOr for Conformance {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

impl core::ops::BitOrAssign for Conformance {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

impl core::ops::BitAnd for Conformance {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}

impl core::ops::BitAndAssign for Conformance {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}

impl core::ops::BitXor for Conformance {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        self.symmetric_difference(rhs)
    }
}

impl core::ops::BitXorAssign for Conformance {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.symmetric_difference(rhs);
    }
}

impl core::ops::Not for Conformance {
    type Output = Self;

    fn not(self) -> Self {
        Self::from_bits(!self.bits)
    }
}

impl fmt::Debug for Conformance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Conformance").field("bits", &format_args!("0x{:06X}", self.bits)).finish()
    }
}

impl fmt::Display for Conformance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Conformance(0x{:06X})", self.bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conformance_constants() {
        assert_eq!(Conformance::GENERAL_PROTECTION.bits(), 0x00000001);
        assert_eq!(Conformance::GET.bits(), 0x00008000);
        assert_eq!(Conformance::ACTION.bits(), 0x00080000);
    }

    #[test]
    fn test_conformance_bytes() {
        let conf = Conformance::from_bits(0x001F8000);
        let bytes = conf.to_bytes();
        assert_eq!(bytes, [0x1F, 0x80, 0x00]);

        let conf2 = Conformance::from_bytes([0x1F, 0x80, 0x00]);
        assert_eq!(conf2.bits(), 0x001F8000);
    }

    #[test]
    fn test_conformance_contains() {
        let conf = Conformance::GET | Conformance::SET | Conformance::ACTION;
        assert!(conf.contains(Conformance::GET));
        assert!(conf.contains(Conformance::SET));
        assert!(conf.contains(Conformance::ACTION));
        assert!(!conf.contains(Conformance::READ));
    }

    #[test]
    fn test_conformance_union() {
        let a = Conformance::GET | Conformance::SET;
        let b = Conformance::ACTION | Conformance::SELECTIVE_ACCESS;
        let c = a | b;

        assert!(c.contains(Conformance::GET));
        assert!(c.contains(Conformance::SET));
        assert!(c.contains(Conformance::ACTION));
        assert!(c.contains(Conformance::SELECTIVE_ACCESS));
    }

    #[test]
    fn test_conformance_intersection() {
        let client = Conformance::GET | Conformance::SET | Conformance::ACTION;
        let server = Conformance::GET | Conformance::ACTION | Conformance::READ;
        let negotiated = client & server;

        assert!(negotiated.contains(Conformance::GET));
        assert!(!negotiated.contains(Conformance::SET));
        assert!(negotiated.contains(Conformance::ACTION));
        assert!(!negotiated.contains(Conformance::READ));
    }

    #[test]
    fn test_typical_client_ln() {
        let conf = Conformance::TYPICAL_CLIENT_LN;
        assert!(conf.contains(Conformance::GET));
        assert!(conf.contains(Conformance::SET));
        assert!(conf.contains(Conformance::ACTION));
        assert!(conf.contains(Conformance::SELECTIVE_ACCESS));
        assert!(!conf.contains(Conformance::READ)); // SN-specific
        assert!(!conf.contains(Conformance::WRITE)); // SN-specific
    }

    #[test]
    fn test_typical_client_sn() {
        let conf = Conformance::TYPICAL_CLIENT_SN;
        assert!(conf.contains(Conformance::READ));
        assert!(conf.contains(Conformance::WRITE));
        assert!(!conf.contains(Conformance::GET)); // LN-specific
        assert!(!conf.contains(Conformance::SET)); // LN-specific
    }

    #[test]
    fn test_conformance_empty() {
        let conf = Conformance::EMPTY;
        assert!(conf.is_empty());
        assert_eq!(conf.bits(), 0);
    }

    #[test]
    fn test_conformance_bitops() {
        let mut conf = Conformance::EMPTY;
        conf |= Conformance::GET;
        assert!(conf.contains(Conformance::GET));

        conf &= Conformance::GET | Conformance::SET;
        assert!(conf.contains(Conformance::GET));
        assert!(!conf.contains(Conformance::SET));

        let conf2 = Conformance::GET ^ Conformance::SET;
        assert!(conf2.contains(Conformance::GET));
        assert!(conf2.contains(Conformance::SET));
    }

    #[test]
    fn test_conformance_difference() {
        let a = Conformance::GET | Conformance::SET | Conformance::ACTION;
        let b = Conformance::SET;
        let c = a.difference(b);

        assert!(c.contains(Conformance::GET));
        assert!(!c.contains(Conformance::SET));
        assert!(c.contains(Conformance::ACTION));
    }
}
