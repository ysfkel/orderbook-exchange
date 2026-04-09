use socket2::{Domain, Protocol, Socket, Type};
use tracing::{info, instrument};

use std::net::{Ipv4Addr, SocketAddr};

use crate::error::ProgramError;

#[instrument(name = "create_socket", skip_all, fields(interface = %interface, multicast_group = %multicast_group, port))]
pub fn create_socket(
    interface: Ipv4Addr,
    multicast_group: Ipv4Addr,
    port: u16,
) -> Result<Socket, ProgramError> {
    // Create a new IPv4 UDP socket.
    // Type::DGRAM = UDP (Datagram — connectionless, no handshake, packets may arrive out of order)
    // `None` for protocol = let the OS pick the default (correct for UDP)
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    // SO_REUSEADDR: tells the OS it's OK to bind to a port in TIME_WAIT state.
    // TIME_WAIT happens for ~60 seconds after a socket closes. Without this option,
    // restarting the program quickly would fail with "address already in use".
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;

    // Set a 500ms read timeout on the socket.
    // Without a timeout, recv_from blocks forever waiting for the next packet.
    // With a 500ms timeout, recv_from periodically returns TimedOut, which lets
    // the receive loop in listener.rs check the shutdown flag and exit on Ctrl-C.
    // "WouldBlock" and "TimedOut" are both used by different OSes for the same thing.
    // socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    // Bind to 0.0.0.0 (all interfaces) rather than the specific interface IP.
    // Why not bind to the interface IP directly?
    // On Windows, binding to a specific IP can silently prevent multicast packets
    // from being delivered even after a successful join. The safer cross-platform
    // pattern is: bind to 0.0.0.0, control the interface via join_multicast_v4.

    // socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)).into())?;
    socket.bind(&SocketAddr::from((interface, port)).into())?;

    // IP_MULTICAST_LOOP: allows a sender and receiver running on the *same machine*
    // to communicate through multicast. Without this, packets you send loop back
    // silently — your own receiver never sees them. Essential for local dev/testing.
    socket.set_multicast_loop_v4(true)?;

    // The actual multicast group join (IP_ADD_MEMBERSHIP at the OS level).
    // This tells the OS: "deliver UDP packets addressed to `multicast_group`
    // that arrive on `interface` to this socket."
    // Without this call the socket would never see any multicast traffic —
    // multicast is opt-in, unlike broadcast.
    socket.join_multicast_v4(&multicast_group, &interface)?;

    info!(
      interface = %interface,
      multicast_group = %multicast_group,
      port,
        "Socket created and joined multicast group"
    );

    Ok(socket)
}
