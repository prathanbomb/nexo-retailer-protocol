# Roadmap: Nexo Retailer Protocol (Rust)

## Overview

This roadmap delivers a Rust implementation of the Nexo Retailer Protocol (ISO 20022 CASP) that uniquely supports both standard server environments and embedded bare-metal devices. The journey begins with schema conversion from XSD to protobuf, builds core serialization and validation layers, implements dual runtime transport abstraction, and culminates with client/server APIs and comprehensive testing. Each phase delivers verifiable capabilities that unblock subsequent work, enabling early validation of the dual std/no_std architecture that is the project's primary differentiator.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Schema Conversion** - Convert 17 CASP XSD files to protobuf with field number registry and code generation ✓ (2026-02-28)
- [x] **Phase 2: Core Library** - Implement codec, validation, error handling, and platform feature flags ✓ (2026-03-01)
- [ ] **Phase 3: Transport Layer** - Build dual runtime (Tokio/Embassy) transport abstraction with custom TCP framing
- [ ] **Phase 4: Client API** - Deliver POS initiator API with connection management and reconnection
- [ ] **Phase 5: Server API & Reliability** - Implement concurrent server, heartbeat, logging, and deduplication
- [ ] **Phase 6: Testing & Verification** - Comprehensive test suite with property-based, integration, and embedded CI

## Phase Details

### Phase 1: Schema Conversion

**Goal**: All 17 CASP message types defined as protobuf schemas with Rust code generation, field number registry established, and monetary values using integer representation.

**Depends on**: Nothing (first phase)

**Requirements**: SCHEMA-01, SCHEMA-02, SCHEMA-03, SCHEMA-04, SCHEMA-05

**Success Criteria** (what must be TRUE):
  1. All 17 CASP XSD files are converted to .proto format in the source tree
  2. Field number registry document exists with reserved field tracking to prevent silent data corruption
  3. Running `cargo build` generates Rust message structs via prost-build build.rs script
  4. Generated message structs compile with BTreeMap configuration (not HashMap) for no_std compatibility
  5. All monetary value fields use integer representation (int64 units + int32 nanos), never float/double

**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Convert XSD schemas to protobuf format (SCHEMA-01, SCHEMA-05)
- [x] 01-02-PLAN.md — Establish field number registry and build code generation pipeline (SCHEMA-02, SCHEMA-03, SCHEMA-04)
- [x] 01-03-PLAN.md — Consolidate duplicate message types and complete code generation

### Phase 2: Core Library

**Goal**: Codec, validation, and error handling implemented with no_std compatibility and platform feature flag architecture established from day one.

**Depends on**: Phase 1 (requires generated message types)

**Requirements**: CODEC-01, CODEC-02, CODEC-03, CODEC-04, CODEC-05, VAL-01, VAL-02, VAL-03, VAL-04, VAL-05, VAL-06, ERROR-01, ERROR-02, ERROR-03, ERROR-04, PLAT-01, PLAT-02, PLAT-03, PLAT-04, PLAT-05

**Success Criteria** (what must be TRUE):
  1. All 17 CASP message types can be encoded to and decoded from protobuf bytes
  2. Message size limits are enforced before allocation to prevent unbounded memory usage
  3. Validation rejects messages with missing required fields, invalid currency codes, or malformed UTF-8
  4. Custom error types work in both std and no_std environments (no thiserror dependency)
  5. Bare-metal target (thumbv7em-none-eabihf) compiles successfully with no_std feature
  6. Dependency audit confirms no std leakage when building with no_std feature

**Plans**: 4 plans

Plans:
- [x] 02-01-PLAN.md — Implement no_std-compatible error types and platform feature flags (ERROR-01, ERROR-02, ERROR-03, ERROR-04, PLAT-01, PLAT-02, PLAT-03)
- [x] 02-02-PLAN.md — Implement protobuf codec layer with size limits (CODEC-01, CODEC-02, CODEC-03, CODEC-04, CODEC-05)
- [x] 02-03-PLAN.md — Implement message validation for XSD constraints (VAL-01, VAL-02, VAL-03, VAL-04, VAL-05, VAL-06)
- [x] 02-04-PLAN.md — Establish bare-metal CI and dependency audit (PLAT-04, PLAT-05)

### Phase 3: Transport Layer

**Goal**: Runtime-agnostic transport trait with Tokio and Embassy implementations, custom TCP framing per Nexo specification, and connection timeout handling.

**Depends on**: Phase 2 (requires codec for message framing tests)

**Requirements**: TRANS-01, TRANS-02, TRANS-03, TRANS-04, TRANS-05, TRANS-06

**Success Criteria** (what must be TRUE):
  1. Transport trait defines async read/write interface that works with both Tokio and Embassy
  2. Messages are framed with Nexo-specified length prefixes and message boundaries
  3. Tokio transport implementation works in std environment
  4. Embassy transport implementation works in no_std environment
  5. Connection timeouts are enforced with proper error signaling
  6. Round-trip tests pass with oversized, malformed, and boundary condition messages

**Plans**: TBD

Plans:
- [ ] 03-01: Define transport trait and custom TCP framing protocol
- [ ] 03-02: Implement Tokio transport (std feature)
- [ ] 03-03: Implement Embassy transport (no_std feature)
- [ ] 03-04: Implement connection timeout and message framing tests

### Phase 4: Client API

**Goal**: High-level client API for POS initiators with builder pattern, connection management, request/response correlation, reconnection logic, and timeout handling.

**Depends on**: Phase 3 (requires transport layer)

**Requirements**: CLIENT-01, CLIENT-02, CLIENT-03, CLIENT-04, CLIENT-05, CLIENT-06, CLIENT-07, CLIENT-08, RELY-03

**Success Criteria** (what must be TRUE):
  1. Client can connect to a server and send payment transaction requests
  2. Builder pattern enables fluent construction of complex CASP messages
  3. Client tracks pending requests and correlates responses with unique message IDs
  4. Client automatically reconnects with exponential backoff when connection fails
  5. Late responses are rejected after timeout to prevent state confusion
  6. Integration tests verify complete request/response flow with mock server

**Plans**: TBD

Plans:
- [ ] 04-01: Implement client connection management and request/response API
- [ ] 04-02: Implement builder pattern for message construction
- [ ] 04-03: Implement reconnection logic with exponential backoff
- [ ] 04-04: Implement timeout handling and late response rejection
- [ ] 04-05: Write client integration tests with mock server

### Phase 5: Server API & Reliability

**Goal**: Concurrent server API with connection management, request dispatcher, message deduplication for replay protection, heartbeat/keepalive protocol, and structured logging integration.

**Depends on**: Phase 4 (requires client API for integration testing)

**Requirements**: SERVER-01, SERVER-02, SERVER-03, SERVER-04, SERVER-05, RELY-01, RELY-02, LOG-01, LOG-02, LOG-03

**Success Criteria** (what must be TRUE):
  1. Server accepts concurrent connections from multiple clients
  2. Per-connection state tracking isolates client sessions
  3. Request dispatcher routes incoming messages to application handlers
  4. Duplicate message IDs are rejected within time window (replay attack prevention)
  5. Heartbeat protocol detects dead connections faster than TCP keepalive
  6. Structured logging works with tracing (std) and defmt (no_std)
  7. Integration tests verify concurrent client handling and load scenarios

**Plans**: TBD

Plans:
- [ ] 05-01: Implement server connection manager and concurrent request handling
- [ ] 05-02: Implement request dispatcher and application handler routing
- [ ] 05-03: Implement message deduplication for replay attack prevention
- [ ] 05-04: Implement heartbeat/keepalive protocol
- [ ] 05-05: Integrate structured logging (tracing for std, defmt for no_std)
- [ ] 05-06: Write server integration tests with concurrent clients

### Phase 6: Testing & Verification

**Goal**: Comprehensive test suite covering unit tests for all message types, validation logic, transport layer, client/server communication, and property-based tests for serialization edge cases.

**Depends on**: Phase 5 (requires complete protocol stack for end-to-end testing)

**Requirements**: TEST-01, TEST-02, TEST-03, TEST-04, TEST-05

**Success Criteria** (what must be TRUE):
  1. All 17 CASP message types have encode/decode round-trip unit tests
  2. Validation logic has unit tests for all constraint types (presence, type, length, range)
  3. Transport layer has integration tests for both std and no_std variants
  4. Client/server communication has end-to-end integration tests
  5. Property-based tests discover serialization edge cases (malformed inputs, boundary conditions)
  6. Bare-metal CI target runs full test suite on every commit

**Plans**: TBD

Plans:
- [ ] 06-01: Write unit tests for all message types (encode/decode round-trip)
- [ ] 06-02: Write unit tests for validation logic
- [ ] 06-03: Write integration tests for transport layer
- [ ] 06-04: Write integration tests for client/server communication
- [ ] 06-05: Write property-based tests for serialization edge cases
- [ ] 06-06: Configure bare-metal CI to run full test suite

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Schema Conversion | 3/3 | ✓ Complete | 2026-02-28 |
| 2. Core Library | 0/4 | Planning complete | 2026-02-28 |
| 3. Transport Layer | 0/4 | Not started | - |
| 4. Client API | 0/5 | Not started | - |
| 5. Server API & Reliability | 0/6 | Not started | - |
| 6. Testing & Verification | 0/6 | Not started | - |
