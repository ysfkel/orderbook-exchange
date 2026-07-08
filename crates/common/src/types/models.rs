use serde::{Deserialize, Serialize};
use zerocopy::{FromZeros, Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

use crate::types::dto::OrderDTO;

#[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, FromZeros)]
#[repr(i8)]
pub enum OrderSide {
    Unset = 0,
    Buy = 1,
    Sell = -1,
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
