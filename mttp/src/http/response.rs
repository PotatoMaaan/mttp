use super::{header::HeaderMap, StatusCode};
use crate::http::consts::headers::CONTENT_TYPE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Option<Vec<u8>>,
}

pub struct HttpResponseBuilder {
    status: StatusCode,
    header: HeaderMap,
    body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder {
            status: StatusCode::Ok,
            header: HeaderMap::empty(),
            body: None,
        }
    }

    pub fn sucess() -> Self {
        Self {
            status: StatusCode::Ok,
            headers: HeaderMap::empty(),
            body: None,
        }
    }

    pub fn not_found() -> Self {
        HttpResponse::builder()
            .status(StatusCode::NotFound)
            .text("The requested resource was not found on the server".to_owned())
            .build()
    }
}

impl HttpResponseBuilder {
    pub fn header(mut self, key: &str, value: String) -> Self {
        self.header.values.insert(key.to_owned(), value);
        self
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.header = headers;
        self
    }

    pub fn body(mut self, body: Option<Vec<u8>>) -> Self {
        self.body = body;
        self
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.body = Some(text.into_bytes());
        self.header
            .values
            .insert(CONTENT_TYPE.to_owned(), "text/plain".to_owned());
        self
    }

    pub fn json(mut self, json: String) -> Self {
        self.body = Some(json.into_bytes());
        self.header
            .values
            .insert(CONTENT_TYPE.to_owned(), "application/json".to_owned());
        self
    }

    pub fn bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body = Some(bytes);
        self
    }

    pub fn build(self) -> HttpResponse {
        HttpResponse {
            status: self.status,
            headers: self.header,
            body: self.body,
        }
    }
}
