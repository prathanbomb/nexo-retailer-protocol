---
phase: 02-core-library
plan: 01
subsystem: error-handling
tags: [no_std, defmt, error-types, feature-flags, core::error::Error]

# Dependency graph
requires:
  - phase: 01-schema-conversion
    provides: [Generated protobuf code, prost Message trait]
provides:
  - NexoError enum with core::error::Error implementation for no_std compatibility
  - ValidationError enum with field-specific validation error variants
  - Feature flag architecture (std, alloc, defmt) following additive principle
  - Error code constants organized by category (connection, timeout, validation, encoding, decoding)
  - defmt::Format derive support for NexoError (embedded logging)
affects: [02-core-library/02-02, 02-core-library/02-03, 02-core-library/02-04, 03-transport-layer]

# Tech tracking
tech-stack:
  added: [defmt 0.3, heapless 0.8]
  patterns: [core::error::Error for no_std, additive feature flags, conditional defmt derives]

key-files:
  created: [src/error/mod.rs, src/error/codes.rs, src/features/mod.rs, src/features/std.rs, src/features/no_std.rs]
  modified: [src/lib.rs, Cargo.toml]

key-decisions:
  - "Use core::error::Error (not std::error::Error) for no_std compatibility"
  - "Manual Error trait implementation instead of thiserror (thiserror is std-only)"
  - "defmt::Format derive only on NexoError, not ValidationError (String fields not supported)"
  - "Simplified doc comments to avoid defmt parsing issues with words like 'typically' and 'due to'"
  - "Feature flags follow additive principle: default = [\"std\"], no --no-default-features for no_std"

patterns-established:
  - "Pattern 1: core::error::Error for no_std compatible errors"
  - "Pattern 2: Additive feature flags (std, alloc, defmt) with no negative feature names"
  - "Pattern 3: Conditional derives using #[cfg_attr(feature = \"defmt\"), derive(defmt::Format))]"
  - "Pattern 4: &'static str for no_std, Box::leak for std convenience methods"

requirements-completed: [ERROR-01, ERROR-02, ERROR-03, ERROR-04, PLAT-01, PLAT-02, PLAT-03]

# Metrics
duration: 5min
completed: 2026-02-28
---

# Phase 2 Plan 1: Error Handling and Feature Flags Summary

**no_std-compatible error types with core::error::Error implementation, additive feature flag architecture (std/alloc/defmt), and defmt logging support for embedded environments**

## Performance

- **Duration:** 5 minutes
- **Started:** 2026-02-28T16:34:34Z
- **Completed:** 2026-02-28T16:39:56Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Implemented no_std-compatible NexoError enum with 5 variants (Connection, Timeout, Validation, Encoding, Decoding)
- Created comprehensive ValidationError enum with 6 variants for field-specific validation (MissingRequiredField, InvalidCurrencyFormat, InvalidCurrencyLength, StringTooLong, NanosOutOfRange, NanosSignMismatch)
- Established additive feature flag architecture: std (default), alloc, defmt
- Added defmt 0.3 and heapless 0.8 as optional dependencies
- Implemented core::error::Error (not std::error::Error) for no_std compatibility
- Added From<prost::DecodeError> and From<prost::EncodeError> conversions for easy error handling
- Created error code constants organized by category in codes.rs
- Added std-specific convenience methods for creating errors from dynamic strings

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement no_std-compatible error types** - `c03ebef` (feat)
2. **Task 2: Establish platform feature flag architecture** - `e102936` (feat)
3. **Task 3: Add error module integration and documentation** - `7502616` (feat)

**Plan metadata:** `docs(02-01): complete plan` (not yet created)

## Files Created/Modified

### Created
- `src/error/mod.rs` - NexoError and ValidationError enums with core::error::Error impls, Display impls, From impls, std convenience methods, comprehensive documentation
- `src/error/codes.rs` - Error code constants organized by category (0xxx connection, 1xxx timeout, 2xxx validation, 3xxx encoding, 4xxx decoding), helper functions for category checking
- `src/features/mod.rs` - Feature flag module documentation, module organization
- `src/features/std.rs` - std-specific implementations placeholder (Tokio transport for Phase 3)
- `src/features/no_std.rs` - no_std-specific implementations placeholder (Embassy transport for Phase 3)

### Modified
- `Cargo.toml` - Added defmt and heapless optional dependencies, added defmt feature flag
- `src/lib.rs` - Added error and features module exports, re-exported error types at crate root, updated documentation with error handling section

## Decisions Made

1. **core::error::Error over std::error::Error**: Required for no_std compatibility since Rust 1.81 stabilization. Manual Error trait implementation instead of thiserror (which is std-only).

2. **defmt::Format only on NexoError**: ValidationError contains String fields which defmt doesn't support even with alloc. Defmt derive is conditional on feature flag only for NexoError.

3. **Simplified doc comments**: Removed words like "typically", "due to", "corruption" from variant-level doc comments because defmt derive macro was incorrectly parsing them as enum variants.

4. **Additive feature flags**: Used default = ["std"] with --no-default-features for no_std instead of negative feature names like "no_std". Follows Rust ecosystem best practices.

5. **&'static str for no_std**: Error variants use &'static str for compatibility with no_std. Std convenience methods use Box::leak to convert owned strings, which is acceptable for error paths.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed defmt parsing issue with doc comments**
- **Found during:** Task 1 (error type implementation)
- **Issue:** defmt::Format derive macro was parsing words from doc comments ("typically", "due to", "corruption") as enum variants, causing compilation errors
- **Fix:** Simplified variant-level doc comments to remove problematic words, keeping only brief descriptions
- **Files modified:** src/error/mod.rs
- **Verification:** Compilation succeeds with defmt feature enabled
- **Committed in:** c03ebef (Task 1 commit)

**2. [Rule 1 - Bug] Removed prost error constructor tests**
- **Found during:** Task 1 (unit tests)
- **Issue:** prost::DecodeError::new() and prost::EncodeError::new() are private functions, cannot be constructed in tests
- **Fix:** Replaced direct construction tests with compile-check tests that verify From impls exist via function signatures
- **Files modified:** src/error/mod.rs
- **Verification:** All tests pass
- **Committed in:** c03ebef (Task 1 commit)

**3. [Rule 2 - Missing Critical] Fixed defmt::Format on ValidationError**
- **Found during:** Task 2 (feature flag verification)
- **Issue:** ValidationError contains String fields which defmt doesn't support even with alloc feature enabled
- **Fix:** Removed defmt::Format derive from ValidationError, kept it only on NexoError which uses &'static str
- **Files modified:** src/error/mod.rs
- **Verification:** cargo build --features defmt,alloc succeeds
- **Committed in:** e102936 (Task 2 commit)

**4. [Rule 1 - Bug] Fixed doctest examples**
- **Found during:** Task 2 (test verification)
- **Issue:** Doctests failed because ValidationError::MissingRequiredField requires String but example provided &str, and prost error constructors are private
- **Fix:** Updated doctests to use .to_string() for ValidationError fields, removed prost error construction example from NexoError doctest
- **Files modified:** src/error/mod.rs
- **Verification:** cargo test passes all doctests
- **Committed in:** e102936 (Task 2 commit)

**5. [Rule 1 - Bug] Fixed features module names**
- **Found during:** Task 2 (build verification)
- **Issue:** features/mod.rs referenced modules std_impl and no_std_impl but files were named std.rs and no_std.rs
- **Fix:** Changed module declarations from `std_impl` to `std` and `no_std_impl` to `no_std`
- **Files modified:** src/features/mod.rs
- **Verification:** cargo build --no-default-features succeeds
- **Committed in:** e102936 (Task 2 commit)

---

**Total deviations:** 5 auto-fixed (4 bugs, 1 missing critical)
**Impact on plan:** All auto-fixes necessary for compilation correctness. No scope creep. Plan execution completed successfully.

## Issues Encountered

1. **defmt derive macro parsing doc comments**: The defmt::Format derive macro was overly aggressive in parsing doc comments, treating common English words as enum variants. Fixed by simplifying doc comments.

2. **prost error constructors are private**: Cannot construct prost::DecodeError and prost::EncodeError in tests due to private new() methods. Worked around with compile-check tests.

3. **String fields incompatible with defmt**: ValidationError uses String for dynamic field names/codes, which defmt doesn't support. Decision to not derive defmt::Format for ValidationError is documented in module docs.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Error handling foundation complete and tested in both std and no_std configurations
- Feature flag architecture established for codec (02-02), validation (02-03), and CI (02-04) plans
- ValidationError enum ready for use in Phase 02-03 validation layer
- NexoError provides std convenience methods for server environments while maintaining no_std compatibility
- defmt logging support available for embedded environments

**No blockers.** Ready to proceed to Plan 02-02 (Codec Layer).

---
*Phase: 02-core-library*
*Completed: 2026-02-28*
