
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crate::engine::orderbook::OrderBook;

pub struct MatchingEngine {
    /// One order book per instrument (the book's OrderBookHashMap, an
    /// std::array<MEOrderBook*, ME_MAX_TICKERS>).
    ticker_order_book: Vec<OrderBook>,
}
