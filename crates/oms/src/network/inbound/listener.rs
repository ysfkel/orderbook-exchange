use crate::network::config::{AERON_DIR, NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID};
use crate::network::inbound::message_handler::handle_message;
use common::thread_handle::ThreadHandle;
use common::traits::ThreadHandler;
use common::{queue::RingBufferProducer, types::NewOrder};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use tracing::info;
use transport::{AeronTransport, Poller};

const RECONNECT_DELAY: Duration = Duration::from_secs(1);

pub struct Listener {
    producer: RingBufferProducer<NewOrder>,
    run: Arc<AtomicBool>,
}

impl Listener {
    pub fn new(producer: RingBufferProducer<NewOrder>) -> Self {
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
                // ───────────────────
                // Shouldn't happen in normal operation — log it loudly
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
                .add_subscription(NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID, &fragment_handler)
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

// /// Runs the inbound new-order listener until `shutdown` is set.
// ///
// /// `producer` is moved into the message-handling closure exactly ONCE, up
// /// front — not per reconnect attempt. This is the key fix: the old version
// /// called `aeron.subscriber(...)` (which built a brand-new closure capturing
// /// `producer` by move) *inside* the reconnect loop. That only compiles for
// /// the first iteration; on the second reconnect there's no `producer` left
// /// to move. Splitting "build the handler" (process-lifetime) from "create a
// /// subscription" (per-connection-lifetime) removes that problem entirely,
// /// with no Mutex or Arc needed around `producer` — only one thread, one
// /// closure, ever touches it.
// pub fn start_new_order_listener(
//     shutdown: Arc<AtomicBool>,
//     max_message_size: usize,
//     mut producer: RingBufferProducer<NewOrder>,
// ) {
//     let messages_received = Arc::new(AtomicU64::new(0));
//     let error_count = Arc::new(AtomicU64::new(0));
//     let messages_received_clone = Arc::clone(&messages_received);
//     let error_count_clone = Arc::clone(&error_count);

//     // ── Build the message handler ONCE ─────────────────────────────────────
//     // `producer` moves into this closure here and only here. The closure
//     // itself is leaked (via Handler::leak_with_fragment_assembler inside
//     // build_fragment_handler) so it lives for the rest of the process.
//     let fragment_handler = AeronTransport::build_fragment_handler(move |msg: &[u8]| {
//         match handle_message(msg, max_message_size, &mut producer) {
//             Ok(()) => {}
//             Err(_) => {
//                 error_count_clone.fetch_add(1, Ordering::Relaxed);
//             }
//         }
//     })
//     .expect("failed to build fragment handler");

//     loop {
//         if shutdown.load(Ordering::Relaxed) {
//             break;
//         }

//         // connect to aeronmd ────────────────────────────────────
//         let aeron = match AeronTransport::connect(&AERON_DIR) {
//             Ok(a) => a,
//             Err(e) => {
//                 error!(error = %e, "failed to connect to aeron, retrying");
//                 std::thread::sleep(RECONNECT_DELAY);
//                 continue;
//             }
//         };

//         // create subscription, borrowing the handler built above ─
//         // No closure is created here anymore — just a new AeronSubscription
//         // wired up to the same (already-built) fragment_handler. This is
//         // what makes it safe to run every reconnect attempt: it borrows
//         // `fragment_handler`, it doesn't consume it.
//         // AeronTransport::add_subscription(channel, stream_id, &fragment_handler) — called on every
//         //  reconnect attempt. It only takes a reference (&'a FragmentHandler<F>) to the handler
//         //  built above, so it never needs to move producer again — it's just wiring up a new Aeron
//         // subscription to point at the same already-built closure.
//         let sub = aeron
//             .add_subscription(NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID, &fragment_handler)
//             .expect("failed to get subscriber");

//         // ── Step 3: create listener ───────────────────────────────────────
//         let mut subscriber = OrderSubscriber::new(sub);

//         // run ───────────────────────────────────────────────────
//         match subscriber.subscribe() {
//             Ok(_) => {
//                 if shutdown.load(Ordering::Relaxed) {
//                     info!(
//                         received = messages_received.load(Ordering::Relaxed),
//                         errors = error_count.load(Ordering::Relaxed),
//                         "engine subscriber exited cleanly"
//                     );
//                     break;
//                 }
//                 tracing::error!("subscriber disconnected, reconnecting");
//                 thread::sleep(Duration::from_secs(1));
//             }
//             Err(e) => {
//                 tracing::error!(error = %e, "subscriber failed, reconnecting");
//                 thread::sleep(Duration::from_secs(1));
//             }
//         }
//     }
// }

// pub struct OrderSubscriber<T: Subscriber> {
//     transport: T,
// }

// impl<T: Subscriber> OrderSubscriber<T>
// where
//     ProgramError: From<T::Error>,
// {
//     pub fn new(transport: T) -> Self {
//         Self { transport }
//     }

//     pub fn subscribe(&mut self) -> Result<(), ProgramError> {
//         loop {
//             match self.transport.poll(10) {
//                 Ok(_) => {}
//                 Err(e) => return Err(e.into()),
//             }
//             if !self.transport.is_connected() {
//                 return Err(ProgramError::TransportDisconnected);
//             }
//         }
//     }
// }
