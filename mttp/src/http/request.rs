use super::{header::HeaderMap, Method};

#[derive(Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub uri: String,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
}
