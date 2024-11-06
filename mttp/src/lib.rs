//! # NOT FOR PRODUCTION USE
//! This implementation is not complete and does not strictly adhear to the HTTP spec!

/// Contains the http implementation
pub mod http;

/// Contains the websocket protocol implementation
pub mod websocket;

/// Contains the main server implementation
pub mod server;
mod url;
