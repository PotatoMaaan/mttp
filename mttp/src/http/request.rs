use super::{header::HeaderMap, Method};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: Method,
    pub raw_route: String,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
    pub route: String,
    pub params: HashMap<String, String>,
}
