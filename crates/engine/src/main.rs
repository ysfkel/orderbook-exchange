mod config;
mod engine;
mod error;
mod network;
mod parser;
mod types;

use network::listener;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{Level, error, event, info};
pub use types::*;

pub fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(false)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    event!(Level::INFO, "Starting the engine...");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    ctrlc::set_handler(move || {
        info!("shutdown signal received");
        shutdown_clone.store(true, Ordering::Relaxed);
    })
    .expect("Failed to set Ctrl-C handler");

    let buf_size = 1024; //  size of  receive buffer
    let max_msg_size = 512; // maximum allowed payload from a single packet
    let reconnect_delay_secs = 5;

    loop {
        // Check shutdown before each attempt so Ctrl-C during a sleep is respected
        if shutdown.load(Ordering::Relaxed) {
            info!("shutting down");
            break;
        }

        match listener::listen(
            Ipv4Addr::UNSPECIFIED,
            Ipv4Addr::new(239, 1, 1, 1),
            9001,
            buf_size,
            max_msg_size,
            Arc::clone(&shutdown),
        ) {
            Ok(()) => {
                // run_listener exited cleanly (shutdown flag was set) — we're done
                break;
            }
            Err(e) => {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                error!(
                   error = %e,
                   delay_secs = reconnect_delay_secs,
                   "Listener failed - reconnecting"
                )
            }
        }
    }
}
