use std::marker;

use crate::types::Market;
use crate::types::dto::OrderDTO;
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

#[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct CreateOrder {
    // The struct alignment = 16 (because of OrderDTO)
    // So total size must be a multiple of 16:
    pub order: OrderDTO, // alignment 16 bytes
    //pub market: Market,  // use market  (u8; u8)
   // _padding: [u8; 14],
}

impl CreateOrder {
    pub fn new(order: OrderDTO) -> Self {
        //    let mut market = [0u8; 16];
        // let bytes = symbol.as_bytes();
        // let len = bytes.len().min(market.len()); // make sure we don’t overflow
        // market[..len].copy_from_slice(&bytes[..len]);
        Self {
            order,
        //    market,
          //  _padding: [0u8; 14],
        }
    }
}

#[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, FromZeros)]
#[repr(u8)]
pub enum MessageType {
    CreateOrder = 0,
    CancelOrder = 1,
}

#[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, FromZeros)]
#[repr(C)]
pub struct MessageHeader {
    pub message_type: MessageType,
}

#[derive(Debug, IntoBytes, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct CreateOrderMessage {
    pub header: MessageHeader, // 1
    _padding: [u8; 15],        // padding to make the struct 32 bytes
    pub body: CreateOrder,     // 16
}

pub fn create_order(dto: OrderDTO) -> CreateOrderMessage {
    CreateOrderMessage {
        header: MessageHeader {
            message_type: MessageType::CreateOrder,
        },
        _padding: [0u8; 15], // zero out padding
        body: CreateOrder::new(dto),
    }
}
