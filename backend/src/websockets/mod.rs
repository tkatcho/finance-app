mod connection;
// mod constants;
mod frame;
// mod handshake;
pub mod request;

use std::io::Error;

pub use connection::WebSocket;
pub use frame::{Frame, OpCode};
pub use request::Request;

// Re-export main types
pub type Result<T> = std::result::Result<T, Error>;
