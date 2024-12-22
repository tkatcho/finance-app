#[derive(Debug)]
pub enum WebSocketError {
    HandshakeFailed(String),
    // Add other error variants as needed
}
impl WebSocketError {
    pub fn HandshakeFailed(message: &str) -> WebSocketError {
        WebSocketError::HandshakeFailed(message.to_string())
    }
    fn from(error: WebSocketError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, error.to_string())
    }

    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
