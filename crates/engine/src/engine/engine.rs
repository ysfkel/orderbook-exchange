use std::{collections::HashMap, hash::Hash};

use common::types::Asset;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// use crate::{engine::orderbook::OrderBook, types::engine::Asset};
// use crate::{engine::orderbook::OrderBook, types::engine::Asset};

// pub enum AmountType {
//     AVAILABLE,
//     LOCKED,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Amount {
//     available: Decimal,
//     locked: Decimal,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct UserBalance {
//     user_id: u64,
//     balance: HashMap<Asset, Amount>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Engine {
//     orderbook: HashMap<(Asset, Asset), OrderBook>,
// }
