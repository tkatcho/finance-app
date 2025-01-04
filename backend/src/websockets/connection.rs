use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::TcpStream;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sha1::{Digest, Sha1};

use super::Frame;
use super::Request;
use crate::websockets::{
    OpCode, OpCode::Binary, OpCode::ConnectionClosed, OpCode::Continuation, OpCode::Ping,
    OpCode::Pong, OpCode::Text,
};

#[derive(Debug)]
pub struct WebSocket {
    stream: TcpStream,
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
//fix this to put the handshakes in the handshake.rs
impl WebSocket {
    pub fn new(stream: TcpStream) -> Self {
        WebSocket {
            stream,
            state: ConnectionState::Connecting,
        }
    }

    pub fn accept(stream: TcpStream) -> Result<Self, Error> {
        let mut ws = WebSocket::new(stream);

        let request = ws.read_handshake_request()?;

        let client_key = request.get_header("Sec-WebSocket-Key").ok_or(Error::new(
            std::io::ErrorKind::Other,
            "Not a WebSocket upgrade request".to_owned(),
        ))?;

        // // Generate accept key using handshake.rs
        let accept_key = generate_accept_key(client_key);

        // Send back handshake response
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

        self.stream.write_all(response.as_bytes())?;

        Ok(())
    }

    pub fn send_pong(&mut self, payload: Vec<u8>) -> Result<(), Error> {
        let frame = Frame::new(OpCode::Pong, payload);
        self.write_all(&frame.to_bytes())
    }
    pub fn send_ping(&mut self, payload: Vec<u8>) -> Result<(), Error> {
        let frame = Frame::new(OpCode::Ping, payload);
        self.write_all(&frame.to_bytes())
    }

    pub fn read_handshake_request(&mut self) -> Result<Request, Error> {
        let mut buffer = Vec::new();
        let mut headers = Vec::new();
        let mut reader = BufReader::new(&mut self.stream);

        // Read headers line by line until we hit an empty line
        loop {
            let mut line = String::new();
            reader.read_line(&mut line);

            // HTTP headers end with \r\n\r\n, so an empty line (just \r\n) marks the end
            if line == "\r\n" || line == "\n" {
                break;
            }

            buffer.extend(line.as_bytes());
            headers.push(line.trim().to_string());
        }

        // Parse into a Request struct
        let request = Request {
            headers,
            raw: buffer,
        };

        // Validate it's a valid WebSocket upgrade request
        if !request
            .headers
            .iter()
            .any(|h| h.contains("Upgrade: websocket"))
        {
            return Err(Error::new(
                std::io::ErrorKind::Other,
                "Not a WebSocket upgrade request".to_owned(),
            ));
        }

        Ok(request)
    }

    pub fn read_exact(&mut self, num: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = vec![0; num]; // Create a buffer of specified size

        self.stream.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut written = 0;
        while written < buf.len() {
            match self.stream.write(&buf[written..]) {
                Ok(n) => {
                    written += n;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    pub fn read_frame(&mut self) -> Result<Frame, std::io::Error> {
        // Read first 2 bytes (header)
        let header = self.read_exact(2)?;

        // Parse header using Frame::parse
        let frame = Frame::parse(
            (header[0] & 0x80) != 0, // fin
            match header[0] & 0x0F {
                // opcode
                0x0 => Continuation,
                0x1 => Text,
                0x2 => Binary,
                0x8 => ConnectionClosed,
                0x9 => Ping,
                0xA => Pong,
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid opcode",
                    ))
                }
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

    pub fn send(&mut self, payload: Vec<u8>) -> Result<(), std::io::Error> {
        let frame = Frame::new(OpCode::Text, payload);
        self.write_all(&frame.to_bytes())
    }
}
pub fn generate_accept_key(client_key: &str) -> String {
    /*
    Additionally, the server can decide on extension/subprotocol requests here;
     see Miscellaneous for details.
     The Sec-WebSocket-Accept header is important in that the server must derive
     it from the Sec-WebSocket-Key that the client sent to it. To get it, concatenate
     the client's Sec-WebSocket-Key and the string "258EAFA5-E914-47DA-95CA-C5AB0DC85B11" t
     ogether (it's a "magic string"),
     take the SHA-1 hash of the result, and return the base64 encoding of that hash. */
    let magic_string = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let to_hash = format!("{}{}", client_key, magic_string);

    let mut hasher = Sha1::new();
    hasher.update(to_hash.as_bytes());
    let result = hasher.finalize();

    BASE64_STANDARD.encode(result)
}
