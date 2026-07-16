use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

use crate::types::{ClientId, OrderId, OrderSide, OrderType, TickerId};

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
#[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes, Clone, Copy)]
#[repr(C)]
pub struct AcceptedOrder {
    pub price: u128,              // offset 0
    pub quantity: u128,           // offset 16
    pub timestamp: u64,           // offset 32
    pub client_order_id: OrderId, // offset 40 - Assigned by OMS
    pub market_order_id: OrderId, // offset 48
    pub client_id: ClientId,      // offset 56
    pub ticker_id: TickerId,      // offset 60
    pub side: OrderSide,          // offset 64
    pub order_type: OrderType,    // offset 65
    _padding: [u8; 14],           // offset 66 + padding(14 bytes) = 18 which is a multiple of 16
}

impl AcceptedOrder {
    pub fn new(
        price: u128,
        quantity: u128,
        timestamp: u64,
        client_order_id: OrderId,
        market_order_id: OrderId,
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
            market_order_id,
            client_id,
            ticker_id,
            side,
            order_type,
            _padding: [0u8; 14],
        }
    }
}

#[derive(Debug, IntoBytes, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct CreateOrderMessage {
    pub header: MessageHeader, // offset 0 bytes
    _padding: [u8; 15],        // offset 1
    pub body: AcceptedOrder, // offset header(1 bytes) + padding(15 bytes)  + body(64 bytes) to make the struct 80 bytes which is a multiple of 16 which is the argest alignment in Order
}

impl From<CreateOrderMessage> for AcceptedOrder {
    fn from(value: CreateOrderMessage) -> Self {
        value.body
    }
}

impl CreateOrderMessage {
    pub fn new(order: AcceptedOrder) -> Self {
        Self {
            header: MessageHeader {
                message_type: MessageType::CreateOrder,
            },
            _padding: [0u8; 15],
            body: order,
        }
    }
}
