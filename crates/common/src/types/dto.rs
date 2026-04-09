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
/// - Each field’s offset is chosen so that it satisfies its alignment requirement.
/// - Automatic padding inserted by Rust is made explicit via `_padding` to ensure the struct is **padding-free** for `zerocopy::IntoBytes`.
/// - Total struct size = 128 bytes (multiple of largest alignment 16), safe for zero-copy serialization.
///
/// Offsets and sizes:
/// ```text
/// price           u128           offset 0    size 16  alignment 16
/// quantity        u128           offset 16   size 16  alignment 16
/// filled_quantity u128           offset 32   size 16  alignment 16
/// price_scale     u64            offset 48   size 8   alignment 8
/// quantity_scale  u64            offset 56   size 8   alignment 8
/// timestamp       u64            offset 64   size 8   alignment 8
/// order_id        OrderId([u8;16]) offset 72 size 16 alignment 1
/// user_id         UserId([u8;32])  offset 88 size 32 alignment 1
/// order_type      OrderType(u8)    offset 120 size 1  alignment 1
/// order_status    OrderStatus(u8)  offset 121 size 1  alignment 1
/// side            OrderSide(u8)    offset 122 size 1  alignment 1
/// _padding        [u8;5]           offset 123 size 5  alignment 1  (fills to make struct size 128)
///
/// Since fields are arranged next to fields of same size.Each field’s offset is a multiple of its own alignment, so no inter-field padding is needed.
/// Only trailing padding is needed when: All field offsets satisfy their own alignment
/// You want the struct size to be multiple of the largest alignment (for IntoBytes / zero-copy)
/// If you had smaller fields between larger ones, you might need inter-field padding too.
/// ```
#[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct OrderDTO {
    pub price: u128,               // offset 0
    pub quantity: u128,            // offset 16
    pub filled_quantity: u128,     // offset 32
    pub price_scale: u64,          // offset 48
    pub quantity_scale: u64,       // offset 56
    pub timestamp: u64,            // offset 64
    pub order_id: OrderId,         // offset 72
    pub user_id: UserId,           // offset 88
    pub order_type: OrderType,     // offset 120
    pub order_status: OrderStatus, // offset 121
    pub side: OrderSide,           // offset 122
    _padding: [u8; 5],             // offset 123, explicit padding to make struct size 128 bytes
}

impl OrderDTO {
    pub fn new(
        price: u128,
        quantity: u128,
        filled_quantity: u128,
        price_scale: u64,
        quantity_scale: u64,
        timestamp: u64,
        order_id: OrderId,
        user_id: UserId,
        order_type: OrderType,
        order_status: OrderStatus,
        side: OrderSide,
    ) -> Self {
        Self {
            price,
            quantity,
            filled_quantity,
            price_scale,
            quantity_scale,
            timestamp,
            order_id,
            user_id,
            order_type,
            order_status,
            side,
            _padding: [0u8; 5], // zero out padding for safety
        }
    }
}
