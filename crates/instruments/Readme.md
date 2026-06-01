# markets

gRPC service that owns the catalog of trading markets (asset pairs, tick size, lot size, etc.). Persists to Postgres.

## Run

```
cargo run -p markets
```

Works from anywhere in the workspace. On startup it:

1. Loads `crates/markets/.env` (via `CARGO_MANIFEST_DIR` — no need to `export DATABASE_URL`)
2. Creates the `markets` database if missing (`db::ensure_database`)
3. Runs embedded migrations from `crates/markets/migrations/`
4. Seeds from `markets.toml`
5. Starts gRPC server on `127.0.0.1:50051`

## Config

`crates/markets/.env`:

```
DATABASE_URL=postgres://yusufkelo@localhost/markets
DB_MAX_CONNECTIONS=3
```

## Migrations

```
cd crates/markets

# add a new migration (creates timestamped .sql in migrations/)
sqlx migrate add <name>
sqlx migrate add -r <name>          # -r = reversible (up + down)

# apply
sqlx migrate run                    # or just `cargo run -p markets`
```

## sqlx offline mode

The `query_as!` macros check SQL against a live DB at compile time. To compile without one, we use the cached schema in `crates/markets/.sqlx/` (committed to git).

Whenever you change a query or migration:

```
cd crates/markets
cargo sqlx prepare
git add .sqlx/
```

Offline mode is forced on for all builds via [`.cargo/config.toml`](../../.cargo/config.toml).

## Postgres / psql

See [psql.md](psql.md) for connecting, listing tables, querying data, dropping the DB, etc.

## Reset to clean state

```
psql -d postgres -c "DROP DATABASE markets WITH (FORCE);"
cargo run -p markets   # recreates DB + tables + seed
```
