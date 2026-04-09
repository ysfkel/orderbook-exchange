use std::net::{AddrParseError, Ipv4Addr};

use crate::error::ProgramError;

// PARSE NETWORK INTERFACE
//
//  Converts CLI input into an IPv4 address.
// pub fn parse_interface(s: &str) -> Result<Ipv4Addr, ProgramError> {
//     match s {
//         "loopback" => Ok(Ipv4Addr::new(127, 0, 0, 1)),
//         "any" => Ok(Ipv4Addr::UNSPECIFIED), // 0.0.0.0
//         ip => ip.parse().map_err(|e: AddrParseError| {
//             ProgramError::InvalidInterface(s.to_string(), e.to_string())
//         }),
//     }
// }

// /// Pase Multicast Group
// ///
// /// Ensures the address is multicast
// pub fn parse_multicast_group(s: &str) -> Result<Ipv4Addr, ProgramError> {
//     let addr: Ipv4Addr = s.parse().map_err(|e: AddrParseError| {
//         ProgramError::InvalidMulticastGroup(s.to_string(), e.to_string())
//     })?;

//     if !addr.is_multicast() {
//         return Err(ProgramError::InvalidMulticastGroup(
//             s.to_string(),
//             "address is not multicast".into(),
//         ));
//     }

//     Ok(addr)
// }
