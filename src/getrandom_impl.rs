//! Custom getrandom implementation for embedded targets
//!
//! This module provides RNG support for bare-metal embedded targets when encryption
//! is needed (AES-GCM in DLMS protocol).
//!
//! # Feature Flag: `unsafe-rng`
//!
//! - **With `unsafe-rng`**: Uses a simple PRNG for testing/development (NOT cryptographically secure!)
//! - **Without `unsafe-rng`**: You MUST provide your own `getrandom_custom` implementation
//!
//! # Platform-Specific Implementation Required (Production)
//!
//! For production embedded deployments, **DO NOT** enable `unsafe-rng`. Instead,
//! implement `getrandom_custom` with hardware RNG for your target platform:
//!
//! - **STM32**: Use hardware RNG peripheral (RNG or TRNG)
//! - **nRF52/nRF53**: Use RNG peripheral
//! - **ESP32**: Use hardware RNG
//! - **RP2040**: Use ROSC-based RNG
//!
//! # Security Warning
//!
//! The `unsafe-rng` feature provides a simple PRNG that is NOT cryptographically secure
//! and should NEVER be used in production for:
//! - Key generation
//! - Nonce generation for encryption
//! - Security-critical random values
//!
//! # Example: STM32 Hardware RNG
//!
//! ```ignore
//! use stm32f4xx_hal::rng::Rng;
//!
//! static mut RNG: Option<Rng> = None;
//!
//! #[no_mangle]
//! fn getrandom_custom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
//!     unsafe {
//!         if let Some(rng) = RNG.as_mut() {
//!             for byte in buf.iter_mut() {
//!                 *byte = rng.gen().map_err(|_| getrandom::Error::UNAVAILABLE)?;
//!             }
//!             Ok(())
//!         } else {
//!             Err(getrandom::Error::UNAVAILABLE)
//!         }
//!     }
//! }
//! ```
//!
//! # Example: nRF52 Hardware RNG
//!
//! ```ignore
//! use nrf52840_hal::rng::Rng;
//!
//! static mut RNG: Option<Rng> = None;
//!
//! #[no_mangle]
//! fn getrandom_custom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
//!     unsafe {
//!         if let Some(rng) = RNG.as_mut() {
//!             rng.random_slice(buf);
//!             Ok(())
//!         } else {
//!             Err(getrandom::Error::UNAVAILABLE)
//!         }
//!     }
//! }
//! ```

#![cfg(all(target_arch = "arm", target_os = "none"))]

// ============================================================================
// UNSAFE PRNG Implementation (feature = "unsafe-rng")
// ============================================================================

#[cfg(feature = "unsafe-rng")]
use core::sync::atomic::{AtomicU32, Ordering};

/// Simple PRNG state for testing/development
/// WARNING: This is NOT cryptographically secure!
#[cfg(feature = "unsafe-rng")]
static PRNG_STATE: AtomicU32 = AtomicU32::new(0x12345678);

/// UNSAFE PRNG implementation for testing/development
///
/// # ⚠️ SECURITY WARNING ⚠️
///
/// This implementation is enabled by the `unsafe-rng` feature and uses a simple
/// PRNG that is NOT cryptographically secure. It should ONLY be used for:
/// - Development and testing on embedded targets without hardware RNG
/// - Non-production environments
///
/// # Production Use
///
/// For production, **disable the `unsafe-rng` feature** and provide your own
/// `getrandom_custom` implementation using hardware RNG. See examples below.
#[cfg(feature = "unsafe-rng")]
#[unsafe(no_mangle)]
pub fn getrandom_custom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
    for byte in buf.iter_mut() {
        // Simple xorshift PRNG (NOT cryptographically secure!)
        let mut state = PRNG_STATE.load(Ordering::Relaxed);
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        PRNG_STATE.store(state, Ordering::Relaxed);
        *byte = (state & 0xFF) as u8;
    }
    Ok(())
}

/// Initialize PRNG with a seed (only available with `unsafe-rng` feature)
///
/// # Arguments
///
/// * `seed` - Initial seed value (ideally from hardware entropy source or unique device ID)
///
/// # Example
///
/// ```ignore
/// // During board initialization, seed with device-specific value
/// let seed = read_unique_device_id();
/// dlms_cosem::getrandom_impl::init_prng_seed(seed);
/// ```
#[cfg(feature = "unsafe-rng")]
pub fn init_prng_seed(seed: u32) {
    PRNG_STATE.store(seed, Ordering::Relaxed);
}

// ============================================================================
// Production Implementation Guide (without "unsafe-rng" feature)
// ============================================================================

/// When `unsafe-rng` is NOT enabled, you must provide your own `getrandom_custom`
/// implementation. The function signature must be exactly:
///
/// ```ignore
/// #[no_mangle]
/// pub fn getrandom_custom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
///     // Your hardware RNG implementation here
/// }
/// ```
///
/// This documentation is provided as a guide - remove this and implement the
/// function in your application code or a custom HAL integration.
#[cfg(not(feature = "unsafe-rng"))]
const _: () = {
    // This is just documentation - the actual getrandom_custom must be provided
    // by the user in their application code when unsafe-rng is disabled
};

#[cfg(test)]
#[cfg(feature = "unsafe-rng")]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    fn test_getrandom_fills_buffer() {
        let mut buf = [0u8; 32];
        assert!(getrandom_custom(&mut buf).is_ok());

        // Check that buffer was filled (not all zeros)
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    #[cfg(all(target_arch = "arm", target_os = "none"))]
    fn test_init_prng_seed() {
        init_prng_seed(0xDEADBEEF);

        let mut buf1 = [0u8; 16];
        let mut buf2 = [0u8; 16];

        // Reset seed
        init_prng_seed(0xDEADBEEF);
        assert!(getrandom_custom(&mut buf1).is_ok());

        // Same seed should produce same sequence
        init_prng_seed(0xDEADBEEF);
        assert!(getrandom_custom(&mut buf2).is_ok());

        assert_eq!(buf1, buf2);
    }
}
