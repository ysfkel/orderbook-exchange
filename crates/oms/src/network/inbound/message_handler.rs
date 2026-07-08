use crate::error::ProgramError;
use common::{
    queue::{QueueProducer, RingBufferProducer},
    types::{error::MessageError, MessageHeader, MessageType, NewOrder, NewOrderMessage},
};
use tracing::{error, info};
use zerocopy::TryFromBytes;

pub struct MessageHandler {}

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
///
/// // producer: RingBufferProducer<NewOrder>
pub fn handle_message(
    msg: &[u8],
    max_msg_size: usize,
    producer: &mut RingBufferProducer<NewOrder>,
) -> Result<(), ProgramError> {
    let header = MessageHeader::parse(msg, max_msg_size)?;
    // ── Deserialization ───────────────────────────────────────────────────
    // ── Dispatch ──────────────────────────────────────────────────────────
    // Match on the message variant. Rust's exhaustive matching means if you
    // add a new variant to the Message enum in message.rs, the compiler will
    // produce an error here until you handle it — you can't accidentally ignore
    // a new message type.
    match header.message_type {
        MessageType::NewOrder => {
            // `try_read_from_bytes` (rather than `try_ref_from_bytes`) copies
            // the bytes into an owned, properly-aligned `NewOrderMessage`
            // instead of handing back a reference into `msg`. This matters
            // because `NewOrder` contains `u128` fields (16-byte alignment),
            // and `msg` comes straight off a UDP recv path with no alignment
            // guarantee — `try_ref_from_bytes` could spuriously fail on a
            // perfectly valid message just because of where the OS happened
            // to place it in memory. Reading into an owned value sidesteps
            // that: it can only fail on validity/size, not alignment. It
            // also means `new_order.body` is already a local, movable value
            // below, so `NewOrder` doesn't need to derive `Copy` just to
            // make this compile.
            match NewOrderMessage::try_read_from_bytes(msg) {
                Ok(new_order) => {
                    info!(
                        "New order recieved  price={}, quantity={}",
                        new_order.body.price, new_order.body.quantity,
                    );

                    producer.push(new_order.body).map_err(|e| {
                        error!(error = %e, "failed to push new order to queue");
                         ProgramError::QueuePushError(e)
                    })?;
                    //todo! send to new order service
                }
                Err(e) => {
                    error!(error = %e, "new order - some error occured ");
                }
            }
        }
        _ => (),
    }

    Ok(())
}