use finance_app::websockets::{OpCode, WebSocket};
use std::net::TcpListener;
use std::thread;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("WebSocket server listening on port 8080");

    // Create a channel for each new connection
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {:?}", stream);
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("Failed to establish a connection: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_connection(stream: std::net::TcpStream) {
    println!("Handling connection: {:?}", stream);

    // Create WebSocket connection
    let mut ws = match WebSocket::accept(stream) {
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
