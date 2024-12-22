pub struct Request {
    pub headers: Vec<String>,
    pub raw: Vec<u8>,
}

impl Request {
    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|h| h.to_lowercase().starts_with(&name.to_lowercase()))
            .map(|h| h.split(':').nth(1))
            .flatten()
            .map(|s| s.trim())
    }
}
