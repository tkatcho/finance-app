use std::{
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
};

use super::worker::{Message, Worker};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        //todo switch this for 1 receiver and multiple senders
        
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute(&self, stream: TcpStream) {
        self.sender.send(Message::NewConnection(stream)).unwrap();
    }
}
