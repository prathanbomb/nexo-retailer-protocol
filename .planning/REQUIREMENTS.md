# Requirements: Nexo Retailer Protocol (Rust)

**Defined:** 2026-02-28
**Core Value:** Enable embedded payment devices to communicate using the Nexo Retailer Protocol with a Rust implementation that works in both bare metal (no_std) and standard environments.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Schema & Code Generation

- [x] **SCHEMA-01**: All 17 CASP XSD files (casp.001-017) converted to .proto format
- [x] **SCHEMA-02**: Field number registry established with reserved field tracking to prevent silent data corruption
- [x] **SCHEMA-03**: Build script (build.rs) using prost-build for Rust code generation
- [x] **SCHEMA-04**: Generated Rust message structs configured with BTreeMap for no_std compatibility
- [x] **SCHEMA-05**: Integer-based monetary value types (int64 units + int32 nanos, never float/double)

### Serialization (Codec)

- [x] **CODEC-01**: Protobuf encode for all 17 CASP message types
- [x] **CODEC-02**: Protobuf decode for all 17 CASP message types
- [x] **CODEC-03**: Message size limits enforced to prevent unbounded allocation
- [x] **CODEC-04**: no_std-compatible codec layer (uses prost with default-features = false)
- [x] **CODEC-05**: Codec layer isolated for testing and potential codec swaps

### Validation

- [x] **VAL-01**: Field presence validation per XSD constraints
- [x] **VAL-02**: Type checking for all message fields
- [x] **VAL-03**: Currency code validation (ISO 4217)
- [x] **VAL-04**: String encoding validation (UTF-8)
- [x] **VAL-05**: Basic constraint validation (length limits, ranges)
- [x] **VAL-06**: Validation conditional on alloc feature (heap required for collections)

### Error Handling

- [x] **ERROR-01**: Custom error enum (not thiserror, which is std-only)
- [x] **ERROR-02**: Meaningful error codes for common failures (connection, timeout, validation, encoding)
- [x] **ERROR-03**: no_std-compatible error types implementing core::error::Error
- [x] **ERROR-04**: Error types support defmt for embedded logging

### Transport Layer

- [x] **TRANS-01**: Transport trait defining async read/write interface
- [ ] **TRANS-02**: Custom TCP framing per Nexo specification (message boundaries, length prefixes)
- [x] **TRANS-03**: Tokio transport implementation (std feature)
- [x] **TRANS-04**: Embassy transport implementation (no_std feature)
- [x] **TRANS-05**: Connection timeout handling
- [x] **TRANS-06**: Message framing tests (round-trip with oversized/malformed messages)

### Feature Flags & Platform Support

- [x] **PLAT-01**: std feature enables Tokio, tracing, and standard library
- [x] **PLAT-02**: no_std feature enables Embassy, heapless, and defmt
- [x] **PLAT-03**: alloc feature enables heap-based collections for validation
- [x] **PLAT-04**: Bare-metal CI target (thumbv7em-none-eabihf) from day one
- [x] **PLAT-05**: Dependency audit for std leakage on no_std builds

### Client API

- [x] **CLIENT-01**: Client API for POS initiators (request/response pattern)
- [x] **CLIENT-02**: Builder pattern for complex message construction
- [x] **CLIENT-03**: Connection management (connect, disconnect)
- [x] **CLIENT-04**: Request/response correlation with pending request tracking
- [x] **CLIENT-05**: Basic reconnection logic with exponential backoff
- [x] **CLIENT-06**: Unique message ID generation for replay protection
- [x] **CLIENT-07**: Timeout handling with late response rejection
- [x] **CLIENT-08**: Client integration tests with mock server

### Server API

- [ ] **SERVER-01**: Server API for device listeners (concurrent connection handling)
- [x] **SERVER-02**: Connection manager with per-connection state tracking
- [ ] **SERVER-03**: Request dispatcher routing to application handlers
- [x] **SERVER-04**: Message deduplication for replay attack prevention
- [ ] **SERVER-05**: Server integration tests with concurrent clients

### Reliability Features

- [x] **RELY-01**: Heartbeat/keepalive protocol for dead connection detection
- [x] **RELY-02**: Configurable heartbeat interval and timeout
- [x] **RELY-03**: Automatic reconnection with configurable retry attempts

### Logging & Diagnostics

- [x] **LOG-01**: Structured logging integration (tracing for std, defmt for no_std)
- [x] **LOG-02**: Configurable log verbosity levels
- [x] **LOG-03**: Connection state logging for debugging

### Testing

- [x] **TEST-01**: Unit tests for all message types (encode/decode round-trip)
- [x] **TEST-02**: Unit tests for validation logic
- [x] **TEST-03**: Integration tests for transport layer
- [x] **TEST-04**: Integration tests for client/server communication
- [x] **TEST-05**: Property-based tests for serialization edge cases

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Performance Optimizations

- **PERF-01**: Zero-copy message parsing using bytes::Bytes / bytes::Buf
- **PERF-02**: Heapless operation mode for deterministic memory usage
- **PERF-03**: Feature-flagged message types (compile-time subset selection)

### Advanced Testing

- **TEST-06**: Fuzzing tests for malformed message handling
- **TEST-07**: Protocol conformance tests against reference implementations

### Developer Experience

- **DX-01**: Protocol diagnostics (state inspection, debugging tools)
- **DX-02**: Embedded examples (client_std.rs, server_no_std.rs)
- **DX-03**: QEMU-based embedded CI testing

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| XML/XSD serialization | Project explicitly uses protobuf for efficiency — no XML support needed |
| HTTP/REST wrapper | Nexo uses custom TCP framing, not HTTP — wrapper would be separate crate |
| Synchronous blocking API | Async-first design; sync wrappers can be added later if needed |
| Dynamic message discovery | All 17 messages known at compile time; type safety is core value |
| Built-in hardware crypto | Hardware-specific, out of scope for protocol library — accept crypto trait objects |
| GUI/CLI tools | Bloats library, maintenance burden — separate nexo-cli crate if needed |
| Transaction persistence | Orthogonal concern, varies by application — library provides messages only |
| EMV kernel implementation | Massive complexity, hardware-dependent, separate certification — focus on protocol |
| ISO 8583 support | Different protocol family — focus on Nexo/ISO 20022 CASP only |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SCHEMA-01 | Phase 1 | Complete |
| SCHEMA-02 | Phase 1 | Complete |
| SCHEMA-03 | Phase 1 | Complete |
| SCHEMA-04 | Phase 1 | Complete |
| SCHEMA-05 | Phase 1 | Complete |
| CODEC-01 | Phase 2 | Complete |
| CODEC-02 | Phase 2 | Complete |
| CODEC-03 | Phase 2 | Complete |
| CODEC-04 | Phase 2 | Complete |
| CODEC-05 | Phase 2 | Complete |
| VAL-01 | Phase 2 | Complete |
| VAL-02 | Phase 2 | Complete |
| VAL-03 | Phase 2 | Complete |
| VAL-04 | Phase 2 | Complete |
| VAL-05 | Phase 2 | Complete |
| VAL-06 | Phase 2 | Complete |
| ERROR-01 | Phase 2 | Complete |
| ERROR-02 | Phase 2 | Complete |
| ERROR-03 | Phase 2 | Complete |
| ERROR-04 | Phase 2 | Complete |
| TRANS-01 | Phase 3 | Complete |
| TRANS-02 | Phase 5.1 | Pending |
| TRANS-03 | Phase 3 | Complete |
| TRANS-04 | Phase 3 | Complete |
| TRANS-05 | Phase 3 | Complete |
| TRANS-06 | Phase 3 | Complete |
| PLAT-01 | Phase 2 | Complete |
| PLAT-02 | Phase 2 | Complete |
| PLAT-03 | Phase 2 | Complete |
| PLAT-04 | Phase 2 | Complete |
| PLAT-05 | Phase 2 | Complete |
| CLIENT-01 | Phase 4 | Complete |
| CLIENT-02 | Phase 4 | Complete |
| CLIENT-03 | Phase 4 | Complete |
| CLIENT-04 | Phase 4 | Complete |
| CLIENT-05 | Phase 4 | Complete |
| CLIENT-06 | Phase 4 | Complete |
| CLIENT-07 | Phase 4 | Complete |
| CLIENT-08 | Phase 4 | Complete |
| SERVER-01 | Phase 5.1 | Pending |
| SERVER-02 | Phase 5 | Complete |
| SERVER-03 | Phase 5.1 | Pending |
| SERVER-04 | Phase 5 | Complete |
| SERVER-05 | Phase 5.1 | Pending |
| RELY-01 | Phase 5 | Complete |
| RELY-02 | Phase 5 | Complete |
| RELY-03 | Phase 4 | Complete |
| LOG-01 | Phase 5 | Complete |
| LOG-02 | Phase 5 | Complete |
| LOG-03 | Phase 5 | Complete |
| TEST-01 | Phase 6 | Complete |
| TEST-02 | Phase 6 | Complete |
| TEST-03 | Phase 6 | Complete |
| TEST-04 | Phase 6 | Complete |
| TEST-05 | Phase 6 | Complete |

**Coverage:**
- v1 requirements: 55 total
- Mapped to phases: 55
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-28*
*Last updated: 2026-02-28 after roadmap creation*
