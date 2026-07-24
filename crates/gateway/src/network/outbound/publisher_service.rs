use super::publisher::OrderPublisher;
use crate::{network::config::{NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID, AERON_DIR}, types::OutboundMessageType};
use common::{
    queue::{QueueConsumer, QueueRecvError, RingBufferConsumer}, thread_handle::ThreadHandle, traits::ThreadHandler, types::{AcceptedOrder, CreateOrderMessage},
};
use std::time::Duration;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use transport::{AeronTransport, PublishError, Publisher};

const RECONNECT_DELAY: Duration = Duration::from_secs(1);

pub struct PublisherService {
    consumer: RingBufferConsumer<OutboundMessageType>,
    run: Arc<AtomicBool>,
    pub publish_count: u64,
    pub dropped_count: u64,
}


impl PublisherService {
    pub fn new(consumer: RingBufferConsumer<OutboundMessageType>) -> Self {
        Self {
            consumer,
            run: Arc::new(AtomicBool::new(false)),
            publish_count: 0,
            dropped_count: 0,
        }
    }

    fn run_loop(mut self) {
        let run = self.run.clone();
        while run.load(Ordering::Acquire) {
            let aeron = match AeronTransport::connect(AERON_DIR) {
                Ok(a) => a,
                Err(_) => {
                    thread::sleep(RECONNECT_DELAY);
                    continue;
                }
            };

            let p = match aeron.publisher(NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID) {
                Ok(p) => p,
                Err(_) => {
                    thread::sleep(RECONNECT_DELAY);
                    continue;
                }
            };

            let publisher = OrderPublisher::new(p);

            match self.pump(&publisher, &run) {
                Ok(()) => break,
                Err(e) => {
                    std::thread::sleep(RECONNECT_DELAY);
                }
            }
        }
    }

    fn pump<T>(
        &mut self,
        publisher: &OrderPublisher<T>,
        run: &AtomicBool,
    ) -> Result<(), PublishError>
    where
        T: Publisher,
        T::Error: Into<PublishError>,
    {
        while run.load(Ordering::Acquire) {
            match self.consumer.pop() {
                Ok(order) => self.publish_one(publisher, order)?,
                Err(QueueRecvError::Empty) => std::hint::spin_loop(),
                Err(QueueRecvError::Disconnected) => return Ok(()),
            }
        }

        // stop() called. Precondition (main's stop order): the service feeding
        // this ring is already stopped, so this terminates.
        while let Ok(order) = self.consumer.pop() {
            self.publish_one(publisher, order)?;
        }
        Ok(())
    }

    fn publish_one<T>(
        &mut self,
        publisher: &OrderPublisher<T>,
        msg: OutboundMessageType,
    ) -> Result<(), PublishError>
    where
        T: Publisher,
        T::Error: Into<PublishError>,
    {
    
        match publisher.publish(msg) {
            Ok(()) => {
                self.publish_count += 1;
                Ok(())
            }
            Err(e) if e.is_retryable() => {
                // micro-retries already exhausted inside publish_order:
                // drop + count, don't tear down the connection
                self.dropped_count += 1;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl ThreadHandler for PublisherService {

    fn start(self) -> ThreadHandle {
        self.run.store(true, Ordering::Release);
        let run = self.run.clone();
        let thread = std::thread::Builder::new()
            .name("gateway-order-publisher".into())
            .spawn(move || self.run_loop())
            .expect("failed to spawn gateway-order-publisher");
        ThreadHandle {
            run,
            thread: Some(thread),
        }
    }
}
