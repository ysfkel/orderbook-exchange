use super::accepted_order::OrderPublisher;
use crate::network::config::{ACCEPTED_ORDER_CHANNEL, ACCEPTED_ORDER_STREAM_ID, AERON_DIR};
use common::{
    queue::{QueueConsumer, QueueRecvError, RingBufferConsumer},
    types::AcceptedOrder,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tracing::{error, info, warn};
use transport::{AeronTransport, PublishError, Publisher};

const RECONNECT_DELAY: Duration = Duration::from_secs(1);

pub struct AcceptedOrderPublisher {
    consumer: RingBufferConsumer<AcceptedOrder>,
    shutdown: Arc<AtomicBool>,
    pub publish_count: u64,
    pub dropped_count: u64,
}

impl AcceptedOrderPublisher {
    pub fn new(consumer: RingBufferConsumer<AcceptedOrder>, shutdown: Arc<AtomicBool>) -> Self {
        Self {
            consumer,
            shutdown,
            publish_count: 0,
            dropped_count: 0,
        }
    }

    fn run_inner<T>(&mut self, publisher: &OrderPublisher<T>) -> Result<(), PublishError>
    where
        T: Publisher,
        T::Error: Into<PublishError>,
    {
        while !self.shutdown.load(Ordering::Relaxed) {
            match self.consumer.pop() {
                Ok(order) => match publisher.publish_order(order) {
                    Ok(()) => self.publish_count += 1,
                    Err(e) if e.is_retryable() => {
                        // micro-retries already exhausted in publish_order
                        // drop + count, do NOT reconnect
                        self.dropped_count += 1;
                    }
                    Err(e) => return Err(e),
                },
                Err(QueueRecvError::Empty) => std::hint::spin_loop(),
                Err(QueueRecvError::Disconnected) => {
                    self.shutdown.store(true, Ordering::Relaxed);
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}

pub fn start_accepted_order_publisher(
    consumer: RingBufferConsumer<AcceptedOrder>,
    shutdown: Arc<AtomicBool>,
) {
    let mut outbound = AcceptedOrderPublisher::new(consumer, Arc::clone(&shutdown));

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        let aeron = match AeronTransport::connect(&AERON_DIR) {
            Ok(a) => a,
            Err(e) => {
                std::thread::sleep(RECONNECT_DELAY);
                continue;
            }
        };

        let p = match aeron.publisher(ACCEPTED_ORDER_CHANNEL, ACCEPTED_ORDER_STREAM_ID) {
            Ok(p) => p,
            Err(e) => {
                std::thread::sleep(RECONNECT_DELAY);
                continue;
            }
        };

        let publisher = OrderPublisher::new(p);

        match outbound.run_inner(&publisher) {
            Ok(()) => {
                if outbound.shutdown.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(RECONNECT_DELAY);
            }
            Err(e) => {
                std::thread::sleep(RECONNECT_DELAY);
            }
        }
    }
}
