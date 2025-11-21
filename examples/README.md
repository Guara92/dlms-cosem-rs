# DLMS/COSEM Rust Client - Examples

This directory contains comprehensive examples demonstrating how to use the `dlms_cosem` library for various scenarios, from basic synchronous operations to advanced async patterns.

## 📚 Overview

The examples are organized by complexity and use case:

### Synchronous Examples
- **[basic_client.rs](basic_client.rs)** - Basic sync client usage (connect, read, write, method, disconnect)

### Asynchronous Examples
- **[async_basic_tokio.rs](async_basic_tokio.rs)** - Basic async client API demonstration with Tokio runtime

## 🚀 Quick Start

### Running Examples

Each example requires specific feature flags. Use the commands below:

#### Synchronous Client
```bash
# Basic sync client
cargo run --example basic_client --features client,encode,parse
```

#### Asynchronous Client (Tokio)
```bash
# Basic async with Tokio (API demonstration)
cargo run --example async_basic_tokio --features async-client,tokio,encode,parse
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

### 2. Async Basic (Tokio)
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

### Synchronous Client Architecture
```
┌─────────────────┐
│  Application    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  DlmsClient<T>  │ ◄─── Sync client
└────────┬────────┘
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
┌─────────────────┐
│AsyncDlmsClient  │ ◄─── Async client
└────────┬────────┘
         │ .await
         ▼
┌─────────────────┐
│ AsyncTransport  │ ◄─── Runtime-agnostic
└────────┬────────┘
         │
    ┌────┴────┬─────────┬──────────┐
    ▼         ▼         ▼          ▼
┌────────┐┌────────┐┌──────┐┌──────────┐
│ Tokio  ││ smol   ││Glommio││ Embassy  │
└────────┘└────────┘└──────┘└──────────┘
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

| Feature | Sync Client | Async (Tokio) | Async (smol) | Async (Embassy) |
|---------|-------------|---------------|--------------|-----------------|
| Binary Size | Small | Large | Small | Tiny |
| Concurrency | Thread-based | Task-based | Task-based | Task-based |
| no_std Support | ✅ | ❌ | ❌ | ✅ |
| Connection Pool | ✅ (r2d2) | ✅ (deadpool) | ✅ (deadpool) | ❌ |
| Batch Operations | ✅ | ✅ | ✅ | ✅ |
| Best For | CLI, Embedded | Web, HES | Concentrators | Bare-metal |

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
- **Example**: (Future) `async_client_embassy.rs`
- **Scale**: Single meter per device

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
- `async_client_embassy.rs` - Bare-metal embedded with Embassy
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
