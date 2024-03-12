use crate::consts::{HEADER_CONTENT_LEN, HEADER_COOKIES};
use std::collections::HashMap;

#[derive(Debug)]
pub struct HeaderMap {
    pub values: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
}

impl HeaderMap {
    pub fn content_length(&self) -> Option<usize> {
        if let Some(value) = self.values.get(HEADER_CONTENT_LEN) {
            value.parse().ok()
        } else {
            None
        }
    }

    pub fn cookies(&self) -> HashMap<&str, &str> {
        let Some(cookies_str) = self.values.get(HEADER_COOKIES) else {
            return HashMap::new();
        };

        if let Some(cookies) = cookies_str
            .split("; ")
            .map(|x| x.split_once('='))
            .collect::<Option<HashMap<_, _>>>()
        {
            cookies
        } else {
            HashMap::new()
        }
    }
}
