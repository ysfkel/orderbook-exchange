
use crate::{error::ProgramError, network::{config::{AERON_DIR, NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID}, new_order::OrderPublisher}, types::{OrderRequest, OutboundMessageType}};
use common::{queue::{QueueConsumer, QueueProducer, QueueRecvError, QueueSendError, RingBufferConsumer, RingBufferProducer}, thread_handle::ThreadHandle, traits::ThreadHandler, types::{NewOrder, NewOrderMessage, OrderSide, OrderType, create_order, error::{Disposition, RejectReason}}};
use crossbeam::queue::ArrayQueue;
use std::{hint::spin_loop, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread};
use tracing::{error, info, warn};
use transport::AeronTransport;
//
use std::time::{SystemTime, UNIX_EPOCH};
use zerocopy::IntoBytes;

const OUTBOUND_PUSH_RETRIES: u32 = 1024;

pub struct OrderService {
  order_consumer: Arc<ArrayQueue<OrderRequest>>,//RingBufferConsumer<OrderRequest>,
  outbound_producer: RingBufferProducer<OutboundMessageType>,
  run: Arc<AtomicBool>,
  forwarded: u64,
  rejected: u64,
}

impl OrderService{

    pub fn new(  order_consumer: Arc<ArrayQueue<OrderRequest>>,
  outbound_producer: RingBufferProducer<OutboundMessageType>) -> Self {
        Self {
            order_consumer,
            outbound_producer,
            run: Arc::new(AtomicBool::new(false)),
            forwarded:0,
            rejected:0
        }
    }

    pub fn run_loop(mut self) {
        let run = self.run.clone();
        while run.load(std::sync::atomic::Ordering::Acquire) {
            match self.order_consumer.pop() {
                Some(order) => {
                   if !self.handle(&order) {
                       return;
                   }
                }
                None => spin_loop(), 
            }
        }

        while let Some(order) = self.order_consumer.pop() {
            if !self.handle(&order) {
                return;
            }
        }
    }

    /// Process one order and record its disposition.
    /// Returns false only on a fatal pipeline error (already logged).
    fn handle(&mut self, order: &OrderRequest) -> bool {
        match self.process_new_order(order) {
            Ok(Disposition::Forwarded) => {
                self.forwarded += 1;
                true
            }
            Ok(Disposition::Rejected(reason)) => {
                self.rejected += 1;

    
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
    pub fn process_new_order(&mut self, order_request: &OrderRequest) -> Result<Disposition, ProgramError> {

        let msg = match order_request {
            OrderRequest::NewOrder(order) => {
                let order_message = NewOrderMessage::new(*order);
                let msg = OutboundMessageType::New(order_message);
                msg
            }
        };
     
        self.forward(msg)
    }


    fn forward(&mut self, order_msg: OutboundMessageType) -> Result<Disposition, ProgramError> {
        let mut order = order_msg;
        for _ in 0..OUTBOUND_PUSH_RETRIES {
            match self.outbound_producer.push(order) {
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

impl ThreadHandler for OrderService {
    fn start(self) -> common::thread_handle::ThreadHandle {
        self.run.store(true, Ordering::Release);
        let run = self.run.clone();
        let thread = std::thread::Builder::new()
        .name("gateway-order-service".into())
        .spawn(move || self.run_loop())
        .expect("failed to spawn gateway-order-service");
       ThreadHandle {
        run, 
        thread: Some(thread),
       }
    }
}

// pub fn run() {
//     let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");
//     let p = tp
//         .publisher(NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID)
//         .expect("failed to get publisher");
//     let publisher = OrderPublisher::new(p).expect("failed to create publisher");

//            let mut new_order = NewOrder::new(
//             23600,
//             18,
//             nano_now(),
//             1000,
//             0,
//             0,
//             OrderSide::Buy,
//             OrderType::MARKET,
//         );
//     loop {
 

//         let order_message = NewOrderMessage::new(new_order);

//         let create_order_bytes = order_message.as_bytes();

//         {
//             let _ = publisher.publish(create_order_bytes);
//             thread::sleep(std::time::Duration::from_secs(1));
//             info!("publishing..")
//         }

//         // just for test
//        // new_order.ticker_id = new_order.ticker_id + 1;
//        // new_order.client_id = new_order.client_id + 1;
//     }
// }

// fn nano_now() -> u64 {
//     match SystemTime::now().duration_since(UNIX_EPOCH) {
//         Ok(n) => n.as_nanos() as u64,
//         Err(_) => 0,
//     }
// }

//  fn sample_orders() -> Vec<NewOrder> {
//     let orders = vec![
//     // =========================
//     // Ticker ID 0
//     // =========================

//     // Sell 100 @ 23600
//     NewOrder::new(
//         23600,
//         100,
//         nano_now(),
//         1,
//         0,
//         0,
//         OrderSide::Sell,
//         OrderType::LIMIT,
//     ),

//     // Buy 100 @ 23700 -> FULL MATCH
//     NewOrder::new(
//         23700,
//         100,
//         nano_now(),
//         2,
//         0,
//         0,
//         OrderSide::Buy,
//         OrderType::LIMIT,
//     ),


//     // Sell 50 @ 23600
//     NewOrder::new(
//         23600,
//         50,
//         nano_now(),
//         3,
//         0,
//         0,
//         OrderSide::Sell,
//         OrderType::LIMIT,
//     ),

//     // Buy 100 @ 23600 -> PARTIAL MATCH (50 filled, 50 remaining)
//     NewOrder::new(
//         23600,
//         100,
//         nano_now(),
//         4,
//         0,
//         0,
//         OrderSide::Buy,
//         OrderType::LIMIT,
//     ),


//     // =========================
//     // Ticker ID 1
//     // =========================

//     // Sell 200 @ 50000
//     NewOrder::new(
//         50000,
//         200,
//         nano_now(),
//         5,
//         1,
//         0,
//         OrderSide::Sell,
//         OrderType::LIMIT,
//     ),

//     // Buy 200 @ 50100 -> FULL MATCH
//     NewOrder::new(
//         50100,
//         200,
//         nano_now(),
//         6,
//         1,
//         0,
//         OrderSide::Buy,
//         OrderType::LIMIT,
//     ),


//     // Sell 300 @ 50000
//     NewOrder::new(
//         50000,
//         300,
//         nano_now(),
//         7,
//         1,
//         0,
//         OrderSide::Sell,
//         OrderType::LIMIT,
//     ),

//     // Buy 100 @ 50000 -> PARTIAL MATCH (sell leaves 200)
//     NewOrder::new(
//         50000,
//         100,
//         nano_now(),
//         8,
//         1,
//         0,
//         OrderSide::Buy,
//         OrderType::LIMIT,
//     ),
// ];

// orders
// }