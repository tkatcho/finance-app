use std::{io::Read, net::TcpStream};

use crate::websockets::OpCode;

use super::Frame;

#[derive(Debug)]
pub struct WebSocket {
    stream: TcpStream,
    is_client: bool,
    state: ConnectionState,
}

#[derive(Debug)]
enum ConnectionState {
    Connecting,
    Connected,
    Closing,
    Closed,
    Bitchass,
}

impl WebSocket {
    pub fn new(stream: TcpStream, is_client: bool) -> Self {
        WebSocket {
            stream,
            is_client,
            state: ConnectionState::Connecting,
        }
    }

    pub fn accept(stream: TcpStream) -> Result<Self, std::io::Error> {
        let mut ws = WebSocket::new(stream, false);

        let request = ws.read_handshake_request();

        // let client_key = request
        //     .get_header("Sec-WebSocket-Key")
        //     .ok_or(WebSocketError::HandshakeFailed("No key provided".into()))?;

        // // Generate accept key using handshake.rs
        // let accept_key = handshake::generate_accept_key(client_key);

        // // Send back handshake response
        // ws.write_handshake_response(&accept_key)?;

        ws.state = ConnectionState::Connected;
        Ok(ws)
    }

    pub fn read_handshake_request(&self) {
        println!("reading hand shake");
    }

    pub fn read_exact(&mut self, num: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = vec![0; num]; // Create a buffer of specified size

        self.stream.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    pub fn write_frame(&mut self, frame: Frame) {}

    pub fn read_frame(&mut self) -> Result<Frame, std::io::Error> {
        Frame::parse(false, OpCode::ConnectionClosed, false, 15)
    }
}
