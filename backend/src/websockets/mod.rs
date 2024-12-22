mod connection;
mod error;
// mod constants;
 mod frame;
// mod handshake;

pub use connection::WebSocket;
pub use error::WebSocketError;
pub use frame::{Frame, OpCode};

// Re-export main types
pub type Result<T> = std::result::Result<T, WebSocketError>;