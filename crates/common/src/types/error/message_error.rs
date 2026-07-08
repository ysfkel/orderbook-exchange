use zerocopy::ConvertError;

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
    #[error("Message too large: {size} bytes (max {max})")]
    MessageTooLarge { size: usize, max: usize },

    #[error("Failed to Deserialize Message Header: {0}")]
    DeserializeMessageHeader(DeserializeErrorKind),

    #[error("Failed to Deserialize Message Body: {0}")]
    DeserializeMessageBody(DeserializeErrorKind),
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

 