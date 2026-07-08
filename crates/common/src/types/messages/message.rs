use crate::types::error::MessageError;
use tracing::{Level, error, event, info};
use zerocopy::TryFromBytes;
use zerocopy::{FromZeros, Immutable, IntoBytes, KnownLayout, Unaligned};

#[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, TryFromBytes)]
#[repr(u8)]
pub enum MessageType {
    CreateOrder = 0,
    CancelOrder = 1,
    NewOrder = 2,
}

#[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct MessageHeader {
    pub message_type: MessageType,
}

impl MessageHeader {
    pub fn parse<'a>(msg: &'a [u8], max_msg_size: usize) -> Result<&'a Self, MessageError> {
        // Reject oversized packets before spending any CPU on them.
        // A buggy or malicious sender could craft a giant packet to make us
        // waste time deserializing garbage — drop it immediately instead.
        let len = msg.len();
        if len > max_msg_size {
            return Err(MessageError::MessageTooLarge {
                size: len,
                max: max_msg_size,
            });
        }
        let header_size = size_of::<Self>();
        let header = Self::try_ref_from_bytes(&msg[..header_size])
            .map_err(|e| MessageError::DeserializeMessageHeader(e.into()))?;

        Ok(header)
    }
}
