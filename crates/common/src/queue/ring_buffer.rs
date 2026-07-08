use rtrb::{RingBuffer, PushError, PopError};
use super::{QueueProducer, QueueConsumer};
use super::error::{QueueSendError, QueueRecvError};

pub struct RingBufferQueue<T> {
     pub producer: RingBufferProducer<T>,
     pub consumer: RingBufferConsumer<T> 
}

impl<T> RingBufferQueue<T> {

    pub fn new(capacity: usize) -> Self {
        let (
         mut _producer, 
         mut _consumer
        ) = RingBuffer::new(capacity);

        let producer = RingBufferProducer::new(_producer);
        let consumer = RingBufferConsumer::new(_consumer);
         Self {
             producer,   
             consumer
         }
    }
}
 pub struct RingBufferProducer<T> {
    pub producer: rtrb::Producer<T>,
 }

 impl<T> RingBufferProducer<T> {
    pub fn new(producer: rtrb::Producer<T>) -> Self {
          Self {
              producer
          }
    }

    /// Number of slots currently occupied. Useful for depth metrics /
    /// gateway-level backpressure decisions, without exposing rtrb types.
    pub fn len(&self) -> usize {
        self.producer.slots()
    }

    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }
 }

 pub struct RingBufferConsumer<T> {
    pub consumer: rtrb::Consumer<T>,
 }

 impl<T> RingBufferConsumer<T> {
    pub fn new(consumer: rtrb::Consumer<T>) -> Self {
        Self {
            consumer
        }
    }

    pub fn len(&self) -> usize {
        self.consumer.slots()
    }

    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }
 }

 impl<T> QueueProducer<T> for RingBufferProducer<T> {
     type Error = QueueSendError<T>;
     fn push(&mut self, item: T) -> Result<(), Self::Error> {
         self.producer.push(item).map_err(|e| match e{ 
               PushError::Full(item) => {
                if self.producer.is_abandoned() {
                    QueueSendError::Disconnected(item)
                } else {
                    QueueSendError::Full(item)
                }
               }

         })
     }
 }

 impl<T> QueueConsumer<T> for RingBufferConsumer<T> {
    type Error = QueueRecvError;

    fn pop(&mut self) -> Result<T, Self::Error> {
        self.consumer.pop().map_err(|e| match e {

            PopError::Empty => {
                if self.consumer.is_abandoned() {
                    QueueRecvError::Disconnected
                } else {
                    QueueRecvError::Empty
                }
            }
        })
    }
 }

