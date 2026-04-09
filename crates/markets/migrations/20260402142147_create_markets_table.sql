-- Add migration script here
CREATE TABLE markets (
    id             SERIAL PRIMARY KEY,
    market_index   INT       NOT NULL UNIQUE,
    base_asset_id  INT       NOT NULL REFERENCES assets(id) ON DELETE RESTRICT,
    quote_asset_id INT       NOT NULL REFERENCES assets(id) ON DELETE RESTRICT,
    tick_size      BIGINT    NOT NULL,
    lot_size       BIGINT    NOT NULL,
    min_notional   BIGINT    NOT NULL,
    price_scale    INT       NOT NULL,
    qty_scale      INT       NOT NULL,
    is_active      BOOLEAN   NOT NULL DEFAULT true,
    UNIQUE(base_asset_id, quote_asset_id)
);