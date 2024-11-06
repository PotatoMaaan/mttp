pub(crate) const CHUNK_END: &[u8; 4] = b"\r\n\r\n";

pub const HTTP_VER_STR: &str = "HTTP/1.1";

pub mod headers {
    pub const CONTENT_LEN: &str = "Content-Length";
    pub const COOKIES: &str = "Cookie";
    pub const CONTENT_TYPE: &str = "Content-Type";
    pub const ORIGIN: &str = "Origin";
}
