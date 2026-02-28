# Nexo Retailer Protocol (Rust)

## What This Is

A Rust library implementing the Nexo Retailer Protocol (ISO 20022 CASP - Cardholder Account Status Poll) specification. This library provides protobuf-based serialization, message validation, custom TCP transport, and client/server APIs for POS terminals and payment devices.

The protocol specification is located at `archive_retailer_protocol_6_2f8950ad21/` with 17 XSD schema files (casp.001 through casp.017) that will be converted to protobuf schemas.

## Core Value

Enable embedded payment devices to communicate using the Nexo Retailer Protocol with a Rust implementation that works in both bare metal (no_std) and standard environments.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Convert all 17 XSD schemas to protobuf (.proto) files
- [ ] Implement protobuf serialization/deserialization for all message types
- [ ] Implement message validation according to spec constraints
- [ ] Implement custom TCP transport layer
- [ ] Implement Client API for initiating payment transactions
- [ ] Implement Server API for handling payment requests
- [ ] Support std environment with Tokio async runtime
- [ ] Support no_std bare metal environment with feature flags

### Out of Scope

- XML/XSD serialization (protobuf only) — project explicitly uses protobuf for efficiency
- Other ISO 20022 message families beyond CASP — focus on Nexo Retailer Protocol only
- Hardware-specific integrations — library is platform-agnostic

## Context

- **Protocol Source:** ISO 20022 CASP specification (version 6) with 17 message type schemas
- **Reference Implementation:** None identified — this is a greenfield implementation
- **Target Domain:** Payment processing, POS terminals, cardholder account status polling
- **Message Types:** 17 distinct CASP message types covering authorization, reversal, reconciliation, etc.

## Constraints

- **Platform:** Must support both `std` (with Tokio) and `no_std` (bare metal) environments via Cargo feature flags
- **Serialization:** Protobuf only — no XML support required
- **Transport:** Custom TCP protocol (not HTTP/2 or gRPC)
- **Memory:** Suitable for resource-constrained embedded devices
- **License:** (To be determined)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Protobuf over XML | Smaller message sizes, faster serialization, better for embedded | — Pending |
| Dual std/no_std support | POS terminals may be bare metal; servers use standard Rust | — Pending |
| Tokio for std async | Industry standard, well-supported ecosystem | — Pending |
| Custom TCP transport | Nexo protocol uses its own framing, not HTTP | — Pending |

---
*Last updated: 2026-02-28 after initialization*
