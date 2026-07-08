 use crate::error::ProgramError;
use crate::network::config::{AERON_DIR, NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID};
use crate::network::inbound::message_handler::handle_message;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use common::{types::NewOrder, queue::RingBufferProducer};
use tracing::{error, info};
use transport::Subscriber;
use transport::{AeronTransport, PollError};

const RECONNECT_DELAY: Duration = Duration::from_secs(1);

/// Runs the inbound new-order listener until `shutdown` is set.
///
/// `producer` is moved into the message-handling closure exactly ONCE, up
/// front — not per reconnect attempt. This is the key fix: the old version
/// called `aeron.subscriber(...)` (which built a brand-new closure capturing
/// `producer` by move) *inside* the reconnect loop. That only compiles for
/// the first iteration; on the second reconnect there's no `producer` left
/// to move. Splitting "build the handler" (process-lifetime) from "create a
/// subscription" (per-connection-lifetime) removes that problem entirely,
/// with no Mutex or Arc needed around `producer` — only one thread, one
/// closure, ever touches it.
pub fn start_new_order_listener(
    shutdown: Arc<AtomicBool>,
    max_message_size: usize,
    mut producer: RingBufferProducer<NewOrder>,
    
) {
    let push_error_count = Arc::new(AtomicU64::new(0));
    let push_error_count_handler = push_error_count.clone();
    // ── Build the message handler ONCE ─────────────────────────────────────
    // `producer` moves into this closure here and only here. The closure
    // itself is leaked (via Handler::leak_with_fragment_assembler inside
    // build_fragment_handler) so it lives for the rest of the process.
    let fragment_handler = match AeronTransport::build_fragment_handler(move |msg: &[u8]| {
        if handle_message(msg, max_message_size, &mut producer).is_err() {

            // A push failure here means a *valid* order failed to enter the
            // system — not the same severity as a malformed packet. Count
            // it so it's visible, rather than only ever a scrolling log line.
            push_error_count_handler.fetch_add(1, Ordering::Relaxed);
        }
    }) {
        Ok(h) => h,
        Err(e) => {
            error!(error = %e, "fatal: could not build message handler");
            return;
        }
    };

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        // ── Step 1: connect to aeronmd ────────────────────────────────────
        let aeron = match AeronTransport::connect(&AERON_DIR) {
            Ok(a) => a,
            Err(e) => {
                error!(error = %e, "failed to connect to aeron, retrying");
                std::thread::sleep(RECONNECT_DELAY);
                continue;
            }
        };

        // ── Step 2: create subscription, borrowing the handler built above ─
        // No closure is created here anymore — just a new AeronSubscription
        // wired up to the same (already-built) fragment_handler. This is
        // what makes it safe to run every reconnect attempt: it borrows
        // `fragment_handler`, it doesn't consume it.
        // AeronTransport::add_subscription(channel, stream_id, &fragment_handler) — called on every
        //  reconnect attempt. It only takes a reference (&'a FragmentHandler<F>) to the handler
        //  built above, so it never needs to move producer again — it's just wiring up a new Aeron 
        // subscription to point at the same already-built closure.
        let subscriber = match aeron.add_subscription(
            NEW_ORDER_CHANNEL,
            NEW_ORDER_STREAM_ID,
            &fragment_handler,
        ) {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "failed to create subscriber, retrying");
                std::thread::sleep(RECONNECT_DELAY);
                continue;
            }
        };

        // ── Step 3: create listener ───────────────────────────────────────
        let mut listener = match NewOrderListener::new(subscriber, shutdown.clone()) {
            Ok(l) => l,
            Err(e) => {
                error!(error = %e, "fatal: could not initialize listener");
                break;
            }
        };

        // ── Step 4: run ───────────────────────────────────────────────────
        match listener.run() {
            Ok(_) => {
                if shutdown.load(Ordering::Relaxed) {
                    info!(
                        poll_count = listener.poll_count,
                        malformed = listener.malformed_count,
                        "listener exited cleanly"
                    );
                    break;
                }
                error!("listener disconnected, reconnecting");
                std::thread::sleep(RECONNECT_DELAY);
            }
            Err(e) => {
                if shutdown.load(Ordering::Relaxed) {
                    info!(
                        poll_count = listener.poll_count,
                        malformed = listener.malformed_count,
                        "listener exited cleanly"
                    );
                    break;
                }
                error!(error = %e, "listener failed, reconnecting");
                std::thread::sleep(RECONNECT_DELAY);
            }
        }
    }
}

pub struct NewOrderListener<T: Subscriber> {
    subscriber: T,
    shutdown: Arc<AtomicBool>,
    pub poll_count: u64,
    pub malformed_count: u64,
    pub push_error_count: Arc<AtomicU64>,
}

impl<T: Subscriber> NewOrderListener<T>
where
    ProgramError: From<T::Error>,
{
    pub fn new(subscriber: T, shutdown: Arc<AtomicBool>) -> Result<Self, ProgramError> {
        Ok(Self {
            subscriber,
            shutdown,
            poll_count: 0,
            malformed_count: 0,
            push_error_count: Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn run(&mut self) -> Result<(), ProgramError> {
        while !self.shutdown.load(Ordering::Relaxed) {
            match self.subscriber.poll(10) {
                Ok(_) => self.poll_count += 1,
                Err(e) => return Err(e.into()),
            }

            if !self.subscriber.is_connected() {
                return Err(ProgramError::TransportSubscriptionError(
                    PollError::DriverTimeout(0),
                ));
            }
        }

        Ok(())
    }
}