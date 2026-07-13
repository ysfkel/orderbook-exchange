use common::{queue::QueueSendError, types::error::MessageError};
use thiserror::Error;

use transport::{PublishError, SetupError};

#[derive(Debug, Error)]
pub enum ProgramError {
    #[error("transport setup failed: {0}")]
    Setup(#[from] SetupError),

    #[error("publish failed: {0}")]
    Publish(PublishError),

    #[error("transport subscription failed: {0}")]
    TransportSubscriptionError(#[from] transport::PollError),

    #[error("instruments service connection failed: {0}")]
    InstrumentsConnection(#[from] tonic::transport::Error),

    #[error("instruments service call failed: {0}")]
    InstrumentsCall(#[from] tonic::Status),

    #[error("message error: {0}")]
    MessageError(#[from] MessageError),

    #[error("queue push failed")]
    QueuePushError(#[from] QueueSendError<common::types::new_order::NewOrder>),

    #[error("outbound queue disconnected")]
    OutboundQueueDisconnected,
    #[error("transport disconnected")]
    TransportDisconnected,
}

impl From<PublishError> for ProgramError {
    fn from(value: PublishError) -> Self {
        Self::Publish(value)
    }
}
