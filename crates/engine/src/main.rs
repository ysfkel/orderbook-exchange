mod config;
mod engine;
mod error;
mod network;
mod types;

use common::queue::RingBufferQueue;
use common::traits::ThreadHandler;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tracing::info;
use types::*;

use crate::engine::matching_engine::MatchingEngine;
use crate::network::Listener;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(false)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    ctrlc::set_handler(move || {
        info!("shutdown signal received");
        shutdown_clone.store(true, Ordering::Relaxed);
    })
    .expect("Failed to set Ctrl-C handler");

    let RingBufferQueue {
        producer: order_producer,
        consumer: order_consumer,
    }: RingBufferQueue<EngineRequest> = RingBufferQueue::new(4096);

    // Start consumer first, then producer — so nothing is ever pushed
    // into a ring nobody is draining.
    let listener = Listener::new(order_producer).start();
    let engine = MatchingEngine::new(order_consumer).start();

    // Park until Ctrl-C.
    while !shutdown.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }

    // Ordered teardown: silence the producer, then drain & stop the consumer.
    engine.stop();
    listener.stop();
    info!("engine stopped");
    Ok(())
}
