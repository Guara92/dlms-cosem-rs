# `dlms_cosem`

[![Crates.io](https://img.shields.io/crates/v/dlms_cosem.svg)](https://crates.io/crates/dlms_cosem)
[![Documentation](https://docs.rs/dlms_cosem/badge.svg)](https://docs.rs/dlms_cosem)

This is a `no_std` library for parsing and encoding DLMS/COSEM messages from smart energy meters.

## Features

- **Parsing (default)**: Decode DLMS/COSEM messages from smart meters
- **Encoding (optional)**: Build DLMS/COSEM messages for sending commands to meters
  - Enable with `encode` feature flag
  - Full support for all DLMS data types
  - Round-trip tested: `parse(encode(x)) == x`
  - Big-endian encoding per DLMS specification

## Usage

### Parsing Only (Default)

```rust
use dlms_cosem::Data;

let input = [0x0F, 0x2A]; // Integer = 42
let (remaining, data) = Data::parse(&input).unwrap();
assert_eq!(data, Data::Integer(42));
```

### Parsing + Encoding

Add to your `Cargo.toml`:

```toml
[dependencies]
dlms_cosem = { version = "0.3", features = ["encode"] }
```

Example:

```rust
use dlms_cosem::Data;

// Encode data to DLMS format
let data = Data::Integer(42);
let encoded = data.encode();
assert_eq!(encoded, vec![0x0F, 0x2A]);

// Round-trip verification
let (_, parsed) = Data::parse(&encoded).unwrap();
assert_eq!(parsed, data);
```

For more information, also take a look at https://github.com/reitermarkus/smart-meter-rs.
