use crate::error::ProgramError;
use common::types::{AcceptedOrder, CreateOrderMessage, MAX_OFFER_RETRIES};
use tracing::info;
use transport::{Publisher, PublishError};
use zerocopy::IntoBytes;

pub struct OrderPublisher<T: Publisher> {
    publisher: T,
}

impl<T: Publisher> OrderPublisher<T>
where
    T::Error: Into<PublishError>,
{
    pub fn new(publisher: T) ->Self{
        Self { publisher }
    }

    pub fn publish_order(&self, accepted_order: AcceptedOrder) -> Result<(), PublishError> {
                info!(" publishing outbound..");
        let msg = CreateOrderMessage::new(accepted_order);
        let bytes = msg.as_bytes();
        for attempt in 0..MAX_OFFER_RETRIES {
          match self.publisher.publish( bytes).map_err(Into::into) {
             Ok(_) => return Ok(()),
             Err(e) if e.is_retryable() => {
                if attempt < MAX_OFFER_RETRIES - 1 {
                    std::hint::spin_loop();;
                } else {
                    return Err(e.into())
                }
             }
             Err(e) => return Err(e.into()) // closed
          }
        }

        Ok(())
    }
}
