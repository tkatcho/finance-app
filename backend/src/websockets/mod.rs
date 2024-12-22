mod connection;
mod error;
// mod constants;
mod frame;
// mod handshake;
pub mod request;

pub use connection::WebSocket;
pub use error::WebSocketError;
pub use frame::{Frame, OpCode};
pub use request::Request;

// Re-export main types
pub type Result<T> = std::result::Result<T, WebSocketError>;
