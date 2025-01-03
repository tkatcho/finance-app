mod checksum;
mod connection;
mod headers;
pub mod socket;
mod tests;

use std::io::Error;

// Re-export main components
pub use headers::{IpHeader, TcpFlags, TcpHeader};

// Constants for TCP protocol
pub const TCP_HEADER_SIZE: usize = 20;
pub const IP_HEADER_SIZE: usize = 20;
pub const MAX_SEGMENT_SIZE: usize = 1460; // Standard MSS for Ethernet

pub type TcpResult<T> = Result<T, Error>;
