# Nexo Retailer Protocol (Rust)

## What This Is

A Rust library implementing the Nexo Retailer Protocol (ISO 20022 CASP - Cardholder Account Status Poll) specification. This library provides protobuf-based serialization, message validation, custom TCP transport, and client/server APIs for POS terminals and payment devices. **v1.0 shipped** with full std and no_std support, 400+ tests, and complete E2E communication.

The protocol specification is located at `archive_retailer_protocol_6_2f8950ad21/` with 17 XSD schema files (casp.001 through casp.017) converted to protobuf schemas.

## Core Value

Enable embedded payment devices to communicate using the Nexo Retailer Protocol with a Rust implementation that works in both bare metal (no_std) and standard environments.

## Requirements

### Validated (v1.0)

- ✓ All 17 CASP XSD files converted to .proto format — v1.0
- ✓ Field number registry with reserved tracking — v1.0
- ✓ Protobuf codec with 4MB size limits — v1.0
- ✓ Validation for ISO 4217, XSD constraints — v1.0
- ✓ no_std error types with defmt support — v1.0
- ✓ FramedTransport with 4-byte length prefix — v1.0
- ✓ TokioTransport (std) and EmbassyTransport (no_std) — v1.0
- ✓ NexoClient with builders, reconnection, timeouts — v1.0
- ✓ NexoServer with concurrent connections, heartbeat, deduplication — v1.0
- ✓ 400+ tests (unit, integration, E2E, property-based) — v1.0
- ✓ CI for std and bare-metal (thumbv7em) targets — v1.0

### Active

(None — v1.0 complete, await v2 planning)

### Out of Scope

- XML/XSD serialization (protobuf only) — project explicitly uses protobuf for efficiency
- Other ISO 20022 message families beyond CASP — focus on Nexo Retailer Protocol only
- Hardware-specific integrations — library is platform-agnostic
- HTTP/REST wrapper — Nexo uses custom TCP framing
- Synchronous blocking API — async-first design
- Built-in hardware crypto — accept crypto trait objects
- GUI/CLI tools — separate nexo-cli crate if needed

## Context

**Shipped v1.0** (2026-03-03) with:
- 28,214 lines of Rust code
- 7 phases, 33 plans executed
- 400+ tests passing
- CI workflows for std and bare-metal targets

**Tech Stack:**
- Runtime: Tokio (std), Embassy (no_std)
- Serialization: prost (protobuf)
- Logging: tracing (std), defmt (no_std)
- Targets: x86_64, thumbv7em-none-eabihf

**Key Architectural Decisions:**
- Protobuf over XML for smaller messages, faster serialization
- Dual std/no_std support via Cargo feature flags
- BTreeMap for all map fields (HashMap needs Random trait unavailable in no_std)
- Int64+int32 monetary representation (no floating-point)
- FramedTransport for 4-byte length-prefix framing
- Per-connection deduplication with 5-minute TTL
- 3:1 heartbeat timeout-to-interval ratio (90s/30s)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Protobuf over XML | Smaller message sizes, faster serialization, better for embedded | ✓ Good |
| Dual std/no_std support | POS terminals may be bare metal; servers use standard Rust | ✓ Good |
| Tokio for std async | Industry standard, well-supported ecosystem | ✓ Good |
| Custom TCP transport | Nexo protocol uses its own framing, not HTTP | ✓ Good |
| BTreeMap for maps | HashMap needs Random trait unavailable in no_std | ✓ Good |
| Int64+int32 monetary | Follows google.type.Money pattern, avoids floating-point precision loss | ✓ Good |
| FramedTransport wrapper | 4-byte big-endian length prefix for message boundaries | ✓ Good |
| Per-connection deduplication | 5-minute TTL prevents replay attacks without global state | ✓ Good |
| tracing + defmt | Async-aware structured logging with embedded support | ✓ Good |

## Constraints

- **Platform:** Must support both `std` (with Tokio) and `no_std` (bare metal) environments via Cargo feature flags
- **Serialization:** Protobuf only — no XML support required
- **Transport:** Custom TCP protocol (not HTTP/2 or gRPC)
- **Memory:** Suitable for resource-constrained embedded devices
- **License:** (To be determined)

---
*Last updated: 2026-03-03 after v1.0 milestone completion*
