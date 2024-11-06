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

impl WebSocketMessage {
    pub(crate) fn opcode(&self) -> OpCode {
        match self {
            WebSocketMessage::Text(_) => OpCode::Text,
            WebSocketMessage::Bytes(_) => OpCode::Binary,
            WebSocketMessage::Close(_) => OpCode::Close,
            WebSocketMessage::Ping(_) => OpCode::Ping,
            WebSocketMessage::Pong(_) => OpCode::Pong,
        }
    }
}
