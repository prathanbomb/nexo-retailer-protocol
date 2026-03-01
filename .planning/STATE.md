---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: in_progress
last_updated: "2026-03-02T19:20:00.000Z"
progress:
  total_phases: 6
  completed_phases: 5
  total_plans: 28
  completed_plans: 27
  current_plan_phase: 05.1-fix-server-framing
  current_plan_number: 01
  current_plan_status: complete
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** Enable embedded payment devices to communicate using the Nexo Retailer Protocol with a Rust implementation that works in both bare metal (no_std) and standard environments.
**Current focus:** Phase 5.1: Fix Server Framing

## Current Position

Phase: 5.1 of 6 (Fix Server Framing)
Plan: 05.1-01 (Integrate FramedTransport into Server) - COMPLETE
Status: Phase 5.1 Complete - 1 of 1 plan complete
Last activity: 2026-03-02 — Completed plan 05.1-01: Integrate FramedTransport into Server (14 min)

Progress: [██████████] 100% (Phase 5.1 complete - 1/1 plan done)

## Performance Metrics

**Velocity:**
- Total plans completed: 27
- Average duration: 85 min (1.4 hours)
- Total execution time: 38.3 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1. Schema Conversion | 3 | 3/3 | 184 min |
| 2. Core Library | 3 | 3/3 | 91 min |
| 3. Transport Layer | 6 | 6/6 | 173 min |
| 4. Client API | 5 | 5/5 | 68 min |
| 5. Server API & Reliability | 6 | 6/6 | 77 min |
| 5.1 Fix Server Framing | 1 | 1/1 | 14 min |
| 6. Testing & Verification | 0 | 0/6 | - |

**Recent Trend:**
- Last 5 plans: P05.1-01(14min), P05-06(3min), P05-05(3min), P05-01(4min), P04-05(2956min)
- Trend: Server FramedTransport integration completed successfully

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

Key architectural decisions will be logged during Phase 1 (Schema Conversion) and Phase 2 (Core Library).
- [Phase 01-schema-conversion]: Enum value naming convention: Prefix enum values to ensure protobuf uniqueness
- [Phase 01-schema-conversion]: Simplified proto structure: Core message types with placeholders for missing types
- [Phase 01-schema-conversion]: Field name normalization: Convert XSD PascalCase to protobuf snake_case
- [Phase 01-schema-conversion P02]: Field number registry with reserved tracking prevents silent data corruption
- [Phase 01-schema-conversion P02]: BTreeMap for all map fields required for no_std compatibility (HashMap needs Random trait)
- [Phase 01-schema-conversion P02]: Int64+int32 monetary representation follows google.type.Money pattern to avoid floating-point precision loss
- [Phase 01-schema-conversion]: Move TransactionResponse23 to common.proto to resolve circular dependency
- [Phase 01-schema-conversion]: Use include! macro for generated code integration (single file approach)
- [Phase 01-schema-conversion]: Keep unique message types in casp.001.proto (PaymentRequest29, etc.)
- [Phase 02-core-library P01]: Use core::error::Error (not std::error::Error) for no_std compatibility
- [Phase 02-core-library P01]: Manual Error trait implementation instead of thiserror (thiserror is std-only)
- [Phase 02-core-library P01]: defmt::Format derive only on NexoError, not ValidationError (String fields not supported)
- [Phase 02-core-library P01]: Feature flags follow additive principle: default = ["std"], no negative feature names
- [Phase 02-core-library P01]: &'static str for no_std, Box::leak for std convenience methods
- [Phase 02-core-library P02]: Codec trait requires Message + Default bound (prost::Message::decode needs Default)
- [Phase 02-core-library P02]: Size limits checked BEFORE encode/decode to prevent unbounded allocation
- [Phase 02-core-library P02]: 4MB default limit follows gRPC standard for maximum message size
- [Phase 02-core-library P02]: encode_to_vec() returns Vec<u8> directly, not Result (allocates properly sized buffer)
- [Phase 02-core-library P02]: Convenience functions use ProstCodec internally for ergonomic API
- [Phase 03-transport-layer P03-03]: Manual address parsing for Embassy's (IpAddress, u16) tuples
- [Phase 03-transport-layer P03-03]: EmbassyTimeoutConfig as separate type for embassy_time::Duration type safety
- [Phase 03-transport-layer P03-03]: Moderate timeout defaults (10s connect, 30s read, 10s write) for embedded environments
- [Phase 03-transport-layer P03-02]: Use tokio::time::timeout for all async operations to prevent hangs
- [Phase 03-transport-layer P03-02]: Import AsyncReadExt and AsyncWriteExt traits for read/write methods
- [Phase 03-transport-layer P03-02]: Add macros and rt-multi-thread features for tokio::test support
- [Phase 03-transport-layer P03-02]: peer_addr().is_ok() for connection state detection (simple but effective)
- [Phase 03-transport-layer P03-02]: Default timeouts: 10s connect, 30s read, 10s write
- [Phase 03-transport-layer P03-05]: Fix Embassy transport TcpSocket import path (embassy_net::tcp::TcpSocket for Embassy 0.8)
- [Phase 03-transport-layer P03-05]: Add smoltcp features (tcp, udp, dhcpv4, proto-ipv4, medium-ethernet) to embassy-net dependency
- [Phase 03-transport-layer P03-05]: Use split_once() instead of Vec for no_std address parsing
- [Phase 03-transport-layer P03-05]: Remove is_open() calls (API changed in Embassy 0.8)
- [Phase 03-transport-layer P03-05]: Add ToString import to test modules for no_std compatibility
- [Phase 03-transport-layer P03-05]: Gate tokio tests with #[cfg(all(test, feature = "std"))]
- [Phase 04-client-api]: Used Arc<AtomicBool> for thread-safe connection state in std environments
- [Phase 04-client-api]: Generic NexoClient over Transport trait enables single codebase for std and no_std
- [Phase 04-client-api]: Feature-gated NexoClient exports prevent conflicts when both std and embassy enabled
- [Phase 04-client-api P03]: rand v0.8 with default features for std::thread_rng() access in std environments
- [Phase 04-client-api P03]: EmbassyDuration type alias to avoid core::time::Duration conflicts in embassy
- [Phase 04-client-api P03]: Exponential backoff with jitter: delay * (1.0 + random(-0.2, 0.2)) for ±20% variation
- [Phase 04-client-api P03]: Runtime-specific backoff: wait_with_jitter() for std, wait_without_jitter() for embassy
- [Phase 04-client-api P04]: UUID v4 for std message IDs (122 bits entropy) vs timestamp-counter for no_std
- [Phase 04-client-api P04]: Late response rejection with warning log (std) or silent drop (no_std)
- [Phase 04-client-api P04]: Pending request cleanup on timeout prevents memory leaks in HashMap
- [Phase 04-client-api P04]: where T::Error: Into<NexoError> bound for timeout wrapper error conversion
- [Phase 05-server-api-reliability P05-01]: BTreeMap for connection tracking (sorted iteration for debugging)
- [Phase 05-server-api-reliability P05-01]: Async bind() method to match Tokio TcpListener::bind() signature
- [Phase 05-server-api-reliability P05-01]: NexoError::connection_owned() for dynamic error messages (leaks strings, acceptable for error paths)
- [Phase 05-server-api-reliability P05-01]: tokio::spawn per connection with automatic cleanup in async move block
- [Phase 05-server-api-reliability P05-01]: Echo placeholder in handle_connection() until message dispatcher added in 05-02
- [Phase 05-server-api-reliability]: Per-connection deduplication cache with 5-minute TTL for replay attack prevention
- [Phase 05-04]: Heartbeat protocol with tokio::select! for concurrent message handling and dead connection detection
- [Phase 05-04]: tokio::time::interval for precise heartbeat timing with drift correction
- [Phase 05-04]: 3:1 timeout-to-interval ratio (90s timeout, 30s interval) for robustness
- [Phase 05-04]: Per-connection heartbeat config via Option<HeartbeatConfig> for flexibility
- [Phase 05-05]: tracing framework (not log crate) for async-aware structured logging with span-based correlation
- [Phase 05-05]: env-filter in tracing-subscriber for RUST_LOG environment variable configuration
- [Phase 05-05]: Direct logging with context fields instead of entered() guards for tokio::spawn compatibility
- [Phase 05-05]: Feature-gated tracing code with #[cfg(feature = "std")] for no_std compatibility
- [Phase 05.1-01]: Wrap TcpStream in FramedTransport at connection accept for proper length-prefix framing
- [Phase 05.1-01]: dispatch_document() accepts decoded Casp001Document structs to eliminate double-encoding
- [Phase 05.1-01]: FramedTransport for all server message I/O ensures 4-byte length prefix protocol

### Pending Todos

None yet.

### Blockers/Concerns

**From Research:**
- Detailed CASP message structures need review during Phase 1 planning (MDRPart1/2/3 files in project archive)
- Nexo TCP framing specification needs detailed review during Phase 3 planning

**Current blockers:**
- None (server framing integration complete)

**Deferred items:**
- Embassy transport integration testing on actual hardware or QEMU (tests marked #[ignore] with instructions)

## Session Continuity

Last session: 2026-03-02 (Phase 5.1 execution)
Stopped at: Completed plan 05.1-01 - Integrate FramedTransport into Server
Next step: Execute Phase 6 - Testing & Verification
Summary file: .planning/phases/05.1-fix-server-framing/05.1-01-SUMMARY.md
