use std::fmt::Display;

use crate::error::{PollError, PublishError};

/// Appends byte messages to a transport stream.
///
/// Built once via the implementing type's constructor (which binds the
/// channel and stream id). `offer` is then called repeatedly on the hot path.
///
/// `offer` is non-blocking — returns immediately whether the message was
/// accepted or not. Callers must handle retryable errors (see
/// [`OfferError::is_retryable`]).

pub trait Publisher: Send {
    type Data: Display;
    type Error: std::error::Error + Send + Sync + 'static;
    /// Attempt to publish `bytes`. On success returns the new stream position.
    fn publish(&self, bytes: &[u8]) -> Result<Self::Data, Self::Error>;
}

/// Drains incoming byte messages from a transport stream.
///
/// The receive callback is bound at construction (not here), so `poll` only
/// drives the pump. Each call delivers up to `fragment_limit` messages.
///
/// `poll` is non-blocking — returns immediately if nothing is available.
/// `Ok(0)` is normal and means "no new messages."
pub trait Subscriber: Send {
    type Data: Display;
    type Error: std::error::Error + Send + Sync + 'static;
    fn poll(&mut self, fragment_limit: usize) -> Result<Self::Data, Self::Error>;
    fn is_connected(&self) -> bool;
}
