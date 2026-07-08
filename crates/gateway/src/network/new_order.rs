use crate::error::ProgramError;
use transport::{Publisher, Subscriber};

pub struct OrderPublisher<T: Publisher> {
    publisher: T,
}

impl<T: Publisher> OrderPublisher<T>
where
    ProgramError: From<T::Error>,
{
    pub fn new(publisher: T) -> Result<Self, ProgramError> {
        Ok(Self { publisher })
    }

    pub fn publish(&self, bytes: &[u8]) -> Result<T::Data, ProgramError> {
        let r = self.publisher.publish(bytes)?;

        Ok(r)
    }
}
