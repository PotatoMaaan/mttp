mod close;
mod connection;
pub mod consts;
pub mod error;
mod frame;
mod opcode;

use opcode::*;

pub use close::{Close, CloseReason, CodeRange};
pub use connection::WsConnection;

#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Bytes(Vec<u8>),
    Close(Option<Close>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

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
