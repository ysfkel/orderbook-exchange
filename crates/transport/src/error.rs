use rusteron_client::{AeronCError, AeronError, AeronErrorType};
use thiserror::Error;

use rusteron_client::bindings::{
    AERON_PUBLICATION_ADMIN_ACTION, AERON_PUBLICATION_BACK_PRESSURED, AERON_PUBLICATION_CLOSED,
    AERON_PUBLICATION_MAX_POSITION_EXCEEDED, AERON_PUBLICATION_NOT_CONNECTED,
};

const NOT_CONNECTED: i64 = AERON_PUBLICATION_NOT_CONNECTED as i64;
const BACK_PRESSURED: i64 = AERON_PUBLICATION_BACK_PRESSURED as i64;
const ADMIN_ACTION: i64 = AERON_PUBLICATION_ADMIN_ACTION as i64;
const CLOSED: i64 = AERON_PUBLICATION_CLOSED as i64;
const MAX_POSITION_EXCEEDED: i64 = AERON_PUBLICATION_MAX_POSITION_EXCEEDED as i64;

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("transport setup failed: (code = {0})")]
    Failed(i32),
}

impl From<AeronCError> for SetupError {
    fn from(e: AeronCError) -> Self {
        tracing::error!(
            error.code = e.code,
            error.kind = ?e.kind(),
            "aeron transport setup failed"
        );
        Self::Failed(e.code)
    }
}

#[derive(Debug, Error, PartialEq, Eq, Clone, Copy)]
pub enum PublishError {
    /// No subscriber attached yet. Retry — they may attach later.
    #[error("no subscriber connected")]
    NotConnected,

    /// Subscriber too slow; the ring buffer is full. Yield and retry.
    #[error("back-pressured")]
    BackPressured,

    /// Transient driver bookkeeping. Yield and retry.
    #[error("admin action in progress")]
    AdminAction,

    /// Publication has been closed. Terminal.
    #[error("publication closed")]
    Closed,

    /// Stream position would overflow. Terminal.
    #[error("max stream position exceeded")]
    MaxPositionExceeded,

    /// Negative return code we don't recognise.
    #[error("unknown offer error code: {0}")]
    Unknown(i64),
}

impl PublishError {
    /// True for errors where calling `offer` again may succeed.
    #[inline]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NotConnected | Self::BackPressured | Self::AdminAction
        )
    }
}

impl From<i64> for PublishError {
    fn from(code: i64) -> Self {
        match code {
            NOT_CONNECTED => PublishError::NotConnected,
            BACK_PRESSURED => PublishError::BackPressured,
            ADMIN_ACTION => PublishError::AdminAction,
            CLOSED => PublishError::Closed,
            MAX_POSITION_EXCEEDED => PublishError::MaxPositionExceeded,
            code => PublishError::Unknown(code),
        }
    }
}

#[derive(Debug, Error)]
pub enum PollError {
    #[error("subscription closed")]
    Closed,

    // aeronmd was stopped or crashed. Caller should reconnect.
    #[error("media driver shutdown (code = {0})")]
    DriverTimeout(i32),

    #[error("transport error: (code = {0})")]
    Other(i32),
}

impl From<AeronCError> for PollError {
    fn from(e: AeronCError) -> Self {
        tracing::error!(
            error.code = e.code,
            error.kind = ?e.kind(),
            "transport error"
        );

        match e.kind() {
            AeronErrorType::PublicationClosed => PollError::Closed,
            AeronErrorType::ClientErrorDriverTimeout => PollError::DriverTimeout(e.code),
            _ => PollError::Other(e.code),
        }
    }
}
