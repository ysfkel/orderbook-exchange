use super::Repository;
use crate::{dto::MarketWithAssets, error::ProgramResult, models::Market, repositories::MarketRepository};
use sqlx::PgPool;
use async_trait::async_trait;

#[derive(Debug)]
pub struct MarketRepo {
    pub pool: PgPool,
}

impl MarketRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Market> for MarketRepo {
    async fn find_all(&self) -> ProgramResult<Vec<Market>> {
        let markets = sqlx::query_as!(
            Market,
            "SELECT id, base_asset_id, quote_asset_id, market_index,tick_size,lot_size ,min_notional,price_scale,qty_scale,is_active FROM markets"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(markets)
    }

    async fn find_by_id(&self, id: i32) -> ProgramResult<Option<Market>> {
        let market = sqlx::query_as!(
            Market,
            "SELECT id, base_asset_id, quote_asset_id, market_index,tick_size,lot_size ,min_notional,price_scale,qty_scale,is_active FROM markets WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(market)
    }
}

#[async_trait]
impl MarketRepository for MarketRepo {
    async fn get_or_create(
    &self,
    base_asset_id: i32,
    quote_asset_id: i32,
    market_index: i32,
    tick_size_scaled: i64,
    lot_size_scaled: i64,
    min_notional_scaled: i64,
    price_scale: i32,
    qty_scale: i32,
    is_active: bool,
) -> ProgramResult<Market> {
    let market = sqlx::query_as!(
        Market,
        "INSERT INTO markets (
            base_asset_id, quote_asset_id, market_index,
            tick_size, lot_size, min_notional,
            price_scale, qty_scale, is_active
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (base_asset_id, quote_asset_id)
        DO UPDATE SET base_asset_id = EXCLUDED.base_asset_id
        RETURNING
            id, base_asset_id, quote_asset_id, market_index,
            tick_size, lot_size, min_notional,
            price_scale, qty_scale, is_active",
        base_asset_id,
        quote_asset_id,
        market_index,
        tick_size_scaled,
        lot_size_scaled,
        min_notional_scaled,
        price_scale,
        qty_scale,
        is_active,
    )
    .fetch_one(&self.pool)
    .await?;
    Ok(market)
}
    async fn find_all_with_assets(&self) -> ProgramResult<Vec<MarketWithAssets>> {
        let markets = sqlx::query_as!(
            MarketWithAssets,
            r#"
            SELECT 
               m.id,
               m.market_index,
               m.is_active,
               m.tick_size,
               m.lot_size,
               m.min_notional,
               m.price_scale,
               m.qty_scale,
               base.name  AS base,
               quote.name AS quote
            FROM markets m
            JOIN assets base  ON base.id  = m.base_asset_id
            JOIN assets quote ON quote.id = m.quote_asset_id 
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(markets)
    }
}

