use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

use crate::types::{ClientId, MessageHeader, MessageType, OrderId, OrderSide, OrderType, TickerId};

/*

client_order_id
pub struct NewOrder {
    pub price: u128,           // offset 0
    pub quantity: u128,        // offset 16
    pub timestamp: u64,        // offset 32
    pub client_id: ClientId,        // offset 40
    pub ticker_id: u32,        // offset 44
    pub side: OrderSide,       // offset 48
    pub order_type: OrderType, // offset 49
    _padding: [u8; 14],        // offset 50 + padding(14 bytes) = 64 which is a multiple of 16
}

*/
/// NewOrderRequest represents an order in the system.
///
/// Memory layout and alignment notes:
/// - The largest alignment in this struct is `u128` (16 bytes). Therefore, the struct size must be a multiple of 16.
/// - Each field's offset is chosen so that it satisfies its alignment requirement.
/// - Automatic padding inserted by Rust is made explicit via `_padding` to ensure the struct is **padding-free** for `zerocopy::IntoBytes`.
/// - Total struct size = 80 bytes (multiple of largest alignment 16), safe for zero-copy serialization.
///
#[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes, Clone, Copy)]
#[repr(C)]
pub struct NewOrder {
    pub price: u128,    // offset 0
    pub quantity: u128, // offset 16
    pub timestamp: u64,
    pub client_order_id: OrderId, // offset 40 -> client generated
    pub client_id: ClientId,      // offset 48
    pub ticker_id: TickerId,      // offset 52
    pub side: OrderSide,          // offset 56
    pub order_type: OrderType,    // offset 57
    _padding: [u8; 6],            // offset 58 + padding(6 bytes) = 64 which is a multiple of 16
}

impl NewOrder {
    pub fn new(
        price: u128,
        quantity: u128,
        timestamp: u64,
        client_order_id: OrderId,
        client_id: ClientId,
        ticker_id: TickerId,
        side: OrderSide,
        order_type: OrderType,
    ) -> Self {
        Self {
            price,
            quantity,
            timestamp,
            client_order_id,
            client_id,
            ticker_id, //market index,
            side,
            order_type,
            _padding: [0u8; 6],
        }
    }
}

#[derive(Debug, IntoBytes, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct NewOrderMessage {
    pub header: MessageHeader, // offset 0  bytes
    _padding: [u8; 15], // offset 15 bytes + padding(  1 bytes) to make the struct 80 bytes which is a multiple of 16 which is the argest alignment in Order
    pub body: NewOrder, // offset 64 bytes
}

impl From<NewOrderMessage> for NewOrder {
    fn from(value: NewOrderMessage) -> Self {
        value.body
    }
}

impl NewOrderMessage {
    pub fn new(order: NewOrder) -> Self {
        Self {
            header: MessageHeader {
                message_type: MessageType::NewOrder,
            },
            _padding: [0u8; 15],
            body: order,
        }
    }
}
