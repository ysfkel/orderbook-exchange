# Low-Latency Crypto Exchange

A from-scratch, production-architecture crypto exchange built in Rust. The goal is to apply the same design decisions used in real low-latency trading systems: deliberate transport selection per path, zero-copy wire formats, lock-free intra-process communication, and strict separation of the hot and cold paths.

> **Work in progress.** Most crates have their structure and interfaces defined but are not yet fully implemented. The project is being built incrementally to explore and document the design decisions along the way.

---

## Architecture

Three planes of communication, each with a transport chosen for its role:

```
  [Client]
     │
     ▼ HTTP / WebSocket  (not yet implemented)
  ┌──────────┐
  │ Gateway  │  ── order entry, session management
  └──────────┘
        │
        │  Aeron IPC  (low-latency message bus)
        ▼
  ┌──────────┐        ┌──────────────┐
  │   OMS    │        │     Risk     │
  │ (orders) │        │   (checks)   │
  └──────────┘        └──────────────┘
        │
        │  Aeron IPC
        ▼
  ┌──────────────┐
  │    Engine    │  ── price-time priority matching
  └──────────────┘

  ┌─────────────┐
  │ Instruments │  ── reference data (markets, assets)
  │  Postgres   │  ── gRPC (cold path, loaded at startup)
  └─────────────┘
```

---

## Crates

### `oms` — Order Management System
The central routing layer. Listens for inbound orders from the gateway over Aeron, validates them, coordinates with the risk service, and forwards accepted orders to the matching engine. Uses an SPSC ring buffer (`rtrb`) to hand messages off from the Aeron subscriber thread to the processing thread without locking.

- Inbound: Aeron subscriber with a reconnecting listener loop
- Outbound: Aeron publisher to engine
- Storage: order state (not yet implemented)

### `engine` — Matching Engine
Receives accepted orders from the OMS and runs price-time priority matching. Each market has its own orderbook. Wire frames arrive as `#[repr(C)]` zero-copy structs — no deserialization step on the hot path.

- Orderbook: price levels with FIFO order queues (scaffolded)
- Matching: limit order matching (in progress)
- Network: Aeron subscriber with reconnect logic

### `instruments` — Reference Data Service
Owns the market and asset catalog. Persists to Postgres via `sqlx`, seeds from a `markets.toml` config file, and exposes the catalog over gRPC. Other services load reference data once at startup — this is a cold-path control plane service, not on the order flow.

- gRPC server (tonic)
- Postgres with compile-time checked queries (sqlx)
- `market_index`: a dense, position-stable integer used as the cross-service market identifier

### `gateway` — Order Entry (stub)
Will be the external-facing entry point: HTTP REST and WebSocket for clients to submit orders and receive execution reports. Currently a hardcoded one-shot test sender used to validate the Aeron pipeline.

### `risk` — Pre-trade Risk (stub)
Will sit between the OMS and engine to perform pre-trade risk checks (position limits, notional caps, etc.) before an order is forwarded for matching.

### `transport` — Aeron Abstraction
A thin wrapper over `rusteron-client` that provides `AeronTransport`, `Publisher`, and `Subscriber` traits. Centralises connection logic, reconnect behaviour, and poll error handling so individual services don't depend on the raw Aeron API.

### `common` — Shared Wire Types
Zero-copy structs shared by all services on the order path. The `OrderDTO`, `NewOrder`, and message header types are `#[repr(C)]` with explicit padding — both producer and consumer cast raw bytes directly to these types, so field layout is a hard contract. Also exposes `QueueProducer` / `QueueConsumer` traits and the `rtrb`-backed ring buffer implementation.

### `shared-protos` — Protobuf / gRPC Definitions
Contains the `.proto` definitions and the `tonic`-generated Rust code for all gRPC services. Generated code is checked in; the build script regenerates it on `cargo build`.

### `markets` — (superseded by `instruments`)
An earlier iteration of the reference data service. Being phased out in favour of `instruments`.

---

## Technology Choices

| Technology | Role | Why |
|---|---|---|
| **Aeron** (`rusteron-client`) | Service-to-service message bus | Designed for low-latency IPC and network transport. Operates with a pre-allocated log buffer — no heap allocation on the critical path. Supports multicast and IPC transports. A standard choice in HFT and exchange infrastructure. |
| **rtrb** | Intra-process handoff (Aeron thread → processing thread) | Lock-free single-producer single-consumer ring buffer. Zero contention, cache-friendly, no `Arc<Mutex<>>` on the hot path. Used to cross the thread boundary inside a single process. |
| **zerocopy** | Wire frame deserialization | Validates and casts raw bytes to typed structs with no copying. Combined with `#[repr(C)]` structs, this means inbound messages are "deserialized" in a single pointer cast. |
| **tonic / gRPC** | Control plane (reference data) | Used only for the cold path — loading market config at startup. Ergonomic, well-supported, correct choice for request-response reference data. |
| **sqlx** | Postgres persistence | Compile-time checked queries against a cached schema. Catches SQL errors at build time, not runtime. |
| **tokio** | Async runtime | Powers the gRPC services and gateway. The hot-path services (engine, OMS) run synchronous polling loops — no async on the critical path. |
| **thiserror** | Error types | Structured, per-crate error enums with `From` impls for clean propagation. |
| **tracing** | Structured logging | Used on the cold path only. Hot-path events are counted with atomics and flushed by a background thread — no I/O on the order path. |

---

## Key Design Decisions

**Two identifiers per market.** A DB-internal `id` (never leaves `instruments`) and a `market_index` — a dense, position-stable integer used everywhere else. The engine indexes directly into `Vec<OrderBook>` by `market_index`, so it must never have gaps or change order. The validator enforces this at startup.

**Two transports, one per path.** gRPC for the control plane (cold, ergonomic, typed). Aeron for the data plane (hot, zero-copy, low-latency). Using gRPC for order flow would add serialization cost on every message; using Aeron for reference data would be unnecessary complexity.

**No I/O on the hot path.** Logging a dropped order or a malformed message on every message would destroy latency. Hot-path events increment atomics; a background monitoring thread reads and logs them on a periodic interval.

---

## Status

| Crate | Status |
|---|---|
| `transport` | Working — Aeron connect, publish, subscribe, reconnect |
| `instruments` | Working — gRPC server, Postgres persistence, seed from TOML |
| `oms` | In progress — inbound listener working, service layer scaffolded |
| `engine` | In progress — network layer working, orderbook scaffolded |
| `common` | Working — wire types, ring buffer, queue traits |
| `shared-protos` | Working |
| `gateway` | Stub — one-shot test sender only |
| `risk` | Stub |
