pub(crate) const CHUNK_END: &[u8; 4] = b"\r\n\r\n";

pub const HTTP_VER_STR: &str = "HTTP/1.1";

pub mod headers {
    pub const CONTENT_LEN: &str = "Content-Length";
    pub const COOKIES: &str = "Cookie";
    pub const CONTENT_TYPE: &str = "Content-Type";
    pub const ORIGIN: &str = "Origin";

    pub mod ws {
        pub const UPGRADE: &str = "Upgrade";
        pub const CONNECTION: &str = "Connection";
        pub const SEC_WEBSOCKET_KEY: &str = "Sec-WebSocket-Key";
        pub const SEC_WEBSOCKET_PROTOCOL: &str = "Sec-WebSocket-Protocol";
        pub const SEC_WEBSOCKET_VERSION: &str = "Sec-WebSocket-Version";
        pub const SEC_WEBSOCKET_ACCEPT: &str = "Sec-WebSocket-Accept";
    }
}
