use common::queue::{QueueConsumer, RingBufferConsumer};
use common::thread_handle::ThreadHandle;
use common::traits::ThreadHandler;
use common::types::{AcceptedOrder, ME_MAX_TICKERS, TickerId};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::engine::orderbook::OrderBook;
use crate::types::EngineRequest;

pub struct MatchingEngine {
    /// One order book per instrument (the book's OrderBookHashMap, an
    /// std::array<MEOrderBook*, ME_MAX_TICKERS>).
    ticker_order_book: Vec<OrderBook>,
    incoming_requests: RingBufferConsumer<EngineRequest>,
    run: Arc<AtomicBool>,
}

impl MatchingEngine {
    pub fn new(incoming_requests: RingBufferConsumer<EngineRequest>) -> Self {
        let ticker_order_book = (0..ME_MAX_TICKERS as TickerId)
            .map(|ticker_id| OrderBook::new(ticker_id))
            .collect();

        Self {
            ticker_order_book,
            incoming_requests,
            run: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn process_order(&mut self, req: EngineRequest) {
        match req {
            EngineRequest::NewOrder(AcceptedOrder {
                price,
                quantity,
                timestamp: _,
                client_order_id,
                market_order_id: _,
                client_id,
                ticker_id,
                side,
                order_type: _,
                ..
            }) => {
                let book = &mut self.ticker_order_book[ticker_id as usize];
                book.add(client_id, client_order_id, ticker_id, side, price, quantity);
            }
        }
    }

    pub fn run_loop(&mut self) {
        while self.run.load(Ordering::Acquire) {
            match self.incoming_requests.pop() {
                Ok(req) => self.process_order(req),
                Err(_) => std::hint::spin_loop(),
            }
        }
        // stop() called. Precondition (enforced by main's stop order):
        // the listener/producer is already stopped, so this terminates.
        while let Ok(req) = self.incoming_requests.pop() {
            self.process_order(req);
        }
    }
}

impl ThreadHandler for MatchingEngine {
    fn start(mut self) -> ThreadHandle {
        self.run.store(true, Ordering::Release);
        let run = self.run.clone();
        let thread = std::thread::Builder::new()
            .name("matching-engine".into())
            .spawn(move || self.run_loop())
            .expect("failed to spawn matching-engine");
        ThreadHandle {
            run,
            thread: Some(thread),
        }
    }
}
