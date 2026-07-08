use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

use crate::types::{OrderSide, OrderType};

use super::{MessageHeader, MessageType};

/// AcceptedOrder represents an order in the system.
///
/// OMS → Engine (AcceptedOrder) — after OMS validates and assigns an order ID:
///
/// Memory layout and alignment notes:
/// - The largest alignment in this struct is `u128` (16 bytes). Therefore, the struct size must be a multiple of 16.
/// - Each field's offset is chosen so that it satisfies its alignment requirement.
/// - Automatic padding inserted by Rust is made explicit via `_padding` to ensure the struct is **padding-free** for `zerocopy::IntoBytes`.
/// - Total struct size = 80 bytes (multiple of largest alignment 16), safe for zero-copy serialization.
///
#[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct Order {
    pub price: u128,           // offset 0
    pub quantity: u128,        // offset 16
    pub timestamp: u64,        // offset 32
    pub user_id: u32,          // offset 40
    pub order_id: u32,         // offset 44 - Assigned by OMS
    pub market_index: u32,     // offset 48
    pub side: OrderSide,       // offset 52
    pub order_type: OrderType, // offset 53
    _padding: [u8; 10],        // offset 54 + padding(10 bytes) = 64 which is a multiple of 16
}

impl Order {
    pub fn new(
        price: u128,
        quantity: u128,
        timestamp: u64,
        user_id: u32,
        order_id: u32,
        market_index: u32,
        side: OrderSide,
        order_type: OrderType,
    ) -> Self {
        Self {
            price,
            quantity,
            timestamp,
            user_id,
            order_id,
            market_index,
            side,
            order_type,
            _padding: [0u8; 10],
        }
    }
}

#[derive(Debug, IntoBytes, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct CreateOrderMessage {
    pub header: MessageHeader, // offset 0 bytes
    _padding: [u8; 15],        // offset 1
    pub body: Order, // offset header(1 bytes) + padding(15 bytes)  + body(64 bytes) to make the struct 80 bytes which is a multiple of 16 which is the argest alignment in Order
}

impl From<CreateOrderMessage> for Order {
    fn from(value: CreateOrderMessage) -> Self {
        value.body
    }
}

impl CreateOrderMessage {
    pub fn new(order: Order) -> Self {
        Self {
            header: MessageHeader {
                message_type: MessageType::CreateOrder,
            },
            _padding: [0u8; 15],
            body: order,
        }
    }
}
