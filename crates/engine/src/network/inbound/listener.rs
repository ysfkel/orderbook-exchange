use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use tracing::info;
use transport::AeronTransport;
use transport::Subscriber;

use crate::error::ProgramError;
use super::handle_message;

const ORDER_CHANNEL: &str = "aeron:udp?endpoint=127.0.0.1:40456";
const AERON_DIR: &str = "/tmp/aeron-exchange";
const ORDER_STREAM_ID: i32 = 1001;

pub fn run(shutdown: Arc<AtomicBool>) {
   // let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");
    let messages_received = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));
    let messages_received_clone = Arc::clone(&messages_received);
    let error_count_clone = Arc::clone(&error_count);

    // Same split as new_order_listener.rs: build the message-handling
    // closure once (this is where anything you capture — here, nothing
    // beyond the printlns — would move in), then create the subscription
    // as a separate step that just borrows it. `fragment_handler` must
    // stay alive for as long as `p` (and anything built from it) is used,
    // which is why it's a local binding here rather than a temporary.
    let fragment_handler = AeronTransport::build_fragment_handler(move |bytes: &[u8]| {
        //  message_handler::handle_message(bytes);

        match handle_message(bytes) {
            Ok(()) => {
                messages_received_clone.fetch_add(1, Ordering::Relaxed);
            }
            Err(ProgramError::DeserializeMessageHeader(e)) => {
                error_count_clone.fetch_add(1, Ordering::Relaxed);
            }

            Err(ProgramError::DeserializeMessageBody(e)) => {
                error_count_clone.fetch_add(1, Ordering::Relaxed);
            }

            // ───────────────────
            // Shouldn't happen in normal operation — log it loudly
            Err(e) => {
                error_count_clone.fetch_add(1, Ordering::Relaxed);
            }
        }
    })
    .expect("failed to build fragment handler");
    loop {
        if shutdown.load(Ordering::Relaxed) {break;}

        let tp = match AeronTransport::connect(&AERON_DIR) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(error = %e, "aeron connect failed, retrying");
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let sub = tp
            .add_subscription(ORDER_CHANNEL, ORDER_STREAM_ID, &fragment_handler)
            .expect("failed to get subscriber");

        let mut tx = OrderSubscriber::new(sub);

        match tx.subscribe() {
            Ok(_) => {
                if shutdown.load(Ordering::Relaxed) {
                    info!(
                        received = messages_received.load(Ordering::Relaxed),
                        errors = error_count.load(Ordering::Relaxed),
                        "engine subscriber exited cleanly"
                    );
                    break;
                }
                tracing::error!("subscriber disconnected, reconnecting");
                thread::sleep(Duration::from_secs(1));
            }
            Err(e) => {
                tracing::error!(error = %e, "subscriber failed, reconnecting");
                thread::sleep(Duration::from_secs(1));
            },
        }
    }
}

pub struct OrderSubscriber<T: Subscriber> {
    transport: T,
}

impl<T: Subscriber> OrderSubscriber<T>
where
    ProgramError: From<T::Error>,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn subscribe(&mut self) -> Result<(), ProgramError> {
        loop {
            match self.transport.poll(10) {
                Ok(_) => {}
                Err(e) => return Err(e.into()),
            }
            if !self.transport.is_connected() {
                return Err(ProgramError::TransportDisconnected);
            }
        }
    }
}
