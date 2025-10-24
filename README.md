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
- **Chrono Integration (optional)**: Interoperability with the `chrono` datetime library
  - Enable with `chrono-conversions` feature flag
  - Convert between DLMS temporal types and chrono types
  - Works in both `std` and `no_std` environments
  - `DateTime::now()` requires `std` feature for system clock access
- **Jiff Integration (optional)**: Interoperability with the modern `jiff` datetime library
  - Enable with `jiff-conversions` feature flag
  - Convert between DLMS temporal types and jiff civil types
  - Works in both `std` and `no_std` environments
  - `DateTime::now_jiff()` requires `std` feature for system clock access

## Implementation Status

This library currently implements a subset of the DLMS/COSEM specification** (Green Book Ed. 12), focusing on core serialization functionality:

### ✅ Implemented (Milestone 1: Basic Serialization - Complete)

- **Data Type Encoding/Parsing**
  - All 18 DLMS data types (Null, Integer, Unsigned, Long, LongUnsigned, DoubleLong, DoubleLongUnsigned, Long64, Long64Unsigned, Enum, Float32, Float64, OctetString, Utf8String, Date, Time, DateTime, Structure)
  - Big-endian encoding per A-XDR specification
  - IEEE 754 floating point support
  - Recursive structure encoding
  
- **OBIS Code Encoding/Parsing**
  - 6-byte OBIS codes (A-B:C.D.E.F format)
  - With/without type tag encoding
  
- **DateTime Encoding/Parsing**
  - Date, Time, DateTime types with wildcard support
  - Timezone offset handling
  - Clock status flags
  - Chrono interoperability (optional)
  - Jiff interoperability (optional)
  
- **Unit and Scaler Encoding/Parsing**
  - 75+ DLMS unit types (energy, power, voltage, current, etc.)
  - ScalerUnit structure for register scaling

### 🚧 Not Yet Implemented

- **Client APDUs**: GET/SET/ACTION requests and responses
- **Association Layer**: AARQ/AARE, RELEASE request/response
- **Security**: Encryption, authentication, ciphering
- **COSEM Object Model**: Register, ProfileGeneric, Clock, AssociationLN objects
- **Selective Access**: RangeDescriptor, EntryDescriptor
- **Client Implementation**: Full DLMS client with transport layer

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

### Chrono Interoperability

Add to your `Cargo.toml`:

```toml
[dependencies]
dlms_cosem = { version = "0.3", features = ["encode", "chrono-conversions"] }
```

Example:

```rust
use dlms_cosem::{Data, DateTime};
use chrono::NaiveDateTime;

// Convert from chrono to DLMS DateTime
let naive_dt = NaiveDateTime::parse_from_str(
    "2024-06-15 14:30:45",
    "%Y-%m-%d %H:%M:%S"
).unwrap();
let dlms_dt = DateTime::from_chrono(&naive_dt, 120, 0x00); // UTC+2

// Get current time (requires std feature)
#[cfg(feature = "std")]
let now = DateTime::now();

// Encode to DLMS format
let data = Data::DateTime(dlms_dt);
let encoded = data.encode();
```

**Note**: `from_chrono()` methods work in `no_std` environments. Only `DateTime::now()` requires the `std` feature.

### Jiff Interoperability

Add to your `Cargo.toml`:

```toml
[dependencies]
dlms_cosem = { version = "0.3", features = ["encode", "jiff-conversions"] }
```

Example:

```rust
use dlms_cosem::{Data, DateTime};
use jiff::civil::DateTime as JiffDateTime;

// Convert from jiff to DLMS DateTime
let jiff_dt = JiffDateTime::new(2024, 6, 15, 14, 30, 45, 0).unwrap();
let dlms_dt = DateTime::from_jiff(&jiff_dt, 120, 0x00); // UTC+2

// Get current time (requires std feature)
#[cfg(feature = "std")]
let now = DateTime::now_jiff();

// Encode to DLMS format
let data = Data::DateTime(dlms_dt);
let encoded = data.encode();
```

**Note**: `from_jiff()` methods work in `no_std` environments. Only `DateTime::now_jiff()` requires the `std` feature.

**Both chrono and jiff**: You can enable both `chrono-conversions` and `jiff-conversions` simultaneously if you need interoperability with both libraries.

### Unit and Scaler Support

```rust
use dlms_cosem::{ScalerUnit, Unit};

// Energy register with scaler: raw_value=123456, scaler=-2, unit=Wh
// Actual value: 123456 * 10^(-2) = 1234.56 Wh
let scaler_unit = ScalerUnit {
    scaler: -2,
    unit: Unit::WattHour,
};

#[cfg(feature = "encode")]
let encoded = scaler_unit.encode(); // [0x02, 0x02, 0x0F, 0xFE, 0x16, 0x1E]

let (_, parsed) = ScalerUnit::parse(&encoded).unwrap();
assert_eq!(parsed, scaler_unit);
```

## Quality Standards

- ✅ **100% Safe Rust**: Zero unsafe blocks
- ✅ **no_std Compatible**: Works in embedded environments
- ✅ **Panic-Free**: All errors returned as Result/IResult
- ✅ **Well-Tested**: >85% code coverage
- ✅ **Green Book Compliant**: Follows DLMS UA 1000-2 Ed. 12 specification

For more information, also take a look at https://github.com/reitermarkus/smart-meter-rs.
