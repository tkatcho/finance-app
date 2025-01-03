use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sha1::{Digest, Sha1};
use std::io::{Error, ErrorKind};

use super::Frame;
use super::Request;
use crate::tcp::socket::RawSocket;
use crate::websockets::{
    OpCode, OpCode::Binary, OpCode::ConnectionClosed, OpCode::Continuation, OpCode::Ping,
    OpCode::Pong, OpCode::Text,
};

pub struct WebSocket {
    socket: RawSocket,
    state: ConnectionState,
    peer_addr: [u8; 4],
    peer_port: u16,
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
    pub fn new(socket: RawSocket, peer_addr: [u8; 4], peer_port: u16) -> Self {
        WebSocket {
            socket,
            state: ConnectionState::Connecting,
            peer_addr,
            peer_port,
        }
    }

    pub fn get_socket(self) -> RawSocket {
        return self.socket;
    }
    pub fn accept(socket:RawSocket , peer_addr: [u8; 4], peer_port: u16) -> Result<Self, Error> {
        let mut ws = WebSocket::new(socket, peer_addr, peer_port);

        let request = ws.read_handshake_request()?;

        let client_key = request.get_header("Sec-WebSocket-Key").ok_or(Error::new(
            ErrorKind::Other,
            "Not a WebSocket upgrade request".to_owned(),
        ))?;

        let accept_key = generate_accept_key(client_key);
        ws.write_handshake_response(&accept_key)?;

        ws.state = ConnectionState::Connected;
        Ok(ws)
    }

    pub fn write_handshake_response(&mut self, accept_key: &str) -> Result<(), Error> {
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
            Connection: Upgrade\r\n\
            Upgrade: websocket\r\n\
            Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );

        self.socket.send_packet(
            self.peer_addr,
            self.peer_port,
            true,
            false,
            false, // SYN flag
            response.as_bytes(),
        )?;

        Ok(())
    }

    pub fn read_handshake_request(&mut self) -> Result<Request, Error> {
        let mut headers = Vec::new();
        let mut buffer = Vec::new();

        // Read incoming packet
        let (data, _, _) = self.socket.receive_packet()?;

        // Convert bytes to string and split by lines
        let request_str = String::from_utf8(data).map_err(|_| {
            Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in handshake request")
        })?;

        for line in request_str.lines() {
            if line.is_empty() {
                break;
            }
            headers.push(line.to_string());
            buffer.extend(line.as_bytes());
            buffer.extend(b"\r\n");
        }

        // Parse into Request struct
        let request = Request {
            headers: headers.clone(),
            raw: buffer,
        };

        // Validate WebSocket upgrade request
        if !headers.iter().any(|h| h.contains("Upgrade: websocket")) {
            return Err(Error::new(
                ErrorKind::Other,
                "Not a WebSocket upgrade request".to_owned(),
            ));
        }

        Ok(request)
    }

    pub fn read_exact(&mut self, num: usize) -> Result<Vec<u8>, Error> {
        let mut remaining = num;
        let mut buffer = Vec::with_capacity(num);

        while remaining > 0 {
            let (data, _, _) = self.socket.receive_packet()?;
            if data.is_empty() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Connection closed"));
            }

            let to_take = remaining.min(data.len());
            buffer.extend_from_slice(&data[..to_take]);
            remaining -= to_take;
        }

        Ok(buffer)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        self.socket.send_packet(
            self.peer_addr,
            self.peer_port,
            true, // ACK flag
            false,
            false,
            buf,
        )?;
        Ok(())
    }

    pub fn read_frame(&mut self) -> Result<Frame, Error> {
        // Read first 2 bytes (header)
        let header = self.read_exact(2)?;

        // Parse header
        let frame = Frame::parse(
            (header[0] & 0x80) != 0, // fin
            match header[0] & 0x0F {
                0x0 => Continuation,
                0x1 => Text,
                0x2 => Binary,
                0x8 => ConnectionClosed,
                0x9 => Ping,
                0xA => Pong,
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid opcode")),
            },
            (header[1] & 0x80) != 0,   // mask
            (header[1] & 0x7F) as u64, // payload_len
        )?;

        // Handle extended payload lengths
        let actual_payload_len = if frame.payload_len == 126 {
            let len_bytes = self.read_exact(2)?;
            u16::from_be_bytes([len_bytes[0], len_bytes[1]]) as u64
        } else if frame.payload_len == 127 {
            let len_bytes = self.read_exact(8)?;
            u64::from_be_bytes(len_bytes.try_into().unwrap())
        } else {
            frame.payload_len
        };

        // Read mask key if needed
        let mask_key: Option<[u8; 4]> = if frame.mask {
            let mask_bytes = self.read_exact(4)?;
            Some(mask_bytes.try_into().unwrap())
        } else {
            None
        };

        // Read and unmask payload
        let mut payload = self.read_exact(actual_payload_len as usize)?;
        if let Some(mask_key) = mask_key {
            for i in 0..payload.len() {
                payload[i] ^= mask_key[i % 4];
            }
        }

        Ok(Frame {
            fin: frame.fin,
            op_code: frame.op_code,
            mask: frame.mask,
            payload_len: actual_payload_len,
            mask_key,
            payload,
        })
    }

    pub fn send(&mut self, payload: Vec<u8>) -> Result<(), Error> {
        let frame = Frame::new(OpCode::Text, payload);
        self.write_all(&frame.to_bytes())
    }
}

// This function can stay the same since it's not socket-related
pub fn generate_accept_key(client_key: &str) -> String {
    let magic_string = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let to_hash = format!("{}{}", client_key, magic_string);

    let mut hasher = Sha1::new();
    hasher.update(to_hash.as_bytes());
    let result = hasher.finalize();

    BASE64_STANDARD.encode(result)
}
