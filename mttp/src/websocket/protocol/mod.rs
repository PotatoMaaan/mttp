mod close;
mod connection;
pub mod consts;
pub mod error;
mod frame;
mod opcode;

#[cfg(test)]
mod test;

use opcode::*;

pub use close::{Close, CloseReason, CodeRange};
pub use connection::WsConnection;

/// A message recieved through a websocket connection
#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    /// Contains valid UTF-8 text
    Text(String),
    /// Contains arbitrary bytes
    Bytes(Vec<u8>),
    /// Signals the connection to close (closing handled by library)
    Close(Option<Close>),
    /// The server recieved a ping request (handled by the library)
    Ping(Vec<u8>),
    /// The client responded with a pong
    Pong(Vec<u8>),
}

/// The same as [`WebSocketMessage`], but containing only borrowd data (used for sending data)
#[derive(Debug, Clone)]
pub enum WebSocketMessageRef<'payload> {
    Text(&'payload str),
    Bytes(&'payload [u8]),
    Close(Option<&'payload Close>),
    Ping(&'payload [u8]),
    Pong(&'payload [u8]),
}

impl<'payload> WebSocketMessageRef<'payload> {
    pub(crate) fn opcode(&self) -> OpCode {
        match self {
            WebSocketMessageRef::Text(_) => OpCode::Text,
            WebSocketMessageRef::Bytes(_) => OpCode::Binary,
            WebSocketMessageRef::Close(_) => OpCode::Close,
            WebSocketMessageRef::Ping(_) => OpCode::Ping,
            WebSocketMessageRef::Pong(_) => OpCode::Pong,
        }
    }
}
