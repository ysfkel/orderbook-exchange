// ═══════════════════════════════════════════════════════════════════════════
// src/message.rs — Message Handling
//
// This file contains the business logic for processing one received UDP packet.
// It is intentionally kept separate from the receive loop (listener.rs) so that:
//
//   1. It is unit-testable without a real socket — you just call handle_message()
//      directly with a byte slice and a fake source address.
//   2. The receive loop stays focused on "get bytes from network" and this file
//      stays focused on "what do we do with those bytes".
//   3. When new message types are added to message.rs, the compiler forces you
//      to handle them here (Rust's exhaustive match).
// ═══════════════════════════════════════════════════════════════════════════
use std::net::SocketAddr;

use crate::{error::ProgramError, types::engine::Order};
use common::types::{CreateOrderMessage, MessageHeader};
use zerocopy::TryFromBytes;
// use common::types::Message;
use rust_decimal::prelude::ToPrimitive;
use tracing::{error, info};

/// Validates and deserializes one UDP payload, then acts on its contents.
///
/// Arguments:
///   `bytes`        — the raw bytes of a single UDP packet
///   `src`          — the sender's IP:port address (for log context only)
///   `max_msg_size` — if the packet exceeds this byte count, we reject it
///
/// Returns Ok(()) if the message was handled successfully.
/// Returns Err(...) if validation or deserialization failed — the caller
/// decides whether to drop and continue or to escalate.
pub fn handle_message(bytes: &[u8], max_msg_size: usize) -> Result<(), ProgramError> {
    // ── Size guard ────────────────────────────────────────────────────────
    // Reject oversized packets before spending any CPU on them.
    // A buggy or malicious sender could craft a giant packet to make us
    // waste time deserializing garbage — drop it immediately instead.
    if bytes.len() > max_msg_size {
        return Err(ProgramError::MessageTooLarge {
            size: bytes.len(),
            max: max_msg_size,
        });
    }

    let header_size = size_of::<MessageHeader>();

    let header = match MessageHeader::try_ref_from_bytes(&bytes[..header_size]) {
        Ok(h) => h,
        Err(e) => {
            error!("Header alignment/size error from {:?}", e);
            return Err(ProgramError::DeserializeMessageHeader(e.into()));
        }
    };

    // ── Deserialization ───────────────────────────────────────────────────
    // ── Dispatch ──────────────────────────────────────────────────────────
    // Match on the message variant. Rust's exhaustive matching means if you
    // add a new variant to the Message enum in message.rs, the compiler will
    // produce an error here until you handle it — you can't accidentally ignore
    // a new message type.
    match header.message_type {
        common::types::MessageType::CreateOrder => {
            info!("Received Create order");
            match CreateOrderMessage::try_ref_from_bytes(&bytes) {
                Ok(msg) => {
                    let order_dto = &msg.body.order;
                    info!(
                        "Parsed CreateOrderMessage: price={}, quantity={}",
                        order_dto.price,
                        order_dto.quantity,
                       // &msg.body.market.to_string()
                    );
                }
                Err(e) => {
                    error!("Failed to parse CreateOrderMessage from  {}", e);
                    return Err(ProgramError::DeserializeMessageBody(e.into()));
                }
            };
        }
        _ => {
            info!("not implemented yet");
        }
    }

    Ok(())
}
