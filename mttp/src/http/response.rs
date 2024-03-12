use super::header::HeaderMap;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub msg: String,
    pub header: HeaderMap,
    pub body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn from_text(status: u16, msg: &str, body: String) -> Self {
        Self {
            status,
            msg: msg.to_owned(),
            header: HeaderMap {
                values: HashMap::new(),
            },
            body: Some(body.into_bytes()),
        }
    }

    pub fn from_bytes(status: u16, msg: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            msg: msg.to_owned(),
            header: HeaderMap {
                values: HashMap::new(),
            },
            body: Some(body),
        }
    }

    pub fn from_status(status: u16, msg: &str) -> Self {
        Self {
            status,
            msg: msg.to_owned(),
            header: HeaderMap {
                values: HashMap::new(),
            },
            body: None,
        }
    }
}
