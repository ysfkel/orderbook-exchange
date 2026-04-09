


#[derive(Debug)]
pub struct MarketWithAssets {
    pub id: i32,
    pub base: String,
    pub quote: String,
    pub market_index: i32,
    pub is_active: bool,
    pub tick_size: i64, 
    pub lot_size: i64,
    pub min_notional: i64, 
    pub price_scale: i32, 
    pub qty_scale: i32,
}