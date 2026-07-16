use common::queue::QueueSendError;
use zerocopy::ConvertError;

use crate::types::EngineRequest;

#[derive(Debug, thiserror::Error)]
pub enum ProgramError {
    #[error("Invalid interface address")]
    InvalidInterface(),

    #[error("Invalid multicast group")]
    InvalidMulticastGroup(),

    #[error("Socket error: {0}")]
    Socket(#[from] std::io::Error),

    #[error("Message too large: {size} bytes (max {max})")]
    MessageTooLarge { size: usize, max: usize },

    #[error("Failed to Deserialize Message Header: {0}")]
    DeserializeMessageHeader(DeserializeErrorKind),

    #[error("Failed to Deserialize Message Body: {0}")]
    DeserializeMessageBody(DeserializeErrorKind),

    #[error("Failed to subscribe to transport: {0}")]
    TransportSubsriptionError(#[from] transport::PollError),

    #[error("transport disconnected")]
    TransportDisconnected,

    #[error("Queue push error: {0}")]
    QueuePushError(#[from] QueueSendError<EngineRequest>),
}

#[derive(Debug, thiserror::Error)]
pub enum DeserializeErrorKind {
    #[error("Alignment error")]
    Alignment,
    #[error("Size error")]
    Size,
    #[error("Validity error")]
    Validity,
}

impl<A, S, V> From<zerocopy::ConvertError<A, S, V>> for DeserializeErrorKind {
    fn from(err: zerocopy::ConvertError<A, S, V>) -> Self {
        match err {
            zerocopy::ConvertError::Alignment(_) => Self::Alignment,
            zerocopy::ConvertError::Validity(_) => Self::Validity,
            zerocopy::ConvertError::Size(_) => Self::Size,
        }
    }
}
