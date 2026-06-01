use crate::types::OrderId;
use crate::types::{
    UserId,
    models::{OrderSide, OrderStatus, OrderType},
};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

/// OrderDTO represents an order in the system.
///
/// Memory layout and alignment notes:
/// - The largest alignment in this struct is `u128` (16 bytes). Therefore, the struct size must be a multiple of 16.
/// - Each field's offset is chosen so that it satisfies its alignment requirement.
/// - Automatic padding inserted by Rust is made explicit via `_padding` to ensure the struct is **padding-free** for `zerocopy::IntoBytes`.
/// - Total struct size = 80 bytes (multiple of largest alignment 16), safe for zero-copy serialization.
///
/// Offsets and sizes:
/// ```text
/// price           u128             offset 0    size 16  alignment 16
/// quantity        u128             offset 16   size 16  alignment 16
/// filled_quantity u128             offset 32   size 16  alignment 16
/// timestamp       u64              offset 48   size 8   alignment 8
/// market_id       u32              offset 56   size 4   alignment 4
/// order_id        u32              offset 60   size 4   alignment 4
/// user_id         u32              offset 64   size 4   alignment 4
/// order_type      OrderType(u8)    offset 68   size 1   alignment 1
/// order_status    OrderStatus(u8)  offset 69   size 1   alignment 1
/// side            OrderSide(u8)    offset 70   size 1   alignment 1
/// _padding        [u8;9]           offset 71   size 9   alignment 1  (fills to make struct size 80)
///
/// Since fields are arranged next to fields of same size, each field's offset is a multiple of its own alignment, so no inter-field padding is needed.
/// Only trailing padding is needed when: all field offsets satisfy their own alignment and
/// you want the struct size to be a multiple of the largest alignment (for IntoBytes / zero-copy).
/// ```
/// ```

#[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct OrderDTO {
    pub price: u128,               // offset 0
    pub quantity: u128,            // offset 16
    pub filled_quantity: u128,     // offset 32
    pub timestamp: u64,            // offset 48
    pub market_id: u32,            // offset 56
    pub order_id: u32,             // offset 60 
    pub user_id: u32,              // offset 64
    pub order_type: OrderType,     // offset 68
    pub order_status: OrderStatus, // offset 69
    pub side: OrderSide,           // offset 70 - 
    _padding: [u8; 9],             // offset 71, explicit padding to make struct size 71 + 9=80 bytes. which makes the struct size a multiple of the largest alignment u128 (16 bytes ) for zero-copy safety. 
}
 
impl OrderDTO {
    pub fn new(
        price: u128,
        quantity: u128,
        filled_quantity: u128,
        timestamp: u64,
        market_id: u32, 
        order_id: u32,
        user_id: u32,
        order_type: OrderType,
        order_status: OrderStatus,
        side: OrderSide,
    ) -> Self {
        Self {
            price,
            quantity,
            filled_quantity,
            timestamp,
            market_id,
            order_id,
            user_id,
            order_type,
            order_status,
            side,
            _padding: [0u8; 9], // zero out padding for safety
        }
    }
}
