# Nexo Retailer Protocol (ISO 20022 CASP)

[![Test Suite](https://github.com/prathanbomb/nexo-retailer-protocol/actions/workflows/test.yml/badge.svg)](https://github.com/prathanbomb/nexo-retailer-protocol/actions/workflows/test.yml)
[![no_std Build](https://github.com/prathanbomb/nexo-retailer-protocol/actions/workflows/no_std.yml/badge.svg)](https://github.com/prathanbomb/nexo-retailer-protocol/actions/workflows/no_std.yml)
[![Coverage](https://codecov.io/gh/prathanbomb/nexo-retailer-protocol/branch/main/graph/badge.svg)](https://codecov.io/gh/prathanbomb/nexo-retailer-protocol)

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

## Testing

### Unit Tests

Run unit tests with:

```bash
cargo test
```

### Property-Based Tests

The project uses `proptest` for property-based testing to discover serialization edge cases:

```bash
# Run property-based tests
cargo test --test serialization_edge_cases --features std

# Run with verbose output
cargo test --test serialization_edge_cases --features std -- --nocapture
```

Property tests verify:
- **Malformed input handling**: Random byte arrays never cause panics
- **Truncated input handling**: Incomplete messages fail gracefully
- **Boundary conditions**: Size limits and numeric boundaries
- **Oversized message rejection**: Messages > 4MB handled correctly
- **Corrupted length prefixes**: Invalid framing handled gracefully
- **Round-trip invariants**: decode(encode(msg)) == msg always holds

#### Regression Persistence

Proptest automatically persists discovered edge cases to `proptest-regressions/`. These files should be committed to version control to prevent regressions:

```bash
# After a property test finds a failing case, commit the regression file
git add proptest-regressions/*.txt
git commit -m "chore: add proptest regression for edge case"
```

The CI workflow verifies that all regression files are committed.

### Integration Tests

The project includes integration tests that verify the complete client request/response flow with a mock Nexo server:

```bash
# Run all integration tests
cargo test --test client_integration

# Run specific integration test
cargo test --test client_integration test_client_connects_to_server -- --exact
```

Integration tests cover:
- Client connection to mock server
- Payment request sending using builder pattern
- Request/response correlation
- Reconnection logic with server failure simulation
- Timeout handling with delayed response simulation
- Builder pattern validation

## Dependency Audit

Check for std leakage in no_std builds:

```bash
./scripts/audit_std_leak.sh
```

This script verifies that no std dependencies are pulled in when building for bare-metal targets.

## CI

The project includes GitHub Actions workflows that:

### Test Suite (test.yml)
- Runs tests on std target with all features
- Runs tests on no_std alloc target
- Checks for uncommitted proptest regressions
- Monitors test execution time (threshold: 5 minutes)
- Uploads code coverage to Codecov

### Bare-Metal CI (no_std.yml)
- Builds for `thumbv7em-none-eabihf` target
- Runs tests on bare-metal target (alloc feature)
- Runs tests with embassy feature
- Runs dependency audit for std leakage on all feature combinations
- Builds documentation for bare-metal target
- Monitors test execution time (threshold: 10 minutes)

### Running Timed Tests Locally

Use the timed test script for local execution monitoring:

```bash
# Run std tests with time monitoring
./scripts/run-timed-tests.sh --std

# Run alloc tests with custom threshold
THRESHOLD_SECONDS=300 ./scripts/run-timed-tests.sh --alloc
```

## Supported CASP Messages

All 17 CASP message types are supported:

- CASP-001 to CASP-017: Document-based messages for payment transactions
- Common types: Header4, ActiveCurrencyAndAmount, CardData8, etc.

## License

MIT OR Apache-2.0
