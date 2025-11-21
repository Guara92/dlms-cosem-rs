//! Asynchronous HDLC transport wrapper for DLMS/COSEM.
//!
//! This module provides an HDLC framing wrapper that can be layered on top of
//! any asynchronous transport (TCP, Serial, etc.). It handles HDLC frame
//! encapsulation/decapsulation according to IEC 62056-46 and ISO/IEC 13239.
//!
//! See parent module [`crate::transport::hdlc`] for detailed documentation,
//! frame structure, and FCS algorithm details.

use crate::transport::r#async::AsyncTransport;

// Import shared constants and functions from parent module
use super::{
    BITS_PER_BYTE, BYTE_MASK, HDLC_ADDRESS_LSB_MASK, HDLC_CONTROL_I_FRAME, HDLC_FCS_SIZE,
    HDLC_FLAG, HDLC_FLAG_COUNT, HDLC_FORMAT_TYPE_3, HDLC_HCS_SIZE, HDLC_LLC_SIZE,
    HDLC_MAX_OVERHEAD_BYTES, HDLC_MIN_FRAME_SIZE, HdlcError, LLC_HEADER, MAX_HDLC_FRAME_SIZE,
    compute_fcs, encode_address,
};

/// Asynchronous HDLC transport wrapper.
///
/// This transport wraps any asynchronous transport and adds HDLC framing.
/// It automatically encapsulates outgoing data in HDLC frames and
/// decapsulates incoming HDLC frames.
///
/// # Type Parameters
///
/// * `T` - The underlying transport type (must implement `AsyncTransport`)
///
/// # Examples
///
/// ```no_run
/// # #[cfg(all(feature = "transport-hdlc-async", feature = "transport-tcp-async", feature = "tokio"))]
/// # {
/// use dlms_cosem::transport::hdlc::AsyncHdlcTransport;
/// use dlms_cosem::transport::tcp::AsyncTcpTransport;
///
/// # async fn example() -> std::io::Result<()> {
/// let tcp = AsyncTcpTransport::connect("192.168.1.100:4059").await?;
/// let mut hdlc = AsyncHdlcTransport::new(tcp, 0x01, 0x10);
/// # Ok(())
/// # }
/// # }
/// ```
#[derive(Debug)]
pub struct AsyncHdlcTransport<T> {
    /// The underlying transport.
    inner: T,
    /// Client address (1-4 bytes, stored as u32).
    client_address: u32,
    /// Server address (1-4 bytes, stored as u32).
    server_address: u32,
    /// Buffer for HDLC frame construction.
    frame_buffer: [u8; MAX_HDLC_FRAME_SIZE],
    /// Buffer for received data.
    recv_buffer: [u8; MAX_HDLC_FRAME_SIZE],
    /// Number of bytes currently in recv_buffer.
    recv_len: usize,
}

impl<T> AsyncHdlcTransport<T> {
    /// Creates a new async HDLC transport wrapper.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying async transport to wrap
    /// * `client_address` - HDLC client address (1-4 bytes)
    /// * `server_address` - HDLC server address (1-4 bytes)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "transport-hdlc-async", feature = "transport-tcp-async", feature = "tokio"))]
    /// # {
    /// use dlms_cosem::transport::hdlc::AsyncHdlcTransport;
    /// use dlms_cosem::transport::tcp::AsyncTcpTransport;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let tcp = AsyncTcpTransport::connect("192.168.1.100:4059").await?;
    /// let hdlc = AsyncHdlcTransport::new(tcp, 0x01, 0x10);
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub fn new(inner: T, client_address: u32, server_address: u32) -> Self {
        Self {
            inner,
            client_address,
            server_address,
            frame_buffer: [0u8; MAX_HDLC_FRAME_SIZE],
            recv_buffer: [0u8; MAX_HDLC_FRAME_SIZE],
            recv_len: 0,
        }
    }

    /// Returns a reference to the underlying transport.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the underlying transport.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consumes the HDLC transport and returns the underlying transport.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Wraps data in an HDLC frame.
    ///
    /// Returns the frame as a slice of the internal buffer and its length.
    fn wrap_hdlc(&mut self, data: &[u8]) -> Result<usize, HdlcError> {
        if data.len() > MAX_HDLC_FRAME_SIZE - HDLC_MAX_OVERHEAD_BYTES {
            // Reserve space for HDLC overhead
            return Err(HdlcError::FrameTooLarge);
        }

        let mut pos = 0;

        // Opening flag
        self.frame_buffer[pos] = HDLC_FLAG;
        pos += 1;

        // Format field (Frame Format Type 3)
        // Format: 1010yyyy where yyyy = length field size indicator
        // For simplicity, we use format 0xA0 (length in 1 byte)
        self.frame_buffer[pos] = HDLC_FORMAT_TYPE_3;
        pos += 1;

        // Length field (placeholder, will be filled later)
        let length_pos = pos;
        pos += 1;

        // Server address (destination)
        let server_addr_len = encode_address(self.server_address, &mut self.frame_buffer[pos..]);
        pos += server_addr_len;

        // Client address (source)
        let client_addr_len = encode_address(self.client_address, &mut self.frame_buffer[pos..]);
        pos += client_addr_len;

        // Control field (Information frame, no segmentation)
        // Control = 0x10 (I-frame, N(S)=0, N(R)=0, P/F=0)
        self.frame_buffer[pos] = HDLC_CONTROL_I_FRAME;
        pos += 1;

        // Calculate and insert HCS (Header Check Sequence)
        // HCS is FCS-16 over Format + Length + Addresses + Control
        let hcs = compute_fcs(&self.frame_buffer[1..pos]);
        self.frame_buffer[pos] = (hcs & BYTE_MASK) as u8;
        self.frame_buffer[pos + 1] = ((hcs >> BITS_PER_BYTE) & BYTE_MASK) as u8;
        pos += HDLC_HCS_SIZE;

        // LLC header
        self.frame_buffer[pos..pos + HDLC_LLC_SIZE].copy_from_slice(&LLC_HEADER);
        pos += HDLC_LLC_SIZE;

        // Information field (APDU data)
        self.frame_buffer[pos..pos + data.len()].copy_from_slice(data);
        pos += data.len();

        // Calculate and insert FCS (Frame Check Sequence)
        // FCS is over everything except opening flag and FCS itself
        let fcs = compute_fcs(&self.frame_buffer[1..pos]);
        self.frame_buffer[pos] = (fcs & BYTE_MASK) as u8;
        self.frame_buffer[pos + 1] = ((fcs >> BITS_PER_BYTE) & BYTE_MASK) as u8;
        pos += HDLC_FCS_SIZE;

        // Closing flag
        self.frame_buffer[pos] = HDLC_FLAG;
        pos += 1;

        // Fill in the length field
        // Length = frame size from format field to FCS (inclusive)
        let frame_length = pos - HDLC_FLAG_COUNT; // Exclude both flags
        self.frame_buffer[length_pos] = frame_length as u8;

        Ok(pos)
    }

    /// Unwraps an HDLC frame and extracts the information field.
    ///
    /// Returns the number of bytes written to the output buffer.
    fn unwrap_hdlc(&self, frame: &[u8], output: &mut [u8]) -> Result<usize, HdlcError> {
        if frame.len() < HDLC_MIN_FRAME_SIZE {
            // Minimum frame: Flag + Format + Length + Addr(2) + Ctrl + HCS(2) + FCS(2) + Flag
            return Err(HdlcError::FrameTooShort);
        }

        let mut pos = 0;

        // Check opening flag
        if frame[pos] != HDLC_FLAG {
            return Err(HdlcError::InvalidFlag);
        }
        pos += 1;

        // Skip format field
        pos += 1;

        // Read length field
        let _length = frame[pos] as usize;
        pos += 1;

        // Skip addresses and control field
        // For simplicity, we scan for HCS position (after addresses + control)
        // In production, properly parse address fields
        while pos < frame.len() - 4 && frame[pos] & HDLC_ADDRESS_LSB_MASK == 0 {
            pos += 1; // Skip address bytes (LSB = 0 means more bytes follow)
        }
        if pos < frame.len() - 4 {
            pos += 1; // Last address byte
        }

        // Skip control field
        pos += 1;

        // Skip HCS (2 bytes)
        pos += HDLC_HCS_SIZE;

        // Find information field end (before FCS)
        let fcs_pos = frame.len() - HDLC_FCS_SIZE - 1; // FCS (2 bytes) + closing flag
        let info_len = fcs_pos - pos;

        if info_len < HDLC_LLC_SIZE {
            // Must have at least LLC header
            return Err(HdlcError::InvalidFrame);
        }

        // Verify FCS
        let expected_fcs = compute_fcs(&frame[1..fcs_pos]);
        let actual_fcs = u16::from_le_bytes([frame[fcs_pos], frame[fcs_pos + 1]]);
        if expected_fcs != actual_fcs {
            return Err(HdlcError::FcsError);
        }

        // Check closing flag
        if frame[frame.len() - 1] != HDLC_FLAG {
            return Err(HdlcError::InvalidFlag);
        }

        // Skip LLC header (3 bytes)
        pos += HDLC_LLC_SIZE;
        let data_len = info_len - HDLC_LLC_SIZE;

        // Copy information field to output
        if data_len > output.len() {
            return Err(HdlcError::BufferTooSmall);
        }
        output[..data_len].copy_from_slice(&frame[pos..pos + data_len]);

        Ok(data_len)
    }
}

impl<T: AsyncTransport> AsyncTransport for AsyncHdlcTransport<T> {
    type Error = HdlcError;

    async fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let frame_len = self.wrap_hdlc(data)?;
        self.inner
            .send(&self.frame_buffer[..frame_len])
            .await
            .map_err(|_| HdlcError::TransportError)?;
        Ok(())
    }

    async fn recv(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        // Read from underlying transport
        let n =
            self.inner.recv(&mut self.recv_buffer).await.map_err(|_| HdlcError::TransportError)?;

        if n == 0 {
            return Err(HdlcError::TransportError);
        }

        self.recv_len = n;

        // Unwrap HDLC frame
        self.unwrap_hdlc(&self.recv_buffer[..n], buffer)
    }

    #[cfg(feature = "std")]
    async fn recv_timeout(
        &mut self,
        buffer: &mut [u8],
        timeout: std::time::Duration,
    ) -> Result<usize, Self::Error> {
        // Read from underlying transport with timeout
        let n = self
            .inner
            .recv_timeout(&mut self.recv_buffer, timeout)
            .await
            .map_err(|_| HdlcError::TransportError)?;

        if n == 0 {
            return Err(HdlcError::TransportError);
        }

        self.recv_len = n;

        // Unwrap HDLC frame
        self.unwrap_hdlc(&self.recv_buffer[..n], buffer)
    }
}

// HdlcError is re-exported from parent module - no need to redefine

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most tests are in parent module (hdlc.rs)
    // Only transport-specific tests here

    #[test]
    fn test_hdlc_constants_accessible() {
        // Verify we can access shared constants
        assert_eq!(HDLC_FLAG, 0x7E);
        assert_eq!(MAX_HDLC_FRAME_SIZE, 2048);
    }

    // Note: Full integration tests with mock transports are in tests/hdlc_transport_async.rs
}
