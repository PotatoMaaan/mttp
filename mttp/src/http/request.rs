use super::{header::HeaderMap, Method};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: Method,
    pub route: String,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
}
