// ═══════════════════════════════════════════════════════════════════════════
// src/config.rs — CLI Configuration
//
// This file owns everything that comes from the outside world at startup:
// the command-line flags. If you later add environment variable support or
// a config file, this is where that logic would live too.
//
// The `Args` struct is the single source of truth for all runtime settings.
// Every other module receives what it needs via function parameters — nothing
// reads Args directly except main.rs. This keeps modules independently testable.
// ═══════════════════════════════════════════════════════════════════════════

use clap::{Parser, arg};
// `Parser` is a trait from clap. Deriving it on a struct auto-generates all
// the CLI parsing code — reading std::env::args(), type-checking, --help output, etc.

/// All command-line flags accepted by the program.
///
/// Run `cargo run -- --help` to see this as formatted help text.
#[derive(Parser, Debug)]
#[command(
    name = "multicast-listener",
    about = "Production multicast UDP listener"
)]
pub struct Args {
    /// Which network interface to listen on.
    /// "any"      = 0.0.0.0, joins on whatever interface the OS picks (needs real network)
    /// "loopback" = 127.0.0.1, stays entirely on this machine (works offline)
    /// "x.x.x.x" = a specific NIC's IP address like "192.168.1.5"
    #[arg(long, default_value = "any")]
    pub interface: String,

    /// The multicast group IP address to subscribe to.
    /// Must be in the 224.0.0.0 – 239.255.255.255 range.
    /// Both sender and receiver must use the same group to talk to each other.
    #[arg(long, default_value = "239.1.1.1")]
    pub multicast_group: String,

    /// UDP port to listen on. The sender must send to the same port.
    #[arg(long, default_value_t = 9000)]
    pub port: u16,

    /// Size of the receive buffer in bytes.
    /// This is the maximum number of bytes we ask the OS to give us per recv_from call.
    /// Should be at least as large as your biggest expected UDP packet.
    #[arg(long, default_value_t = 4096)]
    pub buf_size: usize,

    /// Any message larger than this (in bytes) is immediately dropped.
    /// Protects against malformed or malicious packets consuming too much memory.
    #[arg(long, default_value_t = 1024)]
    pub max_msg_size: usize,

    /// How many seconds to wait before trying to reconnect after a socket error.
    /// e.g. if the network cable is unplugged, we wait this long before retrying.
    #[arg(long, default_value_t = 5)]
    pub reconnect_delay_secs: u64,

    /// Port for the Prometheus metrics HTTP server.
    /// After starting, visit http://localhost:<metrics_port>/metrics to see live stats.
    #[arg(long, default_value_t = 9001)]
    pub metrics_port: u16,
}
