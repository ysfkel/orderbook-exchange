use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crossbeam::queue::ArrayQueue;
use common::{
    thread_handle::ThreadHandle,
    traits::ThreadHandler,
    types::{NewOrder, OrderSide, OrderType},
};
use crate::types::OrderRequest;

/// Stands in for REST ingress during local testing — pushes a fixed batch
/// of sample orders onto the inbound queue, then exits. Runs on its own
/// thread purely so it can be started/stopped like the other services
/// (and so main isn't blocked seeding orders before the event loop starts).
pub struct MockOrderFeed {
    queue: Arc<ArrayQueue<OrderRequest>>,
    run: Arc<AtomicBool>,
}

impl MockOrderFeed {
    pub fn new(queue: Arc<ArrayQueue<OrderRequest>>) -> Self {
        Self {
            queue,
            run: Arc::new(AtomicBool::new(false)),
        }
    }

    fn run_loop(self) {
        for order in sample_orders() {
            // stop() called before we finished seeding — bail early
            if !self.run.load(Ordering::Acquire) {
                break;
            }

            let req = OrderRequest::NewOrder(order);
            if let Err(_returned) = self.queue.push(req) {
                tracing::error!("mock feed: queue full, dropping sample order");
            }

            // optional: space pushes out instead of dumping the whole
            // batch in one instant — remove if you want a pure burst test
            std::thread::sleep(Duration::from_millis(2000));
        }
        tracing::info!("mock order feed exiting");
    }
}

impl ThreadHandler for MockOrderFeed {
    fn start(self) -> ThreadHandle {
        self.run.store(true, Ordering::Release);
        let run = self.run.clone();
        let thread = std::thread::Builder::new()
            .name("mock-order-feed".into())
            .spawn(move || self.run_loop())
            .expect("failed to spawn mock-order-feed");
        ThreadHandle {
            run,
            thread: Some(thread),
        }
    }
}

fn sample_orders() -> Vec<NewOrder> {
    vec![
        NewOrder::new(23600, 100, nano_now(), 1, 0, 0, OrderSide::Sell, OrderType::LIMIT),
        NewOrder::new(23600, 100, nano_now(), 2, 0, 0, OrderSide::Buy, OrderType::LIMIT),
        NewOrder::new(23600, 50, nano_now(), 3, 0, 0, OrderSide::Sell, OrderType::LIMIT),
        NewOrder::new(23600, 100, nano_now(), 4, 0, 0, OrderSide::Buy, OrderType::LIMIT),
        NewOrder::new(50000, 200, nano_now(), 5, 1, 0, OrderSide::Sell, OrderType::LIMIT),
        NewOrder::new(50100, 200, nano_now(), 6, 1, 0, OrderSide::Buy, OrderType::LIMIT),
        NewOrder::new(50000, 300, nano_now(), 7, 1, 0, OrderSide::Sell, OrderType::LIMIT),
        NewOrder::new(50000, 100, nano_now(), 8, 1, 0, OrderSide::Buy, OrderType::LIMIT),
    ]
}

fn nano_now() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_nanos() as u64,
        Err(_) => 0,
    }
}