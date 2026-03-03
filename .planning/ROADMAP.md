# Roadmap: Nexo Retailer Protocol (Rust)

## Milestones

- ✅ **v1.0 MVP** — Phases 1-6 + 5.1 (shipped 2026-03-03)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-6 + 5.1) — SHIPPED 2026-03-03</summary>

- [x] Phase 1: Schema Conversion (3/3 plans) — completed 2026-02-28
- [x] Phase 2: Core Library (4/4 plans) — completed 2026-03-01
- [x] Phase 3: Transport Layer (6/6 plans) — completed 2026-03-02
- [x] Phase 4: Client API (5/5 plans) — completed 2026-03-02
- [x] Phase 5: Server API & Reliability (6/6 plans) — completed 2026-03-02
- [x] Phase 5.1: Fix Server Framing (1/1 plan) — completed 2026-03-02
- [x] Phase 6: Testing & Verification (8/8 plans) — completed 2026-03-03

**Key accomplishments:**
- 17 CASP message types converted to protobuf
- Codec, validation, error handling with no_std support
- FramedTransport with 4-byte length-prefix framing
- Client API with builders, reconnection, timeouts
- Server API with concurrent connections, heartbeat, deduplication
- 400+ tests with CI for std and bare-metal targets

See `.planning/milestones/v1.0-ROADMAP.md` for full details.

</details>

---

## Next Milestone

Run `/gsd:new-milestone` to start planning v2.

---

*Last updated: 2026-03-03 after v1.0 milestone completion*
