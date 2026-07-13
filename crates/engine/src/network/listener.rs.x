// ═══════════════════════════════════════════════════════════════════════════
// src/listener.rs — The Receive Loop
//
// This file is the orchestrator — it wires everything else together:
//   - Calls socket.rs to get a ready-to-use socket
//   - Sits in a loop calling recv_from to read UDP packets
//   - Passes each packet to handler.rs for processing
//   - Tracks metrics (messages received, errors)
//   - Checks the shutdown flag (set by Ctrl-C in main.rs) every 500ms
//   - Returns Err if the socket dies, so main.rs can reconnect
//
// This file deliberately contains no business logic — it only moves data
// from the network to handler.rs and manages the loop's lifecycle.
// ═══════════════════════════════════════════════════════════════════════════

// AtomicBool — a boolean flag that is safe to read/write across threads without a mutex
// Ordering   — controls memory ordering guarantees; Relaxed is fine for a simple flag
// Arc        — "Atomic Reference Count", allows shared ownership of the AtomicBool
//              between this loop and the Ctrl-C handler registered in main.rs
use std::{
    mem::MaybeUninit,
    net::Ipv4Addr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

// counter! — a metric that only ever increases (total messages received, total errors)
//             good for "how many X happened since startup"
// gauge!   — a metric that represents a current value (can go up or down)
//             good for dashboards showing "messages received right now"
use crate::{
    error::ProgramError,
    network::{handle_message, socket::create_socket},
};

/// Runs the main receive loop. This function blocks until one of two things happens:
///
///   (a) The `shutdown` flag is set to true (Ctrl-C pressed)
///       → drains cleanly and returns Ok(())
///
///   (b) A socket-level error occurs (e.g. network cable unplugged)
///       → returns Err(ListenerError::Socket(...))
///       → main.rs catches this and calls run_listener() again after a delay
///
/// `args`     — all CLI configuration (buffer size, port, multicast group, etc.)
/// `shutdown` — shared boolean; when true, the loop exits after the next recv timeout
pub fn listen(
    interface: Ipv4Addr,
    multicast_group: Ipv4Addr,
    port: u16,
    buf_size: usize,
    max_msg_size: usize,
    shutdown: Arc<AtomicBool>,
) -> Result<(), ProgramError> {
    // Parse and validate CLI args into typed values.
    // Done fresh on every call so each reconnect attempt re-reads the config.
    // let interface = parse_interface(&args.interface)?;
    // let multicast_group = parse_multicast_group(&args.multicast_group)?;

    // Create a brand new socket for this attempt.
    // If the previous socket died (Err returned below), this creates a clean replacement.
    let socket = create_socket(interface, multicast_group, port)?;

    // Pre-allocate the receive buffer once, outside the loop.
    // `vec![0u8; n]` creates a Vec filled with n zero bytes.
    // Reusing the same buffer every iteration avoids allocating + freeing memory
    // on every packet — important at high message rates (thousands/sec).
    // The buffer is zeroed (0u8) so we avoid needing unsafe MaybeUninit tricks.
    let mut buf = vec![MaybeUninit::uninit(); buf_size];

    // Local counters — incremented each iteration and also pushed to Prometheus
    let mut messages_received: u64 = 0;
    let mut error_count: u64 = 0;

    tracing::info!("Entering receive loop");

    // ── Main receive loop ─────────────────────────────────────────────────
    // Keeps running until shutdown is set to true.
    // `shutdown.load(Ordering::Relaxed)` reads the AtomicBool safely.
    // Ordering::Relaxed is sufficient here — we just need "did someone ask us to stop",
    // not strict happens-before guarantees.
    while !shutdown.load(Ordering::Relaxed) {
        let (size, sock_addr) = match socket.recv_from(&mut buf[..]) {
            Ok((size, sock_addr)) => (size, sock_addr),

            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                continue;
            }

            Err(e) => {
                return Err(ProgramError::Socket(e));
            }
        };

        let socket_addr = sock_addr
            .as_socket()
            .unwrap_or_else(|| "0.0.0.0:0".parse().unwrap());

        let bytes: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, size) };

        match handle_message(bytes, max_msg_size) {
            Ok(()) => {
                messages_received += 1;
            }

            Err(ProgramError::MessageTooLarge { size, max }) => {
                error_count += 1;
                tracing::warn!(size, max, src = %socket_addr, "Dropping message. Too Large");
            }

            Err(ProgramError::DeserializeMessageHeader(e)) => {
                error_count += 1;
                tracing::warn!(error = %e, src = %socket_addr, "Failed to Deserialize Message Header");
            }

            Err(ProgramError::DeserializeMessageBody(e)) => {
                error_count += 1;
                tracing::warn!(error = %e, src = %socket_addr, "Failed to Deserialize Message Body");
            }

            // ───────────────────
            // Shouldn't happen in normal operation — log it loudly
            Err(e) => {
                error_count += 1;
                tracing::error!(error = %e, "Unexpected error")
            }
        }
    }

    // We only reach this line after shutdown=true — log final stats for the run
    tracing::info!(
        messages_received,
        error_count,
        "Listener shutting down cleanly"
    );
    Ok(())
}
