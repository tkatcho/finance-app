use std::net::TcpListener;
use std::thread;

use finance_app::websockets::WebSocket;

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
            Ok(_) => {}
            Err(e) => {
                println!("Error reading frame: {}", e);
                break;
            }
        }
    }
}
