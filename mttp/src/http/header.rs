use crate::consts::{HEADER_CONTENT_LEN, HEADER_CONTENT_TYPE, HEADER_COOKIES};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderMap {
    pub values: HashMap<String, String>,
}

impl<const N: usize> From<[(&str, &str); N]> for HeaderMap {
    fn from(value: [(&str, &str); N]) -> Self {
        Self {
            values: value
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

impl<const N: usize> From<[(String, String); N]> for HeaderMap {
    fn from(value: [(String, String); N]) -> Self {
        Self {
            values: value.into_iter().collect(),
        }
    }
}

impl HeaderMap {
    pub fn empty() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn content_length(&self) -> Option<usize> {
        if let Some(value) = self.values.get(HEADER_CONTENT_LEN) {
            value.parse().ok()
        } else {
            None
        }
    }

    pub fn content_type(&self) -> Option<&str> {
        self.values.get(HEADER_CONTENT_TYPE).map(|x| x.as_str())
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

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
}
