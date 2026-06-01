# psql cheat sheet

Quick reference for inspecting the `markets` Postgres database.

## Connect

```
psql -d markets               # interactive session in the markets DB
psql -d postgres              # connect to the default `postgres` DB (use this when markets doesn't exist)
\q                            # quit psql
```

Once connected, the prompt becomes `markets=#`.

## Run a single command without entering psql

```
psql -d markets -c "SELECT * FROM assets;"
psql -d markets -x -c "SELECT * FROM markets;"      # -x = expanded vertical layout (one row at a time)
```

## List things (meta-commands, run inside psql)

```
\l            -- list all databases on the server
\dt           -- list tables in current DB
\dt+          -- list tables with sizes
\d            -- list relations (tables, views, sequences)
\d assets     -- describe assets table (columns, types, indexes, FKs)
\d markets    -- describe markets table
\du           -- list users / roles
\df           -- list functions
\di           -- list indexes
\dn           -- list schemas
```

Same from the shell without entering psql:

```
psql -d postgres -c '\l'
psql -d markets  -c '\dt'
psql -d markets  -c '\d markets'
```

## Query data

```sql
SELECT * FROM assets;
SELECT * FROM markets;

-- Pick columns
SELECT id, name FROM assets;

-- Filter
SELECT * FROM markets WHERE is_active = true;

-- Sort + limit
SELECT * FROM markets ORDER BY market_index LIMIT 10;

-- Count rows
SELECT COUNT(*) FROM assets;
SELECT COUNT(*) FROM markets;

-- Join markets to asset names
SELECT m.id, m.market_index,
       b.name AS base, q.name AS quote,
       m.tick_size, m.lot_size, m.is_active
FROM markets m
JOIN assets b ON b.id = m.base_asset_id
JOIN assets q ON q.id = m.quote_asset_id
ORDER BY m.market_index;
```

## Modify data

```sql
INSERT INTO assets (name) VALUES ('SOL');
UPDATE markets SET is_active = false WHERE id = 1;
DELETE FROM assets WHERE id = 99;
```

## Migrations table

sqlx tracks applied migrations in `_sqlx_migrations`:

```sql
SELECT version, description, success FROM _sqlx_migrations ORDER BY version;
```

If a row exists here that no longer has a matching file on disk, sqlx errors with `migration X was previously applied but is missing in the resolved migrations`. Fix it by either restoring the file or deleting the row:

```sql
DELETE FROM _sqlx_migrations WHERE version = 20260403113959;
```

## Create / drop the database

```
# Create (rarely needed — `cargo run -p markets` auto-creates it)
psql -d postgres -c "CREATE DATABASE markets;"

# Drop (FORCE terminates any open connections first)
psql -d postgres -c "DROP DATABASE markets WITH (FORCE);"

# Confirm it's gone
psql -d postgres -lqt | cut -d \| -f 1 | grep -qw markets && echo "still exists" || echo "gone"
```

If the running markets server is holding the connection, kill it first:

```
lsof -i :50051 -t | xargs kill
```

## Switch DB without leaving psql

```
markets=# \c postgres        -- switch to `postgres` DB
postgres=# \c markets        -- switch back
```

## Useful display toggles (inside psql)

```
\x            -- toggle expanded display (vertical layout — great for wide rows)
\timing       -- show duration after every query
\pset null '∅'-- show NULL as a visible symbol instead of blank
\! ls         -- run a shell command without leaving psql
\h SELECT     -- SQL help for a keyword
\?            -- list every psql meta-command
```

## One-liners I keep reaching for

```
# Drop, recreate, seed from scratch
psql -d postgres -c "DROP DATABASE IF EXISTS markets WITH (FORCE);" && cargo run -p markets

# Quick row counts
psql -d markets -c "SELECT 'assets' AS t, COUNT(*) FROM assets UNION ALL SELECT 'markets', COUNT(*) FROM markets;"

# Dump schema only (no data) to a file
pg_dump -d markets --schema-only > schema.sql

# Dump data only
pg_dump -d markets --data-only > data.sql
```
