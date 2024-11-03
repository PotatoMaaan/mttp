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
