# Requirements: Nexo Retailer Protocol (Rust)

**Defined:** 2026-02-28
**Core Value:** Enable embedded payment devices to communicate using the Nexo Retailer Protocol with a Rust implementation that works in both bare metal (no_std) and standard environments.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Schema & Code Generation

- [x] **SCHEMA-01**: All 17 CASP XSD files (casp.001-017) converted to .proto format
- [x] **SCHEMA-02**: Field number registry established with reserved field tracking to prevent silent data corruption
- [ ] **SCHEMA-03**: Build script (build.rs) using prost-build for Rust code generation
- [ ] **SCHEMA-04**: Generated Rust message structs configured with BTreeMap for no_std compatibility
- [x] **SCHEMA-05**: Integer-based monetary value types (int64 units + int32 nanos, never float/double)

### Serialization (Codec)

- [ ] **CODEC-01**: Protobuf encode for all 17 CASP message types
- [ ] **CODEC-02**: Protobuf decode for all 17 CASP message types
- [ ] **CODEC-03**: Message size limits enforced to prevent unbounded allocation
- [ ] **CODEC-04**: no_std-compatible codec layer (uses prost with default-features = false)
- [ ] **CODEC-05**: Codec layer isolated for testing and potential codec swaps

### Validation

- [ ] **VAL-01**: Field presence validation per XSD constraints
- [ ] **VAL-02**: Type checking for all message fields
- [ ] **VAL-03**: Currency code validation (ISO 4217)
- [ ] **VAL-04**: String encoding validation (UTF-8)
- [ ] **VAL-05**: Basic constraint validation (length limits, ranges)
- [ ] **VAL-06**: Validation conditional on alloc feature (heap required for collections)

### Error Handling

- [x] **ERROR-01**: Custom error enum (not thiserror, which is std-only)
- [x] **ERROR-02**: Meaningful error codes for common failures (connection, timeout, validation, encoding)
- [x] **ERROR-03**: no_std-compatible error types implementing core::error::Error
- [x] **ERROR-04**: Error types support defmt for embedded logging

### Transport Layer

- [ ] **TRANS-01**: Transport trait defining async read/write interface
- [ ] **TRANS-02**: Custom TCP framing per Nexo specification (message boundaries, length prefixes)
- [ ] **TRANS-03**: Tokio transport implementation (std feature)
- [ ] **TRANS-04**: Embassy transport implementation (no_std feature)
- [ ] **TRANS-05**: Connection timeout handling
- [ ] **TRANS-06**: Message framing tests (round-trip with oversized/malformed messages)

### Feature Flags & Platform Support

- [x] **PLAT-01**: std feature enables Tokio, tracing, and standard library
- [x] **PLAT-02**: no_std feature enables Embassy, heapless, and defmt
- [x] **PLAT-03**: alloc feature enables heap-based collections for validation
- [ ] **PLAT-04**: Bare-metal CI target (thumbv7em-none-eabihf) from day one
- [ ] **PLAT-05**: Dependency audit for std leakage on no_std builds

### Client API

- [ ] **CLIENT-01**: Client API for POS initiators (request/response pattern)
- [ ] **CLIENT-02**: Builder pattern for complex message construction
- [ ] **CLIENT-03**: Connection management (connect, disconnect)
- [ ] **CLIENT-04**: Request/response correlation with pending request tracking
- [ ] **CLIENT-05**: Basic reconnection logic with exponential backoff
- [ ] **CLIENT-06**: Unique message ID generation for replay protection
- [ ] **CLIENT-07**: Timeout handling with late response rejection
- [ ] **CLIENT-08**: Client integration tests with mock server

### Server API

- [ ] **SERVER-01**: Server API for device listeners (concurrent connection handling)
- [ ] **SERVER-02**: Connection manager with per-connection state tracking
- [ ] **SERVER-03**: Request dispatcher routing to application handlers
- [ ] **SERVER-04**: Message deduplication for replay attack prevention
- [ ] **SERVER-05**: Server integration tests with concurrent clients

### Reliability Features

- [ ] **RELY-01**: Heartbeat/keepalive protocol for dead connection detection
- [ ] **RELY-02**: Configurable heartbeat interval and timeout
- [ ] **RELY-03**: Automatic reconnection with configurable retry attempts

### Logging & Diagnostics

- [ ] **LOG-01**: Structured logging integration (tracing for std, defmt for no_std)
- [ ] **LOG-02**: Configurable log verbosity levels
- [ ] **LOG-03**: Connection state logging for debugging

### Testing

- [ ] **TEST-01**: Unit tests for all message types (encode/decode round-trip)
- [ ] **TEST-02**: Unit tests for validation logic
- [ ] **TEST-03**: Integration tests for transport layer
- [ ] **TEST-04**: Integration tests for client/server communication
- [ ] **TEST-05**: Property-based tests for serialization edge cases

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
| SCHEMA-03 | Phase 1 | Pending |
| SCHEMA-04 | Phase 1 | Pending |
| SCHEMA-05 | Phase 1 | Complete |
| CODEC-01 | Phase 2 | Pending |
| CODEC-02 | Phase 2 | Pending |
| CODEC-03 | Phase 2 | Pending |
| CODEC-04 | Phase 2 | Pending |
| CODEC-05 | Phase 2 | Pending |
| VAL-01 | Phase 2 | Pending |
| VAL-02 | Phase 2 | Pending |
| VAL-03 | Phase 2 | Pending |
| VAL-04 | Phase 2 | Pending |
| VAL-05 | Phase 2 | Pending |
| VAL-06 | Phase 2 | Pending |
| ERROR-01 | Phase 2 | Complete |
| ERROR-02 | Phase 2 | Complete |
| ERROR-03 | Phase 2 | Complete |
| ERROR-04 | Phase 2 | Complete |
| TRANS-01 | Phase 3 | Pending |
| TRANS-02 | Phase 3 | Pending |
| TRANS-03 | Phase 3 | Pending |
| TRANS-04 | Phase 3 | Pending |
| TRANS-05 | Phase 3 | Pending |
| TRANS-06 | Phase 3 | Pending |
| PLAT-01 | Phase 2 | Complete |
| PLAT-02 | Phase 2 | Complete |
| PLAT-03 | Phase 2 | Complete |
| PLAT-04 | Phase 2 | Pending |
| PLAT-05 | Phase 2 | Pending |
| CLIENT-01 | Phase 4 | Pending |
| CLIENT-02 | Phase 4 | Pending |
| CLIENT-03 | Phase 4 | Pending |
| CLIENT-04 | Phase 4 | Pending |
| CLIENT-05 | Phase 4 | Pending |
| CLIENT-06 | Phase 4 | Pending |
| CLIENT-07 | Phase 4 | Pending |
| CLIENT-08 | Phase 4 | Pending |
| SERVER-01 | Phase 5 | Pending |
| SERVER-02 | Phase 5 | Pending |
| SERVER-03 | Phase 5 | Pending |
| SERVER-04 | Phase 5 | Pending |
| SERVER-05 | Phase 5 | Pending |
| RELY-01 | Phase 5 | Pending |
| RELY-02 | Phase 5 | Pending |
| RELY-03 | Phase 4 | Pending |
| LOG-01 | Phase 5 | Pending |
| LOG-02 | Phase 5 | Pending |
| LOG-03 | Phase 5 | Pending |
| TEST-01 | Phase 6 | Pending |
| TEST-02 | Phase 6 | Pending |
| TEST-03 | Phase 6 | Pending |
| TEST-04 | Phase 6 | Pending |
| TEST-05 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 55 total
- Mapped to phases: 55
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-28*
*Last updated: 2026-02-28 after roadmap creation*
