use std::io::Error;

mod pool;
mod worker;

pub use pool::ThreadPool;
pub use worker::Message;

pub type Result<T> = std::result::Result<T, Error>;
