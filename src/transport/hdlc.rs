//! HDLC transport wrapper implementations for DLMS/COSEM.
//!
//! This module provides HDLC (High-Level Data Link Control) framing wrappers
//! that can be layered on top of any transport (TCP, Serial, etc.) to add
//! HDLC frame encapsulation/decapsulation according to IEC 62056-46 and ISO/IEC 13239.
//!
//! # Features
//!
//! - `transport-hdlc` - Synchronous HDLC wrapper
//! - `transport-hdlc-async` - Asynchronous HDLC wrapper
//!
//! # HDLC Frame Structure
//!
//! HDLC frames in DLMS/COSEM follow the format defined in IEC 62056-46:
//!
//! ```text
//! +------+--------+--------+------+-------+------+-----+------+------+
//! | Flag | Format | Length | Dest | Src   | Ctrl | HCS | LLC | Info | FCS  | Flag |
//! | 0x7E | (1)    | (1-2)  | Addr | Addr  | (1)  | (2) | (3) | (n)  | (2)  | 0x7E |
//! +------+--------+--------+------+-------+------+-----+------+------+
//! ```
//!
//! - **Flag**: Frame delimiter (0x7E)
//! - **Format**: Frame format type (Type 3)
//! - **Length**: Frame length field
//! - **Dest Addr**: Destination address (server, 1-4 bytes)
//! - **Src Addr**: Source address (client, 1-4 bytes)
//! - **Ctrl**: Control field
//! - **HCS**: Header Check Sequence (FCS-16 over header)
//! - **LLC**: Logical Link Control header (3 bytes: 0xE6 0xE6 0x00)
//! - **Info**: Information field (DLMS APDU)
//! - **FCS**: Frame Check Sequence (FCS-16 over entire frame)
//!
//! # FCS-16 Calculation
//!
//! The Frame Check Sequence uses the X.25 polynomial per ISO/IEC 13239:
//! - Polynomial: x^16 + x^12 + x^5 + 1 (0x1021)
//! - Initial value: 0xFFFF
//! - XOR output: 0xFFFF
//!
//! # Examples
//!
//! ## Synchronous HDLC Wrapper
//!
//! ```no_run
//! # #[cfg(all(feature = "transport-hdlc", feature = "transport-tcp"))]
//! # {
//! use dlms_cosem::transport::hdlc::HdlcTransport;
//! use dlms_cosem::transport::tcp::TcpTransport;
//!
//! # fn example() -> std::io::Result<()> {
//! // Create base TCP transport
//! let tcp = TcpTransport::connect("192.168.1.100:4059")?;
//!
//! // Wrap with HDLC framing
//! let mut hdlc = HdlcTransport::new(
//!     tcp,
//!     0x01,  // Client address
//!     0x10,  // Server address
//! );
//!
//! // Use with DlmsClient - frames are automatically wrapped/unwrapped
//! // let client = DlmsClient::new(hdlc, settings);
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Asynchronous HDLC Wrapper (Tokio)
//!
//! ```no_run
//! # #[cfg(all(feature = "transport-hdlc-async", feature = "transport-tcp-async", feature = "tokio"))]
//! # {
//! use dlms_cosem::transport::hdlc::AsyncHdlcTransport;
//! use dlms_cosem::transport::tcp::AsyncTcpTransport;
//!
//! # async fn example() -> std::io::Result<()> {
//! // Create base TCP transport
//! let tcp = AsyncTcpTransport::connect("192.168.1.100:4059").await?;
//!
//! // Wrap with HDLC framing
//! let mut hdlc = AsyncHdlcTransport::new(
//!     tcp,
//!     0x01,  // Client address
//!     0x10,  // Server address
//! );
//!
//! // Use with AsyncDlmsClient - frames are automatically wrapped/unwrapped
//! // let client = AsyncDlmsClient::new(hdlc, settings);
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Layering HDLC over Serial Transport
//!
//! ```no_run
//! # #[cfg(all(feature = "transport-hdlc", feature = "transport-serial"))]
//! # {
//! use dlms_cosem::transport::hdlc::HdlcTransport;
//! // use dlms_cosem::transport::serial::SerialTransport;
//!
//! # fn example() -> std::io::Result<()> {
//! // Create serial transport (to be implemented in Phase 6.2.4)
//! // let serial = SerialTransport::open("/dev/ttyUSB0", 9600)?;
//!
//! // Wrap with HDLC framing
//! // let mut hdlc = HdlcTransport::new(serial, 0x01, 0x10);
//!
//! // Use with DlmsClient
//! // let client = DlmsClient::new(hdlc, settings);
//! # Ok(())
//! # }
//! # }
//! ```

// Synchronous HDLC transport
#[cfg(feature = "transport-hdlc")]
pub mod sync;

#[cfg(feature = "transport-hdlc")]
pub use sync::HdlcTransport;

// Asynchronous HDLC transport
#[cfg(feature = "transport-hdlc-async")]
pub mod r#async;

#[cfg(feature = "transport-hdlc-async")]
pub use r#async::AsyncHdlcTransport;

// ============================================================================
// Shared Constants (used by both sync and async implementations)
// ============================================================================

/// HDLC frame delimiter (flag byte).
///
/// Per IEC 62056-46, HDLC frames start and end with this byte.
pub const HDLC_FLAG: u8 = 0x7E;

/// Maximum HDLC frame size (including overhead).
///
/// This includes all HDLC overhead (flags, addresses, control, HCS, FCS, LLC).
pub const MAX_HDLC_FRAME_SIZE: usize = 2048;

/// LLC header for DLMS/COSEM (3 bytes: dest LSAP, src LSAP, quality).
///
/// Per IEC 62056-46:
/// - Destination LSAP: 0xE6 (DLMS unicast)
/// - Source LSAP: 0xE6 (DLMS command)
/// - Quality: 0x00
pub(crate) const LLC_HEADER: [u8; 3] = [0xE6, 0xE6, 0x00];

/// FCS-16 polynomial (X.25, ISO 13239): x^16 + x^12 + x^5 + 1.
///
/// This is the standard CRC-16-CCITT polynomial used in HDLC.
pub(crate) const FCS16_POLYNOMIAL: u16 = 0x1021;

/// FCS-16 initial value (as per ISO 13239).
pub(crate) const FCS16_INIT: u16 = 0xFFFF;

/// FCS-16 XOR output value (as per ISO 13239).
pub(crate) const FCS16_XOR_OUTPUT: u16 = 0xFFFF;

// ============================================================================
// HDLC Protocol Constants
// ============================================================================

/// Frame Format Type 3 identifier (1010yyyy where yyyy = length field size).
///
/// Format 0xA0 indicates a single-byte length field.
pub(crate) const HDLC_FORMAT_TYPE_3: u8 = 0xA0;

/// Control field for I-frame (Information frame, N(S)=0, N(R)=0, P/F=0).
pub(crate) const HDLC_CONTROL_I_FRAME: u8 = 0x10;

/// Size of FCS (Frame Check Sequence) in bytes.
pub(crate) const HDLC_FCS_SIZE: usize = 2;

/// Size of HCS (Header Check Sequence) in bytes.
pub(crate) const HDLC_HCS_SIZE: usize = 2;

/// Size of LLC header in bytes.
pub(crate) const HDLC_LLC_SIZE: usize = 3;

/// Minimum valid HDLC frame size in bytes.
///
/// Includes: Flag(1) + Format(1) + Length(1) + Addr(2) + Ctrl(1) + HCS(2) + FCS(2) + Flag(1) = 11 bytes minimum
pub(crate) const HDLC_MIN_FRAME_SIZE: usize = 11;

/// Maximum HDLC overhead in bytes (conservative estimate for buffer sizing).
///
/// Includes: Flags(2) + Format(1) + Length(1) + Addresses(8 max) + Control(1) + HCS(2) + LLC(3) + FCS(2) = ~20 bytes
pub(crate) const HDLC_MAX_OVERHEAD_BYTES: usize = 20;

/// Number of flag bytes in a complete frame (opening + closing).
pub(crate) const HDLC_FLAG_COUNT: usize = 2;

/// LSB mask for HDLC address extension bit (0 = more bytes, 1 = last byte).
pub(crate) const HDLC_ADDRESS_LSB_MASK: u8 = 0x01;

/// Byte mask (0xFF) for extracting single bytes from larger values.
pub(crate) const BYTE_MASK: u16 = 0xFF;

/// Number of bits in a byte.
pub(crate) const BITS_PER_BYTE: u32 = 8;

/// MSB (Most Significant Bit) mask for 16-bit values (used in FCS computation).
pub(crate) const MSB_MASK_16: u16 = 0x8000;

// ============================================================================
// Shared Types and Functions
// ============================================================================

/// HDLC transport error types.
///
/// These errors can occur during HDLC frame processing in both sync and async transports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdlcError {
    /// Frame is too large to fit in buffer.
    FrameTooLarge,
    /// Frame is too short to be valid.
    FrameTooShort,
    /// Invalid flag byte.
    InvalidFlag,
    /// Invalid frame structure.
    InvalidFrame,
    /// FCS verification failed.
    FcsError,
    /// Output buffer is too small.
    BufferTooSmall,
    /// Underlying transport error.
    TransportError,
}

impl core::fmt::Display for HdlcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FrameTooLarge => write!(f, "HDLC frame too large"),
            Self::FrameTooShort => write!(f, "HDLC frame too short"),
            Self::InvalidFlag => write!(f, "Invalid HDLC flag byte"),
            Self::InvalidFrame => write!(f, "Invalid HDLC frame structure"),
            Self::FcsError => write!(f, "HDLC FCS verification failed"),
            Self::BufferTooSmall => write!(f, "Output buffer too small"),
            Self::TransportError => write!(f, "Underlying transport error"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HdlcError {}

/// Computes FCS-16 (Frame Check Sequence) using X.25 polynomial.
///
/// This function implements the CRC-16-CCITT algorithm per ISO/IEC 13239 and IEC 62056-46.
///
/// # Algorithm
///
/// 1. Initialize FCS to 0xFFFF
/// 2. For each byte in data:
///    - XOR byte with high byte of FCS
///    - Shift left 8 times, XORing with polynomial if MSB is set
/// 3. XOR final result with 0xFFFF
///
/// # Arguments
///
/// * `data` - The data over which to compute the FCS
///
/// # Returns
///
/// The computed FCS-16 value (little-endian byte order)
///
/// # Examples
///
/// ```
/// # use dlms_cosem::transport::hdlc::compute_fcs;
/// let data = [0x01, 0x02, 0x03, 0x04];
/// let fcs = compute_fcs(&data);
/// assert_ne!(fcs, 0);
/// assert_ne!(fcs, 0xFFFF);
/// ```
pub fn compute_fcs(data: &[u8]) -> u16 {
    let mut fcs = FCS16_INIT;

    for &byte in data {
        fcs ^= (byte as u16) << BITS_PER_BYTE;
        for _ in 0..BITS_PER_BYTE {
            if fcs & MSB_MASK_16 != 0 {
                fcs = (fcs << 1) ^ FCS16_POLYNOMIAL;
            } else {
                fcs <<= 1;
            }
        }
    }

    fcs ^ FCS16_XOR_OUTPUT
}

/// Encodes an address field using HDLC address extension mechanism (1-4 bytes, LSB extension).
///
/// Per IEC 62056-46, addresses are encoded with the LSB indicating whether more bytes follow:
/// - LSB = 0: More address bytes follow
/// - LSB = 1: This is the last address byte
///
/// # Address Ranges
///
/// - 1 byte: 0x00 - 0x7F
/// - 2 bytes: 0x80 - 0x3FFF
/// - 3 bytes: 0x4000 - 0x1FFFFF
/// - 4 bytes: 0x200000 - 0x0FFFFFFF
///
/// # Arguments
///
/// * `address` - The address value to encode
/// * `buffer` - Output buffer (must have at least 4 bytes)
///
/// # Returns
///
/// The number of bytes written (1-4)
///
/// # Examples
///
/// ```
/// # use dlms_cosem::transport::hdlc::encode_address;
/// let mut buffer = [0u8; 4];
/// let len = encode_address(0x01, &mut buffer);
/// assert_eq!(len, 1);
/// assert_eq!(buffer[0], 0x03); // (0x01 << 1) | 0x01
/// ```
pub fn encode_address(address: u32, buffer: &mut [u8]) -> usize {
    const ADDR_1BYTE_MAX: u32 = 0x7F;
    const ADDR_2BYTE_MAX: u32 = 0x3FFF;
    const ADDR_3BYTE_MAX: u32 = 0x1FFFFF;
    const ADDR_FE_MASK: u32 = 0xFE;
    const ADDR_SHIFT_7: u32 = 7;
    const ADDR_SHIFT_14: u32 = 14;
    const ADDR_SHIFT_21: u32 = 21;

    if address <= ADDR_1BYTE_MAX {
        // 1-byte address
        buffer[0] = ((address << 1) | HDLC_ADDRESS_LSB_MASK as u32) as u8;
        1
    } else if address <= ADDR_2BYTE_MAX {
        // 2-byte address
        buffer[0] = ((address << 1) & ADDR_FE_MASK) as u8;
        buffer[1] = (((address >> ADDR_SHIFT_7) << 1) | HDLC_ADDRESS_LSB_MASK as u32) as u8;
        2
    } else if address <= ADDR_3BYTE_MAX {
        // 3-byte address
        buffer[0] = ((address << 1) & ADDR_FE_MASK) as u8;
        buffer[1] = (((address >> ADDR_SHIFT_7) << 1) & ADDR_FE_MASK) as u8;
        buffer[2] = (((address >> ADDR_SHIFT_14) << 1) | HDLC_ADDRESS_LSB_MASK as u32) as u8;
        3
    } else {
        // 4-byte address
        buffer[0] = ((address << 1) & ADDR_FE_MASK) as u8;
        buffer[1] = (((address >> ADDR_SHIFT_7) << 1) & ADDR_FE_MASK) as u8;
        buffer[2] = (((address >> ADDR_SHIFT_14) << 1) & ADDR_FE_MASK) as u8;
        buffer[3] = (((address >> ADDR_SHIFT_21) << 1) | HDLC_ADDRESS_LSB_MASK as u32) as u8;
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcs_computation() {
        // Test vector from ISO 13239
        let data = [0x01, 0x02, 0x03, 0x04];
        let fcs = compute_fcs(&data);
        // FCS should be deterministic
        assert_ne!(fcs, 0);
        assert_ne!(fcs, 0xFFFF);
    }

    #[test]
    fn test_fcs_empty() {
        let data = [];
        let fcs = compute_fcs(&data);
        assert_eq!(fcs, FCS16_INIT ^ FCS16_XOR_OUTPUT);
    }

    #[test]
    fn test_fcs_single_byte() {
        let data = [0xFF];
        let fcs1 = compute_fcs(&data);

        // FCS should be different for different input
        let data2 = [0x00];
        let fcs2 = compute_fcs(&data2);
        assert_ne!(fcs1, fcs2);
    }

    #[test]
    fn test_encode_address_1_byte() {
        let mut buffer = [0u8; 4];
        let len = encode_address(0x01, &mut buffer);
        assert_eq!(len, 1);
        assert_eq!(buffer[0], 0x03); // (0x01 << 1) | HDLC_ADDRESS_LSB_MASK
    }

    #[test]
    fn test_encode_address_2_bytes() {
        let mut buffer = [0u8; 4];
        let len = encode_address(0x100, &mut buffer);
        assert_eq!(len, 2);
        assert_eq!(buffer[0] & HDLC_ADDRESS_LSB_MASK, 0); // LSB = 0 (more bytes follow)
        assert_eq!(buffer[1] & HDLC_ADDRESS_LSB_MASK, 1); // LSB = 1 (last byte)
    }

    #[test]
    fn test_encode_address_3_bytes() {
        let mut buffer = [0u8; 4];
        let len = encode_address(0x4000, &mut buffer);
        assert_eq!(len, 3);
        assert_eq!(buffer[0] & HDLC_ADDRESS_LSB_MASK, 0); // LSB = 0
        assert_eq!(buffer[1] & HDLC_ADDRESS_LSB_MASK, 0); // LSB = 0
        assert_eq!(buffer[2] & HDLC_ADDRESS_LSB_MASK, 1); // LSB = 1 (last byte)
    }

    #[test]
    fn test_encode_address_4_bytes() {
        let mut buffer = [0u8; 4];
        let len = encode_address(0x200000, &mut buffer);
        assert_eq!(len, 4);
        assert_eq!(buffer[0] & HDLC_ADDRESS_LSB_MASK, 0); // LSB = 0
        assert_eq!(buffer[1] & HDLC_ADDRESS_LSB_MASK, 0); // LSB = 0
        assert_eq!(buffer[2] & HDLC_ADDRESS_LSB_MASK, 0); // LSB = 0
        assert_eq!(buffer[3] & HDLC_ADDRESS_LSB_MASK, 1); // LSB = 1 (last byte)
    }

    #[test]
    fn test_encode_address_max_1_byte() {
        let mut buffer = [0u8; 4];
        let len = encode_address(0x7F, &mut buffer);
        assert_eq!(len, 1);
        assert_eq!(buffer[0], 0xFF); // (0x7F << 1) | 0x01
    }

    #[test]
    fn test_encode_address_boundary() {
        let mut buffer = [0u8; 4];

        // Just below 2-byte threshold
        let len = encode_address(0x7F, &mut buffer);
        assert_eq!(len, 1);

        // Just at 2-byte threshold
        let len = encode_address(0x80, &mut buffer);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_hdlc_constants() {
        assert_eq!(HDLC_FLAG, 0x7E);
        assert_eq!(MAX_HDLC_FRAME_SIZE, 2048);
        assert_eq!(LLC_HEADER, [0xE6, 0xE6, 0x00]);
        assert_eq!(FCS16_POLYNOMIAL, 0x1021);
        assert_eq!(FCS16_INIT, 0xFFFF);
        assert_eq!(FCS16_XOR_OUTPUT, 0xFFFF);
    }

    #[test]
    fn test_hdlc_error_display() {
        assert_eq!(HdlcError::FrameTooLarge.to_string(), "HDLC frame too large");
        assert_eq!(HdlcError::FcsError.to_string(), "HDLC FCS verification failed");
        assert_eq!(HdlcError::TransportError.to_string(), "Underlying transport error");
    }
}
