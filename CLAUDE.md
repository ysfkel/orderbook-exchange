# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A learning-focused, low-latency crypto exchange built as a Cargo workspace. Three binaries (`markets`, `engine`, `gateway`) plus two library crates (`common`, `shared-protos`). The codebase is in active development and parts of it (orderbook, gateway HTTP/WS surface, persistence beyond markets) are stubs or commented-out scaffolding — do not assume a feature exists just because a type is defined.

## Common commands

```bash
# build / check the whole workspace
cargo build
cargo check
cargo clippy --workspace -- -D warnings

# run one binary
cargo run -p markets       # starts gRPC server on 127.0.0.1:50051
cargo run -p engine        # joins UDP multicast group 239.1.1.1:9001
cargo run -p gateway       # currently a one-shot test sender

# tests
cargo test --workspace
cargo test -p markets -- --nocapture
cargo test -p markets config::load_markets::test::test_load_markets
```

### Database (markets crate only)

Postgres must be reachable at the URL in `crates/markets/.env`. On `cargo run -p markets` the binary auto-creates the DB if missing, runs migrations, and seeds from `crates/markets/markets.toml`.

```bash
# new migration
cd crates/markets && sqlx migrate add <name>
cd crates/markets && sqlx migrate add -r <name>   # -r = reversible

# regenerate sqlx offline schema cache after changing any query! or migration
cd crates/markets && cargo sqlx prepare
git add .sqlx/

# nuke and reseed
psql -d postgres -c "DROP DATABASE markets WITH (FORCE);"
cargo run -p markets
```

`SQLX_OFFLINE=true` is forced for all workspace builds via [.cargo/config.toml](.cargo/config.toml), so the `.sqlx/` cache **must** be committed and current for the project to compile without a live DB. If a `query_as!` macro changes, regenerate the cache before committing.

See [crates/markets/psql.md](crates/markets/psql.md) for the full psql cheat sheet.

## Architecture

Three processes talk to each other over **two different transports** chosen deliberately:

```
            ┌─────────────┐   gRPC (tonic)        ┌──────────┐
            │  markets    │ ◄──────────────────── │ gateway  │
            │ (Postgres)  │   GetMarkets,         │          │
            │  port 50051 │   GetMarketsIds       │          │
            └─────────────┘                       └──────────┘
                                                       │
                                                       │ UDP multicast 239.1.1.1:9001
                                                       │ zero-copy #[repr(C)] frames
                                                       ▼
                                                  ┌──────────┐
                                                  │  engine  │
                                                  └──────────┘
```

- **markets ↔ gateway: gRPC** — control plane, cold path. Reference data only. The gateway loads the catalog once at startup.
- **gateway → engine: UDP multicast + zerocopy** — order flow, hot path. Frames are `#[repr(C)]` `zerocopy::IntoBytes` structs sent without serialization (see [common/src/types/dto.rs](crates/common/src/types/dto.rs) for the exact padding/alignment contract — every field offset is hand-calculated and the struct size must stay a multiple of 16).

### Wire-format invariants (do not break)

The `OrderDTO` and `CreateOrder` structs in [common/src/types](crates/common/src/types/) are pinned to specific byte layouts:

- `#[repr(C)]` + explicit `_padding` fields — both producer (gateway) and consumer (engine) `try_ref_from_bytes` over raw UDP bytes. Any reorder, type change, or removed padding breaks the wire format silently.
- Total size of `CreateOrder` is 128 bytes; `CreateOrderMessage` is 32-byte header + body. The doc-comment header on `OrderDTO` lists the canonical offset table — update it if you ever change the struct.
- Numeric fields are pre-scaled integers (`i64` with separate `price_scale` / `qty_scale`). Do not put `rust_decimal::Decimal` on the wire.

### Market identifier discipline

Two identifiers exist per market and they are **not** interchangeable:

- `id` (`SERIAL` PK) — DB-internal only. Never leaves the `markets` service.
- `market_index` (declared in `markets.toml`, validated to equal its position) — the cross-service contract. Dense `0..N`, used as a vector index by the engine and (eventually) as the only market identifier on the UDP wire.

The validator at [crates/markets/src/config/load_markets.rs](crates/markets/src/config/load_markets.rs) enforces `market_index == position_in_toml`. Preserve that invariant — the engine will index into `Vec<OrderBook>` directly. See [.docs/market-id-vs-market-index.md](.docs/market-id-vs-market-index.md) for the full rationale.

### Per-crate roles

- **`crates/markets`** — gRPC service owning the market catalog. Persists to Postgres via `sqlx`. The `services::MarketsService` impl is the gRPC surface; `repositories/` does DB I/O behind a `Repository<T>` + `MarketRepository` trait split. Seed source of truth is [markets.toml](crates/markets/markets.toml).
- **`crates/engine`** — UDP multicast listener + (planned) orderbook matcher. `network/listener.rs` is the reconnecting receive loop, `network/message_handler.rs` is where dispatch on `MessageHeader.message_type` happens. Orderbook code under `engine/` is currently commented-out scaffolding.
- **`crates/gateway`** — order producer. Currently a hardcoded one-shot test that builds a `CreateOrderMessage`, sends one UDP packet, and calls `GetMarkets`. Real HTTP/WS surface is not yet implemented.
- **`crates/common`** — wire types (`OrderDTO`, `CreateOrder`, `Asset`, `Market`, `OrderId`, `UserId`) shared by gateway and engine. `pub use` flattens the `types::*` module at the crate root, so `common::types::Foo` and `common::Foo` both work — prefer `common::types::*` for clarity.
- **`crates/shared-protos`** — `tonic` build of [`proto/markets.proto`](crates/shared-protos/proto/markets.proto). Generated code is checked into [src/protos/markets.rs](crates/shared-protos/src/protos/markets.rs); the build script regenerates it on `cargo build`. Edit the `.proto` and rebuild — do not hand-edit the generated file.

### Two `Market` types (do not confuse)

There are intentionally two unrelated types named `Market`:

- [`common::types::Market`](crates/common/src/types/types.rs) — `Market(Asset, Asset)`, a zero-copy pair of asset enums currently used on the wire. Will be replaced by `market_index: u32`.
- [`markets::models::Market`](crates/markets/src/models/market.rs) — the full DB row (`id`, `market_index`, asset FKs, tick/lot/scale fields).

They live in different crates and serve different layers (wire vs. reference data). Don't try to unify them.

## Conventions in this codebase

- **Edition 2024** workspace-wide. `shared-protos` is on 2021 because the generated tonic code prefers it.
- **Async runtime is tokio** (`features = ["full"]`). `markets` and `gateway` are `#[tokio::main]`; `engine` is currently sync with a manual `ctrlc` handler and a blocking UDP recv loop.
- **Errors** use `thiserror`-derived enums in a per-crate `error.rs` (`ProgramError` / `ProgramResult`). New error variants go there.
- **Logging** is `tracing` with `tracing_subscriber::fmt`. Use `info!`/`warn!`/`error!` macros, not `println!`.
- **sqlx queries are compile-time checked.** Every `query_as!` call validates against the cached schema in `crates/markets/.sqlx/`. Forgetting to run `cargo sqlx prepare` after a query change will break CI builds.
- **Workspace dependency versions** live in the root [Cargo.toml](Cargo.toml) `[workspace.dependencies]`. Crates use `foo.workspace = true` — do not pin per-crate versions.

## Design docs

- [.docs/market-id-vs-market-index.md](.docs/market-id-vs-market-index.md) — why `market_index` (not DB `id`) is the cross-service identifier, with production-exchange context.
- [docs/architecture.md](docs/architecture.md), [docs/messaging.md](docs/messaging.md), [docs/trading-terms.md](docs/trading-terms.md), [docs/requirements.md](docs/requirements.md), [docs/guides.md](docs/guides.md) — broader design notes; consult before changing wire formats or adding new message types.
