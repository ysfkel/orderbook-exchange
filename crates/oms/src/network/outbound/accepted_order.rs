



use transport::{Publisher, Subscriber};
use crate::error::ProgramError;

pub struct OrderBookPublisher<T: Publisher> {
    publisher: T,
}

impl<T: Publisher> OrderBookPublisher<T> where ProgramError: From<T::Error>{

    pub fn new(publisher: T) -> Result<Self, ProgramError> {
          
           //publisher.
    
          Ok(Self {
            publisher
          })
    }

    pub fn publish(&self, bytes: &[u8]) -> Result<T::Data, ProgramError> {
       let r =  self.publisher.publish(bytes)?;

       Ok(r)
    }
}



 