use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
// pub struct Market {
//     pub id: i32,
//     pub base_asset_id: i32,
//     pub quote_asset_id: i32,
//     pub market_index: i32,
// }

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Market {
    pub id: i32,
    /// ID of the base asset (the asset being bought/sold)
    /// Example: ETH in ETH/USDT
    pub base_asset_id: i32,

    /// ID of the quote asset (the asset used to price the base asset)
    /// Example: USDT in ETH/USDT
    pub quote_asset_id: i32,

    pub market_index: i32,

    /// Minimum price increment, expressed in QUOTE asset units (scaled integer)
    /// Example: tick_size = 1 with price_scale = 2 → 0.01 USDT
    /// Rule: price must be a multiple of tick_size
    pub tick_size: i64,

    /// Minimum quantity increment, expressed in BASE asset units (scaled integer)
    /// Example: lot_size = 1 with qty_scale = 4 → 0.0001 ETH
    /// Rule: quantity must be a multiple of lot_size
    pub lot_size: i64,

    /// Minimum order value (price * quantity), expressed in QUOTE asset units (scaled integer)
    /// Prevents very small trades (dust orders)
    /// Example: min_notional = 10 USDT (scaled to integer)
    pub min_notional: i64,

    /// Number of decimal places used to scale prices (QUOTE asset precision)
    /// Example: price_scale = 2 → price = 2000.01 stored as 200001
    /// Derived from tick_size
    pub price_scale: i32,

    /// Number of decimal places used to scale quantities (BASE asset precision)
    /// Example: qty_scale = 4 → quantity = 0.005 stored as 50
    /// Derived from lot_size
    pub qty_scale: i32,

    pub is_active: bool
}