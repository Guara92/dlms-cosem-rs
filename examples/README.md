# DLMS/COSEM Rust Client - Examples

This directory contains comprehensive examples demonstrating how to use the `dlms_cosem` library for various scenarios, from basic synchronous operations to advanced async patterns.

## 📚 Overview

The examples are organized by complexity and use case:

### Synchronous Examples
- **[basic_client.rs](basic_client.rs)** - Basic sync client usage (connect, read, write, method, disconnect)
- **[tcp_transport_sync.rs](tcp_transport_sync.rs)** - TCP transport usage and configuration
- **[tcp_hdlc_transport_sync.rs](tcp_hdlc_transport_sync.rs)** - Layered TCP + HDLC transport

### Asynchronous Examples
- **[async_basic_tokio.rs](async_basic_tokio.rs)** - Basic async client API demonstration with Tokio runtime
- **[tcp_transport_async_tokio.rs](tcp_transport_async_tokio.rs)** - Async TCP transport with Tokio runtime
- **[tcp_hdlc_transport_async_tokio.rs](tcp_hdlc_transport_async_tokio.rs)** - Async TCP + HDLC transport with Tokio
- **[tcp_transport_async_smol.rs](tcp_transport_async_smol.rs)** - Async TCP transport with Smol runtime (lightweight)
- **[tcp_hdlc_transport_async_smol.rs](tcp_hdlc_transport_async_smol.rs)** - Async TCP + HDLC transport with Smol
- **[tcp_transport_async_glommio.rs](tcp_transport_async_glommio.rs)** - Async TCP transport with Glommio runtime (Linux io_uring) ✨ **NEW**
- **[tcp_transport_async_embassy.rs](tcp_transport_async_embassy.rs)** - Async TCP transport with Embassy runtime (embedded-first) ✨ **NEW**

## 🚀 Quick Start

### Running Examples

Each example requires specific feature flags. Use the commands below:

#### Synchronous Client
```bash
# Basic sync client
cargo run --example basic_client --features client,encode,parse

# TCP transport example
cargo run --example tcp_transport_sync --features client,transport-tcp

# TCP + HDLC layered transport
cargo run --example tcp_hdlc_transport_sync --features client,transport-tcp,transport-hdlc
```

#### Asynchronous Client (Tokio)
```bash
# Basic async with Tokio (API demonstration)
cargo run --example async_basic_tokio --features async-client,tokio,encode,parse

# Async TCP transport with Tokio
cargo run --example tcp_transport_async_tokio --features async-client,transport-tcp-async,tokio,encode,parse

# Async TCP + HDLC with Tokio
cargo run --example tcp_hdlc_transport_async_tokio --features async-client,transport-tcp-async,transport-hdlc-async,tokio,encode,parse
```

#### Asynchronous Client (Smol)
```bash
# Async TCP transport with Smol (lightweight runtime)
cargo run --example tcp_transport_async_smol --features async-client,transport-tcp-async,smol,encode,parse

# Async TCP + HDLC with Smol
cargo run --example tcp_hdlc_transport_async_smol --features async-client,transport-tcp-async,transport-hdlc-async,smol,encode,parse
```

#### Asynchronous Client (Glommio) ✨ **NEW**
```bash
# Async TCP transport with Glommio (Linux io_uring, ultra-low latency)
# Platform requirement: Linux kernel 5.8+ with io_uring support
cargo run --example tcp_transport_async_glommio --features async-client,transport-tcp-async,glommio,encode,parse
```

#### Asynchronous Client (Embassy) ✨ **NEW**
```bash
# Async TCP transport with Embassy (embedded-first runtime)
cargo run --example tcp_transport_async_embassy --features async-client,transport-tcp-async,embassy,encode,parse
```

### Using All Features
```bash
# Run any example with all features enabled
cargo run --example <example_name> --all-features
```

## 📋 Example Details

### 1. Basic Client (Sync)
**File**: `basic_client.rs`  
**Features**: `client`, `encode`, `parse`  
**Runtime**: None (synchronous)

Demonstrates:
- Creating a sync DLMS client
- Connecting to a server (AARQ/AARE)
- Reading a register value (GET)
- Writing a data object (SET)
- Invoking a method (ACTION)
- Disconnecting properly (RLRQ/RLRE)

**Use case**: Simple applications, embedded systems, command-line tools

---

### 2. TCP Transport (Sync)
**File**: `tcp_transport_sync.rs`  
**Features**: `client`, `transport-tcp`  
**Runtime**: None (synchronous)

Demonstrates:
- Creating a TCP transport connection
- Configuring read/write timeouts
- Using TCP transport with DLMS client
- Connection info (local/peer addresses)
- Proper error handling for network operations

**Use case**: Direct TCP connections to DLMS devices, HES systems, network meters

---

### 3. TCP + HDLC Transport (Sync)
**File**: `tcp_hdlc_transport_sync.rs`  
**Features**: `client`, `transport-tcp`, `transport-hdlc`  
**Runtime**: None (synchronous)

Demonstrates:
- Layering HDLC framing over TCP transport
- HDLC address configuration (client/server)
- Automatic frame encapsulation/decapsulation
- Transport layer composition pattern
- HDLC frame structure (flags, FCS, LLC header)

**Use case**: HDLC over TCP, legacy systems requiring HDLC framing, serial-over-IP gateways

**Key Concept**: This shows the composability of the transport layer. You can layer transports:
```
DLMS Client → HDLC Wrapper → TCP Transport → Network
```

---

### 4. Async TCP Transport (Tokio)
**File**: `tcp_transport_async_tokio.rs`  
**Features**: `async-client`, `transport-tcp-async`, `tokio`, `encode`, `parse`  
**Runtime**: Tokio

Demonstrates:
- Creating an async TCP transport with Tokio runtime
- Configuring async timeouts
- Using async TCP transport with async DLMS client
- Async/await patterns for non-blocking I/O
- Connection info retrieval

**Use case**: High-concurrency HES systems, SaaS platforms, web services (10,000+ concurrent connections)

---

### 5. Async TCP + HDLC Transport (Tokio)
**File**: `tcp_hdlc_transport_async_tokio.rs`  
**Features**: `async-client`, `transport-tcp-async`, `transport-hdlc-async`, `tokio`, `encode`, `parse`  
**Runtime**: Tokio

Demonstrates:
- Layering async HDLC framing over async TCP transport
- Async frame encapsulation/decapsulation
- Composable async transport architecture
- Non-blocking HDLC frame processing
- Automatic FCS validation in async context

**Use case**: Async HDLC over TCP, serial-over-IP gateways with high load, distributed meter management

---

### 6. Async TCP Transport (Smol)
**File**: `tcp_transport_async_smol.rs`  
**Features**: `async-client`, `transport-tcp-async`, `smol`, `encode`, `parse`  
**Runtime**: Smol

Demonstrates:
- Creating an async TCP transport with Smol runtime
- Lightweight async runtime for embedded systems
- Simple `smol::block_on` API for async code
- Resource-efficient async I/O
- Perfect for concentrators (100-1,000 meters)

**Use case**: Embedded Linux concentrators, edge devices, resource-constrained systems (<100MB RAM)

---

### 7. Async TCP + HDLC Transport (Smol)
**File**: `tcp_hdlc_transport_async_smol.rs`  
**Features**: `async-client`, `transport-tcp-async`, `transport-hdlc-async`, `smol`, `encode`, `parse`  
**Runtime**: Smol

Demonstrates:
- Layering async HDLC over async TCP with Smol runtime
- Lightweight async HDLC frame handling
- Minimal memory footprint for embedded use
- Efficient async frame processing
- Simple deployment on embedded Linux

**Use case**: Embedded concentrators with HDLC, ARM/x86 edge devices, moderate-scale deployments

---

### 8. Async TCP Transport (Glommio) ✨ **NEW**
**File**: `tcp_transport_async_glommio.rs`  
**Features**: `async-client`, `transport-tcp-async`, `glommio`, `encode`, `parse`  
**Runtime**: Glommio  
**Platform**: Linux only (kernel 5.8+, io_uring)

Demonstrates:
- Creating an async TCP transport with Glommio runtime (io_uring)
- Thread-per-core architecture for ultra-low latency
- Linux-specific high-performance async I/O
- LocalExecutor usage patterns
- When to use Glommio vs other runtimes

**Use case**: Linux-only deployments, high-frequency trading systems, ultra-low latency requirements (<1ms), real-time data acquisition

---

### 9. Async TCP Transport (Embassy) ✨ **NEW**
**File**: `tcp_transport_async_embassy.rs`  
**Features**: `async-client`, `transport-tcp-async`, `embassy`, `encode`, `parse`  
**Runtime**: Embassy  
**Platform**: Cross-platform (std), Embedded (no_std ready)

Demonstrates:
- Creating an async TCP transport with Embassy runtime
- Embedded-first async patterns (std and no_std examples)
- Minimal memory footprint for resource-constrained devices
- Cooperative async I/O with yielding
- Stack-allocated buffers for embedded systems
- When to use Embassy vs other runtimes

**Use case**: Embedded systems (MCUs), IoT devices, battery-powered meters, no_std environments, deterministic real-time systems

---

### 10. Async Basic (Tokio)
**File**: `async_basic_tokio.rs`  
**Features**: `async-client`, `tokio`, `encode`, `parse`  
**Runtime**: Tokio

Demonstrates:
- Creating an async DLMS client with Tokio
- API patterns for async operations (read, write, method)
- Proper async workflow (connect, operations, disconnect)
- Code examples ready to adapt for real transports

**Use case**: Learning async client API, template for real implementations

**Note**: This is a demonstrative example showing API usage patterns. To use with real devices, uncomment the client creation and operation calls, and add a real transport (TCP or Serial)

---

## 🏗️ Architecture Overview

### Unified Builder API (Sync & Async)

**All examples now use identical builder patterns with flexible buffer allocation:**

```rust
// Synchronous client - Heap buffer (runtime size)
use dlms_cosem::client::{ClientBuilder, ClientSettings};
let sync_client = ClientBuilder::new(transport, settings)
    .build_with_heap(2048);

// Synchronous client - Stack buffer (compile-time size, no_std)
let sync_client = ClientBuilder::new(transport, settings)
    .build_with_heapless::<2048>();

// Asynchronous client - Heap buffer (runtime size)
use dlms_cosem::async_client::AsyncClientBuilder;
let async_client = AsyncClientBuilder::new(async_transport, settings)
    .build_with_heap(2048);

// Asynchronous client - Stack buffer (compile-time size, no_std)
let async_client = AsyncClientBuilder::new(async_transport, settings)
    .build_with_heapless::<2048>();
```

**Buffer Size Recommendations:**
- **256 bytes**: Minimum for basic read/write
- **2048 bytes**: Standard (used in all examples)
- **4096-8192 bytes**: Load profiles, block transfers
- **65635 bytes**: Maximum PDU + overhead

**Key Benefits:**
- ✅ Identical API for sync and async clients
- ✅ Explicit buffer allocation strategy (heap vs stack)
- ✅ Type-safe buffer management
- ✅ Full `no_std` support with heapless feature
- ✅ Zero runtime overhead

### Synchronous Client Architecture
```
┌─────────────────┐
│  Application    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ClientBuilder<T> │ ◄─── Builder pattern
└────────┬────────┘
         │ .build_with_heap() or
         │ .build_with_heapless()
         ▼
┌─────────────────┐
│DlmsClient<T, B> │ ◄─── Sync client
└────────┬────────┘      (T=Transport, B=Buffer)
         │
         ▼
┌─────────────────┐
│   Transport     │ ◄─── TCP, Serial, HDLC
└─────────────────┘
```

### Asynchronous Client Architecture
```
┌─────────────────┐
│  Application    │
└────────┬────────┘
         │ .await
         ▼
┌─────────────────────┐
│AsyncClientBuilder<T>│ ◄─── Builder pattern
└────────┬────────────┘
         │ .build_with_heap() or
         │ .build_with_heapless()
         ▼
┌──────────────────────┐
│AsyncDlmsClient<T, B> │ ◄─── Async client
└────────┬─────────────┘      (T=AsyncTransport, B=Buffer)
         │ .await
         ▼
┌─────────────────┐
│ AsyncTransport  │ ◄─── Runtime-agnostic
└────────┬────────┘
         │
    ┌────┴────┬─────────┬──────────┐
    ▼         ▼         ▼          ▼           ▼
┌────────┐┌────────┐┌──────┐┌──────────┐┌──────────┐
│ Tokio  ││ Smol   ││Glommio││ Embassy  ││  Sync    │
└────────┘└────────┘└──────┘└──────────┘└──────────┘
```

## 🔧 Production Usage Patterns

### 1. Simple CLI Tool (Sync)
```rust
use dlms_cosem::client::sync::{ClientBuilder, ClientSettings};

let settings = ClientSettings::default();
let transport = TcpTransport::connect("192.168.1.100:4059")?;
let mut client = ClientBuilder::new(transport, settings).build_with_heap(2048);

client.connect()?;
let value = client.read(3, obis, 2, None)?;
println!("Value: {:?}", value);
client.disconnect()?;
```

### 2. Web Service (Async + Pool)
```rust
use dlms_cosem::pool::async::DlmsPoolBuilder;

let pool = DlmsPoolBuilder::new(
    "192.168.1.100:4059",
    TokioTcpTransport::factory(),
    settings
)
.max_size(100)
.build()
.await?;

// In request handler
let mut client = pool.get().await?;
let value = client.read(3, obis, 2, None).await?;
```

### 3. Batch Meter Reading (Async)
```rust
// Read multiple meters concurrently
let tasks: Vec<_> = meter_endpoints.iter()
    .map(|endpoint| {
        let pool = pool.clone();
        tokio::spawn(async move {
            let mut client = pool.get().await?;
            client.read(3, obis, 2, None).await
        })
    })
    .collect();

let results = futures::future::join_all(tasks).await;
```

### 4. Embedded Concentrator (smol)
```rust
use dlms_cosem::client::async::{AsyncClientBuilder, ClientSettings};

smol::block_on(async {
    let mut client = AsyncClientBuilder::new(transport, settings)
        .build_with_heap(1024);
    
    client.connect().await?;
    let value = client.read(3, obis, 2, None).await?;
    client.disconnect().await?;
    Ok(())
})
```

## 📊 Feature Comparison

| Feature | Sync Client | Async (Tokio) | Async (Smol) | Async (Glommio) | Async (Embassy) |
|---------|-------------|---------------|--------------|-----------------|-----------------|
| Binary Size | Small | Large | Small | Medium | Tiny |
| Concurrency | Thread-based | Multi-threaded | Multi-threaded | Thread-per-core | Single-threaded |
| no_std Support | ✅ | ❌ | ❌ | ❌ | ✅ |
| Platform | All | All | All | Linux only | All/Embedded |
| Latency | Medium | Low | Low | Ultra-low (<1ms) | Low (~5-10ms) |
| Connection Pool | ✅ (r2d2) | ✅ (deadpool) | ✅ (deadpool) | Thread-local | ❌ |
| Batch Operations | ✅ | ✅ | ✅ | ✅ | ✅ |
| Best For | CLI, Simple | Web, HES | Resource-limited | Real-time, HFT | Bare-metal MCU |

## 🎯 Use Case Recommendations

### HES (Head-End System)
- **Recommended**: Async with Tokio + connection pool
- **Example**: `async_multiple_reads.rs`
- **Scale**: 10,000+ concurrent meters

### SaaS Metering Platform
- **Recommended**: Async with Tokio + pool + cache + retry
- **Example**: `async_multiple_reads.rs`
- **Scale**: Millions of meters

### Embedded Concentrator (Linux)
- **Recommended**: Async with smol
- **Example**: `async_client_smol.rs`
- **Scale**: 100-1,000 meters per concentrator

### CLI Utility
- **Recommended**: Sync client
- **Example**: `basic_client.rs`
- **Scale**: Single meter operations

### Bare-Metal Embedded
- **Recommended**: Async with Embassy (no_std)
- **Example**: `tcp_transport_async_embassy.rs` ✨ **NEW**
- **Scale**: Single meter per device
- **Benefits**: Minimal footprint, deterministic scheduling, battery-efficient

## 🧪 Testing Examples

All examples use mock transports for demonstration. To test with real devices:

### 1. Replace Mock Transport
```rust
// From:
let transport = MockAsyncTransport::new();

// To (TCP):
let transport = TokioTcpTransport::connect("192.168.1.100:4059").await?;

// To (Serial):
let transport = TokioSerialTransport::open("/dev/ttyUSB0", 9600)?;
```

### 2. Add Authentication
```rust
let settings = ClientSettings {
    authentication: Some(AuthenticationMechanism::LowLevelSecurity {
        password: b"password".to_vec(),
    }),
    ..ClientSettings::default()
};
```

### 3. Enable Encryption
```rust
let settings = ClientSettings {
    use_ciphering: true,
    authentication_key: Some([0u8; 16]),
    encryption_key: Some([0u8; 16]),
    ..ClientSettings::default()
};
```

## 📚 Additional Resources

### Documentation
- [DLMS Green Book Ed. 12](https://www.dlms.com/documentation)
- [IEC 62056-5-3](https://webstore.iec.ch/publication/6016)
- [Crate Documentation](https://docs.rs/dlms_cosem)

### Related Examples
- Connection pooling: (See roadmap Phase 6.2.4)
- Retry logic: (See roadmap Phase 6.2.5)
- Response caching: (See roadmap Phase 6.2.6)
- Batch operations: (See roadmap Phase 6.2.7)

### Community
- [GitHub Repository](https://github.com/reitermarkus/dlms-cosem-rs)
- [Issue Tracker](https://github.com/reitermarkus/dlms-cosem-rs/issues)

## 🔮 Future Examples

Planned for future releases:
- `async_multiple_reads.rs` - Multiple concurrent reads and batch operations
- `async_load_profile.rs` - Load profile reading with selective access
- `async_clock_sync.rs` - Clock synchronization and time management
- `async_client_smol.rs` - Async client with smol runtime (lightweight)
- `async_connection_pool.rs` - Connection pooling with deadpool/r2d2
- `async_retry_logic.rs` - Retry with exponential backoff
- `async_caching.rs` - Response caching with moka
- `async_batch_dataloader.rs` - Batch operations with dataloader
- `hdlc_glommio.rs` - HDLC wrapper for Glommio runtime
- `hdlc_embassy.rs` - HDLC wrapper for Embassy runtime
- `serial_transport_async.rs` - Async serial port communication
- `real_tcp_client.rs` - Real TCP transport example
- `real_serial_client.rs` - Real serial (HDLC) transport example

## 💡 Tips and Best Practices

### Performance
1. Use async for high-concurrency scenarios (>10 concurrent operations)
2. Use connection pooling for SaaS/HES deployments
3. Use batch operations (`read_multiple`) when possible
4. Enable caching for frequently-read static data

### Reliability
1. Always handle errors gracefully
2. Use retry logic with exponential backoff
3. Implement connection health checks
4. Log all DLMS operations for debugging

### Security
1. Always use encryption in production (`use_ciphering: true`)
2. Rotate authentication keys regularly
3. Use HLS-GMAC for highest security
4. Never hardcode credentials

### Debugging
1. Enable transport logging to see raw bytes
2. Use Wireshark with DLMS dissector
3. Compare with Gurux DLMS Director
4. Check clock synchronization first

## 🎨 Code Quality Standards

This project enforces strict quality standards suitable for safety-critical and embedded systems.

### No Magic Numbers Policy

**All protocol values, constants, and byte sequences must use named constants.**

❌ **Bad (Magic Numbers)**
```rust
if data.len() > MAX_FRAME_SIZE - 20 {
    return Err(Error::FrameTooLarge);
}
self.buffer[pos] = 0xA0;  // What is 0xA0?
pos += 2;  // Why 2?
```

✅ **Good (Named Constants)**
```rust
if data.len() > MAX_FRAME_SIZE - HDLC_MAX_OVERHEAD_BYTES {
    return Err(Error::FrameTooLarge);
}
self.buffer[pos] = HDLC_FORMAT_TYPE_3;  // Self-documenting
pos += HDLC_HCS_SIZE;  // Clear intent
```

### Constant Naming Conventions

- Use `SCREAMING_SNAKE_CASE` for all constants
- Include context prefix (e.g., `HDLC_`, `TCP_`, `DLMS_`)
- Include units where applicable (e.g., `_BYTES`, `_SIZE`, `_MASK`)
- Add comprehensive documentation with IEC 62056 standard references

Example:
```rust
/// Frame Format Type 3 identifier (IEC 62056-46).
///
/// Format: 1010yyyy where yyyy = length field size indicator.
/// 0xA0 indicates a single-byte length field.
pub(crate) const HDLC_FORMAT_TYPE_3: u8 = 0xA0;
```

### Quality Checks

All code in this project passes:
```bash
# Zero warnings policy
cargo clippy --all-targets --all-features -- -D warnings

# All tests must pass
cargo test --all-features

# Examples compile and run
cargo build --examples --all-features
```

### Transport Layer Architecture

The transport layer uses a composable design with zero magic numbers:

```rust
// All protocol constants are named and documented
pub const HDLC_FLAG: u8 = 0x7E;              // Frame delimiter
pub const HDLC_FORMAT_TYPE_3: u8 = 0xA0;     // Frame format
pub const HDLC_CONTROL_I_FRAME: u8 = 0x10;   // I-frame control
pub const HDLC_FCS_SIZE: usize = 2;          // Frame check sequence
pub const HDLC_LLC_SIZE: usize = 3;          // LLC header size

// Usage in code is self-documenting
self.frame_buffer[pos] = HDLC_FLAG;
pos += 1;
self.frame_buffer[pos] = HDLC_FORMAT_TYPE_3;
pos += 1;
```

This ensures:
- **Traceability**: All values reference IEC 62056 standards
- **Maintainability**: Protocol changes are centralized
- **Safety**: No typos in safety-critical code
- **Readability**: Code is self-documenting
