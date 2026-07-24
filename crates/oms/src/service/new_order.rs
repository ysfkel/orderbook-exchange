use common::{
    queue::{
        QueueConsumer, QueueProducer, QueueRecvError, QueueSendError, RingBufferConsumer,
        RingBufferProducer,
    }, thread_handle::ThreadHandle, traits::ThreadHandler, types::{AcceptedOrder, CreateOrderMessage, NewOrder, OrderId, error::{Disposition, RejectReason}},
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, atomic::AtomicU64};
use tracing::{error, info, warn};

use crate::{error::ProgramError, types::OutboundMessageType};

/// Bounded backpressure: how many times to re-attempt a push onto a full
/// outbound ring before rejecting the order. Bounded so a stalled publisher
/// degrades into visible rejects instead of an invisible livelock.
const OUTBOUND_PUSH_RETRIES: u32 = 1024;


/// Owns the consumer side of the inbound new-order ring buffer.
///
/// The Aeron listener thread (`start_new_order_listener`) pushes onto the
/// `producer` half of the same queue; this service drains the `consumer`
/// half. Eventually this is where risk checks happen before handing the
/// order off to a downstream publisher — hence the commented-out fields
/// below as placeholders for that.
pub struct NewOrderService {
    new_order_consumer: RingBufferConsumer<NewOrder>,
    outbound_new_order_producer: RingBufferProducer<OutboundMessageType>,
    dropped_orders: AtomicU64,
    // risk: RiskEngine
    run: Arc<AtomicBool>,
    processed_orders: u64,
    forwarded: u64,
    rejected: u64,
}

impl NewOrderService {
    pub fn new(
        new_order_consumer: RingBufferConsumer<NewOrder>,
        outbound_new_order_producer: RingBufferProducer<OutboundMessageType>,
    ) -> Self {
        Self {
            new_order_consumer,
            outbound_new_order_producer,
            dropped_orders: AtomicU64::new(0),
            run: Arc::new(AtomicBool::new(false)),
            processed_orders: 0,
            rejected: 0,
            forwarded: 0,
        }
    }

    fn run_loop(mut self) {
        let run = self.run.clone();
        while run.load(Ordering::Acquire) {
            match self.new_order_consumer.pop() {
                Ok(order) => {
                    if !self.handle(&order) {
                        return; //fatal, already logged
                    }
                }
                Err(QueueRecvError::Empty) => std::hint::spin_loop(),
                // Listener stopped and dropped its producer: input is
                // finished and fully consumed.
                Err(QueueRecvError::Disconnected) => break,
            }
        }

        // stop() called. Precondition (main stops the listener first):
        // the inbound ring can only shrink, so this drain terminates.
        while let Ok(order) = self.new_order_consumer.pop() {
            if !self.handle(&order) {
                return;
            }
        }

        info!(
            forwarded = self.forwarded,
            rejected = self.rejected,
            "new-order service exiting"
        );
    }

    /// Process one order and record its disposition.
    /// Returns false only on a fatal pipeline error (already logged).
    fn handle(&mut self, order: &NewOrder) -> bool {
        match self.process_new_order(order) {
            Ok(Disposition::Forwarded) => {
                self.forwarded += 1;
                true
            }
            Ok(Disposition::Rejected(reason)) => {
                self.rejected += 1;

                warn!(
                    client_id = order.client_id,
                    client_order_id = order.client_order_id,
                    ?reason,
                    "order rejected"
                );

                true
            }
            Err(e) => {
                error!(error = %e, "pipeline broken, new-order service exiting");
                false
            }
        }
    }

    /// Push with bounded backpressure. A transiently full ring absorbs
    /// bursts; a persistently full ring becomes an explicit `SystemBusy`
    /// reject — never a silent drop.

    /// Per-order processing — risk checks, then publish. Left as a stub per
    /// your original signature; note it'll likely need `&mut self` and an
    /// order argument once it actually does something with `self.consumer`.
    pub fn process_new_order(&mut self, order: &NewOrder) -> Result<Disposition, ProgramError> {
        if let Some(reason) = Self::validate(order) {
            return Ok(Disposition::Rejected(reason));
        }
        // TODO!  perform riks checks on the order and if it passes, then publish to downstream publisher
        // TODO! generate market_order_id

        // TODO! Using a stud market order id for now, will implement market order id generation logic later
        let market_order_id = get_market_order_id_stub();

        let ticker_id = 0;

        let accepted_order = AcceptedOrder::new(
            order.price,
            order.quantity,
            order.timestamp,
            order.client_order_id,
            market_order_id,
            ticker_id,
            order.client_id,
            order.side,
            order.order_type,
        );

        let msg = OutboundMessageType::New(CreateOrderMessage::new(accepted_order));

        self.forward(msg)
    }

    /// Pre-risk validation. `None` = pass. Extend here as risk checks land;
    /// each new check is a new `RejectReason` variant, not a new error path.
    fn validate(order: &NewOrder) -> Option<RejectReason> {
        if order.price <= 0 {
            return Some(RejectReason::InvalidPrice);
        }
        if order.quantity == 0 {
            return Some(RejectReason::InvalidQuantity);
        }
        if matches!(order.side, common::types::OrderSide::Unset) {
            return Some(RejectReason::InvalidSide);
        }
        None
    }

    fn forward(&mut self, order_msg: OutboundMessageType) -> Result<Disposition, ProgramError> {
        let mut order = order_msg;
        for _ in 0..OUTBOUND_PUSH_RETRIES {
            match self.outbound_new_order_producer.push(order) {
                Ok(_) => {
                    info!("oms: message forwarded sucessfuly");
                    return Ok(Disposition::Forwarded);
                }
                Err(QueueSendError::Full(returned)) => {
                    error!("oms: message forwarded error - returned");
                    order = returned;
                    std::hint::spin_loop();
                }
                Err(QueueSendError::Disconnected(_)) => {
                    error!("oms: message forwarded error - disconnected");
                    return Err(ProgramError::OutboundQueueDisconnected);
                }
            }
        }
        Ok(Disposition::Rejected(RejectReason::SystemBusy))
    }
}

impl ThreadHandler for NewOrderService {
    fn start(self) -> ThreadHandle {
        self.run.store(true, Ordering::Release);
        let run = self.run.clone();
        let thread = std::thread::Builder::new()
            .name("oms-new-order-service".into())
            .spawn(move || self.run_loop())
            .expect("failed to spawn oms-new-order-service");
        ThreadHandle {
            run,
            thread: Some(thread),
        }
    }
}

fn get_market_order_id_stub() -> OrderId {
    // TODO!  implement market order id generation logic
    0
}
