use finance_app::workers::ThreadPool;
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("WebSocket server listening on port 8080");

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {:?}", stream);
                pool.execute(stream); // Send to thread pool instead of spawning new thread
            }
            Err(e) => {
                eprintln!("Failed to establish a connection: {}", e);
            }
        }
    }

    Ok(())
}
