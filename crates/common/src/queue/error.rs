use std::fmt;

pub enum QueueSendError<T> {
    Full(T),         // backpressure — item returned, caller decides
    Disconnected(T), // consumer side gone, permanent
}

impl<T> fmt::Debug for QueueSendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueSendError::Full(_) => write!(f, "QueueSendError::Full"),
            QueueSendError::Disconnected(_) => write!(f, "QueueSendError::Disconnected"),
        }
    }
}

impl<T> fmt::Display for QueueSendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueSendError::Full(_) => write!(f, "queue full"),
            QueueSendError::Disconnected(_) => write!(f, "queue consumer disconnected"),
        }
    }
}

impl<T> std::error::Error for QueueSendError<T> {}

/// Result of a failed pop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueRecvError {
    Empty,        // transient, poll again next cycle
    Disconnected, // producer side gone, permanent — stop polling this queue
}

impl fmt::Display for QueueRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueRecvError::Empty => write!(f, "queue empty"),
            QueueRecvError::Disconnected => write!(f, "queue producer disconnected"),
        }
    }
}

impl std::error::Error for QueueRecvError {}
