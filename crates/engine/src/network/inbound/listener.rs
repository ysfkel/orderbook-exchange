use common::queue::RingBufferProducer;
use common::thread_handle::ThreadHandle;
use common::traits::ThreadHandler;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use tracing::info;
use transport::{AeronTransport, Poller};

use super::handle_message;
use crate::types::EngineRequest;

const ORDER_CHANNEL: &str = "aeron:udp?endpoint=127.0.0.1:40456";
const AERON_DIR: &str = "/tmp/aeron-exchange";
const ORDER_STREAM_ID: i32 = 1001;

pub struct Listener {
    producer: RingBufferProducer<EngineRequest>,
    run: Arc<AtomicBool>,
}

impl Listener {
    pub fn new(producer: RingBufferProducer<EngineRequest>) -> Self {
        Self {
            producer,
            run: Arc::new(AtomicBool::new(false)),
        }
    }

    fn listen(self) {
        // Destructure: producer moves into the fragment handler,
        // run stays for the loops.
        let Listener { mut producer, run } = self;

        let messages_received = AtomicU64::new(0);
        let error_count = AtomicU64::new(0);

        // Same split as new_order_listener.rs: build the message-handling
        // closure once (this is where anything you capture — here, nothing
        // beyond the printlns — would move in), then create the subscription
        // as a separate step that just borrows it. `fragment_handler` must
        // stay alive for as long as `p` (and anything built from it) is used,
        // which is why it's a local binding here rather than a temporary.
        let fragment_handler = AeronTransport::build_fragment_handler(move |bytes: &[u8]| {
            match handle_message(bytes, &mut producer) {
                Ok(()) => {
                    messages_received.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    error_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        })
        .expect("failed to build fragment handler");

        while run.load(Ordering::Acquire) {
            // was: !shutdown
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

            match Poller::new(sub, 10).poll(&run) {
                // ← flag goes in
                Ok(()) => break, // clean stop
                Err(e) => {
                    tracing::error!(error = %e, "subscriber failed, reconnecting");
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
        info!("order listener exiting");
    }
}

impl ThreadHandler for Listener {
    fn start(self) -> ThreadHandle {
        self.run.store(true, Ordering::Release);
        let run = self.run.clone();
        let thread = std::thread::Builder::new()
            .name("order-listener".into())
            .spawn(move || self.listen())
            .expect("failed to spawn order-listener");
        ThreadHandle {
            run,
            thread: Some(thread),
        }
    }
}

// pub struct Poller<T: Subscriber> {
//     transport: T,
//     fragment_limits: usize
// }

// impl<T: Subscriber> Poller<T>
// where
//     ProgramError: From<T::Error>,
// {
//     pub fn new(transport: T, fragment_limits: usize) -> Self {
//         Self { transport , fragment_limits}
//     }

//        /// Poll until `run` goes false (clean stop → `Ok`), the transport
//     /// disconnects, or a poll errors (→ `Err`, caller decides to reconnect).
//     pub fn poll(&mut self, run: &AtomicBool) -> Result<(), ProgramError> {
//     while run.load(Ordering::Acquire) {
//         self.transport.poll(self.fragment_limits)?;
//         if !self.transport.is_connected() {
//             return Err(ProgramError::TransportDisconnected);
//         }
//     }
//     Ok(())   // run went false: clean shutdown
// }
// }
