pub const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub mod headers {
    pub const UPGRADE: &str = "Upgrade";
    pub const CONNECTION: &str = "Connection";
    pub const SEC_WEBSOCKET_KEY: &str = "Sec-WebSocket-Key";
    pub const SEC_WEBSOCKET_PROTOCOL: &str = "Sec-WebSocket-Protocol";
    pub const SEC_WEBSOCKET_VERSION: &str = "Sec-WebSocket-Version";
    pub const SEC_WEBSOCKET_ACCEPT: &str = "Sec-WebSocket-Accept";
}
