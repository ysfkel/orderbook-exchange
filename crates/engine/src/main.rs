mod config;
mod engine;
mod error;
mod network;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;
pub fn main() {
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

    network::inbound::run(shutdown);
}
