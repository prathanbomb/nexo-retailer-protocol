---
phase: "03"
plan: "01"
subsystem: "Transport Layer"
tags: ["transport", "trait", "framing", "tcp", "no_std"]

dependency_graph:
  requires: []
  provides: ["Transport trait", "FramedTransport wrapper"]
  affects: ["03-02", "03-03"]

tech_stack:
  added: ["Tokio", "Embassy dependencies"]
  patterns: ["Runtime-agnostic traits", "Length-prefixed framing", "no_std compatibility"]

key_files:
  created: ["src/transport/mod.rs", "src/transport/framing.rs"]
  modified: ["src/lib.rs", "Cargo.toml"]

decisions:
  - id: "TRANS-TRAIT-001"
    title: "Use native async traits (Rust 1.75+)"
    rationale: "Project MSRV is 1.85, so async_trait crate is not needed. Native async traits provide better performance and ergonomics."
    alternatives: ["async_trait crate", "Manual futures with Pin"]
  - id: "TRANS-FRAMING-001"
    title: "4-byte big-endian length prefix framing"
    rationale: "Standard protobuf-over-TCP convention. 4 bytes allows messages up to 4GB, enforced 4MB limit for security."
    alternatives: ["Varint length prefix", "Delimiter-based framing"]
  - id: "TRANS-ERROR-001"
    title: "Use &'static str for NexoError in no_std"
    rationale: "format! macro not available in no_std. Static strings ensure compatibility across all environments."
    alternatives: ["Box::leak for dynamic errors", "Heapless format strings"]
  - id: "TRANS-ALLOC-001"
    title: "Gate recv_message behind alloc feature"
    rationale: "recv_message requires heap allocation for message buffer. Framing layer works without alloc for send-only scenarios."
    alternatives: ["Stack-based buffers", "Require alloc for entire transport module"]
  - id: "TRANS-TEST-001"
    title: "Use ActiveCurrencyAndAmount for framing tests"
    rationale: "Simple message type with required fields. prost_types::Any has decoding issues with arbitrary data."
    alternatives: ["Header4 (all optional fields)", "Custom test message type"]

metrics:
  duration: 663
  completed_date: "2026-03-01T15:47:21Z"

---

# Phase 03 Plan 01: Define Transport Trait and Custom TCP Framing Protocol Summary

## Overview

Implemented runtime-agnostic transport trait and length-prefixed TCP framing layer that works across both Tokio (std) and Embassy (no_std) async runtimes, enabling protocol message transmission over TCP connections with proper message boundary handling.

**One-liner:** Async Transport trait with 4-byte big-endian length-prefixed TCP framing, runtime-agnostic for Tokio/Embassy, enforcing 4MB message size limits.

## Implementation Summary

### Tasks Completed

1. **Create transport module structure and define Transport trait** (6ad7c91)
   - Defined native async `Transport` trait with `read()`, `write()`, `connect()`, `is_connected()` methods
   - Used associated `Error` type for runtime-specific error flexibility
   - Used `core::time::Duration` for no_std compatibility
   - Added comprehensive documentation with examples

2. **Implement length-prefixed TCP framing logic** (c3b524c)
   - Implemented `FramedTransport<T>` wrapper for any `Transport` implementation
   - Added `send_message()` and `recv_message()` with 4-byte big-endian length prefix
   - Enforced 4MB `MAX_FRAME_SIZE` limit from codec layer
   - Implemented `write_all()` and `read_exact()` helpers for complete I/O
   - Added comprehensive unit tests (8 test cases)
   - Added prost-types and futures-executor dev dependencies

3. **Add transport module to lib.rs and update Cargo.toml** (e95c984)
   - Exported transport module from lib.rs with re-exports
   - Added optional Tokio dependencies (tokio, tokio-util)
   - Added optional Embassy dependencies (embassy-executor, embassy-time, embassy-net, embassy-futures)
   - Fixed embassy-net version from 0.9 to 0.8 (actual available version)
   - Fixed NexoError usage to use `&'static str` instead of `format!` for no_std compatibility

4. **Write unit tests for framing logic** (d849172)
   - Fixed critical bug: added missing `read_exact()` call in `recv_message()`
   - Fixed test message types: use `ActiveCurrencyAndAmount` instead of `prost_types::Any`
   - All 9 framing tests pass
   - Verified both std and no_std builds

### Files Created

- `src/transport/mod.rs` - Transport trait definition with documentation
- `src/transport/framing.rs` - FramedTransport implementation with tests

### Files Modified

- `src/lib.rs` - Added transport module export and documentation
- `Cargo.toml` - Added Tokio and Embassy optional dependencies

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed recv_message missing read_exact call**
- **Found during:** Task 4
- **Issue:** `recv_message()` was not calling `read_exact()` to read message body into buffer, causing decode failures
- **Fix:** Added `self.read_exact(&mut buffer).await?` after buffer allocation
- **Files modified:** `src/transport/framing.rs`
- **Commit:** d849172

**2. [Rule 1 - Bug] Fixed prost_types::Any decoding failures**
- **Found during:** Task 4
- **Issue:** `prost_types::Any` cannot decode arbitrary protobuf data, causing test failures
- **Fix:** Changed tests to use `ActiveCurrencyAndAmount` which has a stable encoding
- **Files modified:** `src/transport/framing.rs`
- **Commit:** d849172

**3. [Rule 1 - Bug] Fixed format! macro usage in no_std**
- **Found during:** Task 2
- **Issue:** `format!` macro not available in no_std, causing compilation failures
- **Fix:** Replaced `format!()` calls with static `&'static str` error messages
- **Files modified:** `src/transport/framing.rs`, `src/transport/mod.rs`
- **Commit:** e95c984

**4. [Rule 3 - Blocking] Fixed vec! macro availability in no_std**
- **Found during:** Task 3
- **Issue:** `vec!` macro not available in no_std without alloc feature
- **Fix:** Added `#[cfg(any(feature = "std", feature = "alloc"))]` to `recv_message()` and added alloc imports
- **Files modified:** `src/transport/framing.rs`
- **Commit:** d849172 (via fix iteration)

**5. [Rule 2 - Missing] Added futures-executor import for tests**
- **Found during:** Task 2
- **Issue:** Tests use `block_on()` but futures-executor was not imported
- **Fix:** Added `use futures_executor::block_on;` to test module and replaced `futures::executor::block_on` with `block_on`
- **Files modified:** `src/transport/framing.rs`
- **Commit:** d849172 (via fix iteration)

**6. [Rule 1 - Bug] Fixed embassy-net version**
- **Found during:** Task 3
- **Issue:** Embassy-net version 0.9 does not exist in crates.io
- **Fix:** Changed embassy-net version from 0.9 to 0.8
- **Files modified:** `Cargo.toml`
- **Commit:** e95c984

**7. [Rule 1 - Bug] Fixed .to_string() usage in error construction**
- **Found during:** Task 3
- **Issue:** NexoError expects `&'static str` but code used `.to_string()`
- **Fix:** Removed `.to_string()` calls and used static strings directly
- **Files modified:** `src/transport/framing.rs`, `src/transport/mod.rs`
- **Commit:** e95c984

### Auth Gates

None encountered during this plan execution.

## Verification Results

### Success Criteria

- [x] `Transport` trait defined with async `read()`, `write()`, `connect()`, and `is_connected()` methods
- [x] `FramedTransport` wrapper implements length-prefixed framing with 4-byte big-endian length header
- [x] Framing layer enforces 4MB message size limit from codec layer
- [x] All code compiles with both `std` and `no_std` feature flags
- [x] Unit tests verify framing logic (encode/decode length prefix, handle partial reads)

### Test Results

```
running 9 tests
test transport::framing::tests::test_framing_module_exists ... ok
test transport::framing::tests::test_oversized_length_prefix_rejected ... ok
test transport::framing::tests::test_empty_message ... ok
test transport::framing::tests::test_length_prefix_big_endian ... ok
test transport::framing::tests::test_multiple_messages_sequential ... ok
test transport::framing::tests::test_oversized_message_rejected ... ok
test transport::framing::tests::test_partial_read_handling ... ok
test transport::framing::tests::test_zero_length_prefix ... ok
test transport::framing::tests::test_encode_decode_normal_message ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out
```

### Build Verification

- `cargo build --features std` - SUCCESS
- `cargo build --no-default-features` - SUCCESS
- `cargo build --features alloc` - SUCCESS
- `cargo test --features std,alloc` - ALL TESTS PASS

## Key Decisions

### TRANS-TRAIT-001: Use Native Async Traits
Chose native async traits (Rust 1.75+) over `async_trait` crate because:
- Project MSRV is 1.85, so native support is guaranteed
- Better performance (no boxing)
- More ergonomic API
- Simpler dependency management

### TRANS-FRAMING-001: 4-Byte Big-Endian Length Prefix
Selected standard protobuf-over-TCP framing:
- 4-byte length prefix allows messages up to 4GB
- Big-endian byte order for network protocol standard
- Enforced 4MB limit at application layer for security
- Widely supported across platforms and languages

### TRANS-ERROR-001: Static Strings for no_std Compatibility
Used `&'static str` instead of `format!` or dynamic strings:
- `format!` macro not available in no_std
- Ensures compatibility across all environments
- Trade-off: Less detailed error messages in no_std
- Acceptable for embedded systems where error messages are less critical

### TRANS-ALLOC-001: Conditional Compilation for recv_message
Gated `recv_message()` behind `alloc` feature:
- Send-only scenarios work without alloc
- Receive requires heap allocation for message buffer
- Graceful compilation error if used without alloc
- Clear API contract for users

## Performance Metrics

- **Total tasks completed:** 4
- **Total commits:** 4
- **Files created:** 2
- **Files modified:** 2
- **Lines added:** ~600
- **Test coverage:** 9 tests, all passing
- **Execution time:** 11 minutes (663 seconds)

## Next Steps

This plan establishes the foundation for Plan 03-02 (Tokio Transport Implementation) and Plan 03-03 (Embassy Transport Implementation), which will provide concrete runtime-specific implementations of the `Transport` trait.

## Self-Check: PASSED

✅ All tasks completed and committed
✅ SUMMARY.md created with substantive content
✅ All success criteria met
✅ All tests passing
✅ Both std and no_std builds verified
✅ Deviations documented
