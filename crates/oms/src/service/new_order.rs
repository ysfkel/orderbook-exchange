use common::{
    queue::{QueueConsumer, RingBufferConsumer},
    types::NewOrder,
};
use tracing::info;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Owns the consumer side of the inbound new-order ring buffer.
///
/// The Aeron listener thread (`start_new_order_listener`) pushes onto the
/// `producer` half of the same queue; this service drains the `consumer`
/// half. Eventually this is where risk checks happen before handing the
/// order off to a downstream publisher — hence the commented-out fields
/// below as placeholders for that.
pub struct NewOrderService {
    consumer: RingBufferConsumer<NewOrder>,
    // risk: RiskEngine
    // publisher: NewOrderPublisher
}

impl NewOrderService {
    pub fn new(consumer: RingBufferConsumer<NewOrder>) -> Self {
        Self { consumer }
    }

    /// Per-order processing — risk checks, then publish. Left as a stub per
    /// your original signature; note it'll likely need `&mut self` and an
    /// order argument once it actually does something with `self.consumer`.
    pub fn process_new_order(&self) {}

    /// Drains the consumer until `shutdown` is set. This is the loop
    /// `main.rs` spawns on its own thread — it's what actually ties
    /// `consumer` to `process_new_order` on an ongoing basis.
    pub fn run(&mut self, shutdown: Arc<AtomicBool>) {
        while !shutdown.load(Ordering::Relaxed) {
            match self.consumer.pop() {
                Ok(_order) => {

                    info!("[SERVICE]:: ->-> New order received from consumer");
                    // todo: pass `_order` into process_new_order once its
                    // signature takes one; for now this just drains the queue.
                    self.process_new_order();
                }
                Err(_e) => {
                    // `pop()` returning Err here is assumed to just mean
                    // "queue is empty right now," not a real fault — worth
                    // confirming against QueueConsumer's error type. If Err
                    // can also signal a genuine problem (e.g. a torn or
                    // corrupted read), this branch should distinguish that
                    // case and log/escalate rather than silently yielding.
                    //
                    // Avoid a hot spin when there's nothing to do. Swap for
                    // a short park/backoff if this ends up burning too much
                    // CPU.
                    std::thread::yield_now();
                }
            }
        }
    }
}