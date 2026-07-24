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

    #[error("outbound queue disconnected")]
    OutboundQueueDisconnected,
}

impl From<PublishError> for ProgramError {
    fn from(value: PublishError) -> Self {
        Self::Publish(value)
    }
}
