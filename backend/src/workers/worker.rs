use std::{
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::websockets::{OpCode, WebSocket};
pub enum Message {
    NewConnection(TcpStream),
    Terminate,
}
pub enum Workers {
    Listener,
    Fetcher,
    Worker,
}
pub struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}
impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            match message {
                Message::NewConnection(stream) => {
                    println!("Worker {} handling connection", id);
                    handle_connection(stream);
                }
                Message::Terminate => {
                    println!("Worker {} terminating", id);
                    break;
                }
            }
        });

        Worker { id, thread }
    }
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

    let ping_interval = Duration::from_secs(30); // Send ping every 30 seconds
    let mut last_ping = Instant::now();

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
                    OpCode::Ping => {
                        println!("Received ping");
                        ws.send_pong(frame.payload).expect("Failed to send pong");
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
        if last_ping.elapsed() >= ping_interval {
            ws.send_ping(vec![]).expect("Failed to send ping");
            last_ping = Instant::now();
        }
    }
}
