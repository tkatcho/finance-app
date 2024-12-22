use std::net::TcpListener;
use std::thread;

use finance_app::websockets::{Frame, OpCode, WebSocket};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("WebSocket server listening on port 8080");

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
    let mut ws = match WebSocket::accept(stream) {
        Ok(ws) => ws,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    //loop

    loop {
        match ws.read_frame() {
            Ok(frame) => {
                match frame.op_code {
                    OpCode::Text => {
                        // Echo the message back
                        let message = String::from_utf8_lossy(&frame.payload);
                        println!("Received: {}", message);

                        // Send it back
                        ws.write_frame(Frame::new(OpCode::Text, message.as_bytes().to_vec()));
                    }
                    OpCode::ConnectionClosed => break,
                    _ => { /* Handle other frame types */ }
                }
            }
            Err(e) => {
                println!("Error reading frame: {}", e);
                break;
            }
        }
    }
}
