# Nexo Retailer Protocol (ISO 20022 CASP)

A Rust implementation of the Nexo Retailer Protocol based on ISO 20022 CASP (Cardholder and Acquirer System Protocol) specifications.

## Features

- **no_std compatible**: Works on bare-metal embedded targets
- **Protobuf codec**: Encode/decode all 17 CASP message types
- **Message validation**: XSD constraint validation (currency codes, string lengths, required fields)
- **Error handling**: Custom error types with `defmt` support for embedded logging

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
nexo-retailer-protocol = { version = "0.1", default-features = false }
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | Yes | Enables standard library, Tokio, and prost std support |
| `alloc` | No | Enables heap-based collections for validation |
| `defmt` | No | Enables `defmt::Format` derive for embedded logging |

### Examples

```rust
use nexo_retailer_protocol::{
    Casp001Document, Header4, NexoError,
    encode, decode, Validate
};

// Create a message
let message = Casp001Document {
    header: Some(Header4 {
        message_type: "Request".to_string(),
        // ... other fields
        ..Default::default()
    }),
    ..Default::default()
};

// Validate before encoding
message.validate()?;

// Encode to bytes
let bytes = encode(&message)?;

// Decode from bytes
let decoded: Casp001Document = decode(&bytes)?;
```

## no_std Support

This crate is designed to work in `no_std` environments, making it suitable for embedded systems and bare-metal targets.

### Building for Bare-Metal

```bash
# Install the ARM target
rustup target add thumbv7em-none-eabihf

# Build without standard library
cargo build --no-default-features --target thumbv7em-none-eabihf

# Build with alloc support (requires allocator)
cargo build --features alloc --target thumbv7em-none-eabihf

# Build with embedded logging
cargo build --features defmt --target thumbv7em-none-eabihf
```

### Feature Combinations

| Use Case | Build Command |
|----------|---------------|
| Standard application | `cargo build` |
| Embedded (no heap) | `cargo build --no-default-features` |
| Embedded with allocator | `cargo build --features alloc --no-default-features` |
| Embedded with defmt logging | `cargo build --features defmt --no-default-features` |
| Full embedded | `cargo build --features alloc,defmt --no-default-features` |

## Dependency Audit

Check for std leakage in no_std builds:

```bash
./scripts/audit_std_leak.sh
```

This script verifies that no std dependencies are pulled in when building for bare-metal targets.

## CI

The project includes a GitHub Actions workflow that:
- Builds for `thumbv7em-none-eabihf` target
- Runs dependency audit for std leakage
- Builds documentation for bare-metal target

## Supported CASP Messages

All 17 CASP message types are supported:

- CASP-001 to CASP-017: Document-based messages for payment transactions
- Common types: Header4, ActiveCurrencyAndAmount, CardData8, etc.

## License

MIT OR Apache-2.0
