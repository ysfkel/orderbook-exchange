use common::types::MAX_OFFER_RETRIES;
use transport::{PublishError, Publisher};
use zerocopy::IntoBytes;
use crate::types::OutboundMessageType;

pub struct OrderPublisher<T: Publisher> {
    publisher: T,
}

impl<T: Publisher> OrderPublisher<T>
where
    T::Error: Into<PublishError>,
{
    pub fn new(publisher: T) -> Self {
        Self { publisher }
    }

    pub fn publish(&self, msg: OutboundMessageType) -> Result<(), PublishError> {
        let bytes: &[u8] = match &msg {
            OutboundMessageType::New(msg) => msg.as_bytes(),
            // ..other messages
         };
        for attempt in 0..MAX_OFFER_RETRIES {
            match self.publisher.publish(bytes).map_err(Into::into) {
                Ok(_) => return Ok(()),
                Err(e) if e.is_retryable() => {
                    if attempt < MAX_OFFER_RETRIES - 1 {
                        std::hint::spin_loop();
                    } else {
                        return Err(e.into());
                    }
                }
                Err(e) => return Err(e.into()), // closed
            }
        }

        Ok(())
    }
 
}
