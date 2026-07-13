 
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use transport::AeronTransport;
use transport::Subscriber;

use crate::error::ProgramError;
use crate::network::{handle_message, message_handler};

const ORDER_CHANNEL: &str = "aeron:udp?endpoint=127.0.0.1:40456";
const AERON_DIR: &str = "/tmp/aeron-exchange";
const ORDER_STREAM_ID: i32 = 1001;

pub fn run() {
    let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");
    let  messages_received = Arc::new(AtomicU64::new(0));
    let  error_count = Arc::new(AtomicU64::new(0));

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

    let p = tp
        .add_subscription(ORDER_CHANNEL, ORDER_STREAM_ID, &fragment_handler)
        .expect("failed to get subscriber");

    let mut tx = OrderSubscriber::new(p).expect("failed to create publisher");
    match tx.subscribe() {
        Ok(r) => println!("subscription ended: {:?}", r),
        Err(e) => eprintln!("subscription error: {:?}", e),
    }
}

pub struct OrderSubscriber<T: Subscriber> {
    transport: T,
}

impl<T: Subscriber> OrderSubscriber<T>
where
    ProgramError: From<T::Error>,
{
    pub fn new(transport: T) -> Result<Self, ProgramError> {
        Ok(Self { transport })
    }

    pub fn subscribe(&mut self) -> Result<(), ProgramError> {
        loop {
            match self.transport.poll(10) {
                Ok(_) => {}
                Err(e) => return Err(e.into()),
            }
           if !self.transport.is_connected() {
              return Err(ProgramError::TransportDisconnected)
           }
        }
    }
}

