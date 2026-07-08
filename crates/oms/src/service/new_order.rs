use common::{
    queue::{QueueConsumer, QueueProducer, QueueSendError, RingBufferConsumer, RingBufferProducer}, types::{AcceptedOrder, NewOrder, OrderId},
};
use tracing::info;
use std::sync::{Arc, atomic::AtomicU64};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::ProgramError;

/// Owns the consumer side of the inbound new-order ring buffer.
///
/// The Aeron listener thread (`start_new_order_listener`) pushes onto the
/// `producer` half of the same queue; this service drains the `consumer`
/// half. Eventually this is where risk checks happen before handing the
/// order off to a downstream publisher — hence the commented-out fields
/// below as placeholders for that.
pub struct NewOrderService {
    new_order_consumer: RingBufferConsumer<NewOrder>,
    outbound_new_order_producer: RingBufferProducer<AcceptedOrder>,
    dropped_orders: AtomicU64,
    // risk: RiskEngine
    // publisher: NewOrderPublisher
}

impl NewOrderService {
    pub fn new(new_order_consumer: RingBufferConsumer<NewOrder>, outbound_new_order_producer: RingBufferProducer<AcceptedOrder>) -> Self {
        Self { 
            new_order_consumer,
           outbound_new_order_producer,
           dropped_orders: AtomicU64::new(0)
         }
    }

    /// Per-order processing — risk checks, then publish. Left as a stub per
    /// your original signature; note it'll likely need `&mut self` and an
    /// order argument once it actually does something with `self.consumer`.
    pub fn process_new_order(&mut self, order: &NewOrder) ->  Result<(), ProgramError> {

        // TODO!  perform riks checks on the order and if it passes, then publish to downstream publisher
        // TODO! generate market_order_id 

        // TODO! Using a stud market order id for now, will implement market order id generation logic later
        let market_order_id = get_market_order_id_stub();
       
        let acceped_order = AcceptedOrder::new(
            order.price,
            order.quantity,
            order.timestamp,
            order.client_order_id,
            market_order_id,
            order.client_id,
            order.side,
            order.order_type

        );
       
       // todo! handle the error if the push fails, for now just log it
       match self.outbound_new_order_producer.push(acceped_order) {
            Ok(_) => {}
            Err(QueueSendError::Full(_)) => {

                self.dropped_orders.fetch_add(1, Ordering::Relaxed);
                // todo! send reject execution report back to client via outbound publisher
            }
            Err(QueueSendError::Disconnected(_)) => {
              return Err(ProgramError::OutboundQueueDisconnected)
            }
        
       }

       Ok(())

    }

    /// Drains the consumer until `shutdown` is set. This is the loop
    /// `main.rs` spawns on its own thread — it's what actually ties
    /// `consumer` to `process_new_order` on an ongoing basis.
    pub fn run(&mut self, shutdown: Arc<AtomicBool>) -> Result<(), ProgramError> {
        while !shutdown.load(Ordering::Relaxed) {
            match self.new_order_consumer.pop() {
                Ok(_order) => {
                    // todo: pass `_order` into process_new_order once its
                    // signature takes one; for now this just drains the queue.
                    self.process_new_order(&_order)?;  // Disconnected bubbles up and breaks the loop
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

        Ok(())
    }
}

fn get_market_order_id_stub() -> OrderId {
    // TODO!  implement market order id generation logic
    0
}