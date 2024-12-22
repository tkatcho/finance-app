use std::net::TcpStream;

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
            stream: stream,
            is_client: is_client,
            state: ConnectionState::Connecting,
        }
    }

    pub fn accept(stream: TcpStream) -> Result<Self, std::io::Error> {
        let ws = WebSocket::new(stream, false);
        Ok(ws)
    }
}
