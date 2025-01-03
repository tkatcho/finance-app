use finance_app::{
    tcp::socket::RawSocket,
    websockets::{OpCode, WebSocket},
};
use socket2::Socket;
use std::io::{self};

fn main() -> io::Result<()> {
    println!("Creating server socket...");
    let mut server = RawSocket::new()?;
    server.bind("127.0.0.1", 8080)?;

    println!("Server listening on 127.0.0.1:8080");
    println!("Try connecting with: netcat 127.0.0.1 8080 or telnet 127.0.0.1 8080");

    let (client_socket, addr) = loop {
        match server.accept() {
            Ok((socket, addr)) => {
                println!("Accepted connection from {}", addr);
                break (socket, addr); // Return the client socket
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("Accept error: {}", e);
                continue;
            }
        }
    };

    println!("Server Ready");
    let peer_addr = match addr.ip() {
        std::net::IpAddr::V4(ipv4) => ipv4.octets(),
        _ => [0; 4],
    };
    let peer_port = addr.port();

    handle_connection(client_socket, peer_addr, peer_port);

    Ok(())
}
fn handle_connection(socket: Socket, peer_addr: [u8; 4], peer_port: u16) {
    println!("Handling connection");
    let sock = match RawSocket::new_with_params(socket, peer_addr, peer_port) {
        Ok(sock) => sock,
        Err(e) => {
            eprintln!("Failed to create RawSocket: {}", e);
            return;
        }
    };

    // Create WebSocket connection
    let mut ws = match WebSocket::accept(sock, peer_addr, peer_port) {
        Ok(ws) => ws,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    ws.send("Hello from the server!".as_bytes().to_vec())
        .expect("Failed to send message");

    loop {
        match ws.read_frame() {
            Ok(frame) => {
                println!("Received frame: {:?}", frame);
                match frame.op_code {
                    OpCode::Text => {
                        let message = String::from_utf8(frame.payload.clone()).unwrap();
                        println!("Received message: {}", message);
                        ws.send(frame.payload).expect("Failed to send message");
                    }
                    OpCode::ConnectionClosed => {
                        println!("Connection closed");
                        break;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Failed to read frame: {}", e);
                break;
            }
        }
    }
}
