use crate::Subscriber;
use std::sync::atomic::{AtomicBool, Ordering};
pub struct Poller<T: Subscriber> {
    transport: T,
    fragment_limits: usize,
}

impl<T: Subscriber> Poller<T> {
    pub fn new(transport: T, fragment_limits: usize) -> Self {
        Self {
            transport,
            fragment_limits,
        }
    }

    /// Poll until `run` goes false (clean stop → `Ok`), the transport
    /// disconnects, or a poll errors (→ `Err`, caller decides to reconnect).
    pub fn poll(&mut self, run: &AtomicBool) -> Result<(), PollerError<T::Error>> {
        while run.load(Ordering::Acquire) {
            self.transport
                .poll(self.fragment_limits)
                .map_err(PollerError::Poll)?;
            if !self.transport.is_connected() {
                return Err(PollerError::Disconnected);
            }
        }
        Ok(())
    }
}

/// Error from a poll loop: either the underlying transport's poll failed,
/// or the subscription lost its image.
#[derive(Debug)]
pub enum PollerError<E> {
    Poll(E),
    Disconnected,
}

impl<E: std::fmt::Display> std::fmt::Display for PollerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PollerError::Poll(e) => write!(f, "poll failed: {e}"),
            PollerError::Disconnected => write!(f, "transport disconnected"),
        }
    }
}
