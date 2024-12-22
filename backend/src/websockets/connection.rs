use std::net::TcpStream;

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
        let ws = WebSocket::new(stream, false);

        ws.read_handshake_request();
        Ok(ws)
    }

    pub fn read_handshake_request(&self) {
        println!("{:?} heyyy", self);
    }

    pub fn read_frame(&mut self) -> Result<Frame, std::io::Error> {
        println!("{:?}", self);

        unimplemented!()
    }
}
