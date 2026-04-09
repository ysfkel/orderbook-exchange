use serde::{Deserialize, Serialize};
use zerocopy::{FromZeros, Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

use crate::types::dto::OrderDTO;

#[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, FromZeros)]
#[repr(u8)]
pub enum OrderSide {
    Buy = 0,
    Sell = 1,
}

#[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, FromZeros)]
#[repr(u8)]
pub enum OrderType {
    LIMIT = 0,
    MARKET = 1,
}

#[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, FromZeros)]
#[repr(u8)]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
}
