use finance_app::websockets::{Frame, OpCode, WebSocket};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

enum WebSocketEvent {
    Message(Vec<u8>),
    Close,
    Error(String),
}

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

    

}
