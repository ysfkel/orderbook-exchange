

use serde::Deserialize;

#[derive(Deserialize)]
pub struct MarketEntry {
    pub market_index: i32,
    pub base: String,
    pub quote: String,
    pub tick_size: f64,
    pub lot_size: f64,
    pub min_notional: f64,
}

impl MarketEntry {
    pub fn price_scale(&self) -> i32 {
        decimal_places(self.tick_size)
    }

    pub fn qty_scale(&self) -> i32 {
        decimal_places(self.lot_size)
    }

    pub fn tick_size_scaled(&self) -> i64 {
        scale(self.tick_size, self.price_scale())
    }

    pub fn lot_size_scaled(&self) -> i64 {
        scale(self.lot_size, self.qty_scale())
    }

    pub fn min_notional_scaled(&self) -> i64 {
        scale(self.min_notional, self.price_scale())
    }
}

fn decimal_places(value: f64) -> i32 {
    let s = format!("{}", value);
    match s.find('.') {
        Some(dot_pos) => (s.len() - dot_pos - 1) as i32,
        None => 0,
    }
}

fn scale(value: f64, scale: i32) -> i64 {
    (value * 10f64.powi(scale)).round() as i64
}