use crate::{dto::MarketWithAssets, error::ProgramResult, models::Market};
use async_trait::async_trait;

#[async_trait]
pub trait Repository<T: Send + Sync> {
    async fn find_all(&self) -> ProgramResult<Vec<T>>;
    async fn find_by_id(&self, id: i32) -> ProgramResult<Option<T>>;
}

#[async_trait]
pub trait MarketRepository {
    async fn find_all_with_assets(&self) -> ProgramResult<Vec<MarketWithAssets>>;
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
    ) -> ProgramResult<Market>;
}
