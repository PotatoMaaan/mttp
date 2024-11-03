use crate::websocket;

use super::WebSocketMessage;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    Continue = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl OpCode {
    pub fn parse(code: u8) -> Result<Self, websocket::ProtocolError> {
        match code {
            0x0 => Ok(OpCode::Continue),
            0x1 => Ok(OpCode::Text),
            0x2 => Ok(OpCode::Binary),
            0x8 => Ok(OpCode::Close),
            0x9 => Ok(OpCode::Ping),
            0xA => Ok(OpCode::Pong),
            _ => Err(websocket::ProtocolError::InvalidOpcode(code)),
        }
    }

    pub fn from_msg(msg: &WebSocketMessage) -> Self {
        match msg {
            WebSocketMessage::Text(_) => OpCode::Text,
            WebSocketMessage::Bytes(_) => OpCode::Binary,
            WebSocketMessage::Close(_) => OpCode::Close,
            WebSocketMessage::Ping(_) => OpCode::Ping,
            WebSocketMessage::Pong(_) => OpCode::Pong,
        }
    }

    pub fn is_control(&self) -> bool {
        match self {
            OpCode::Text => false,
            OpCode::Binary => false,
            OpCode::Continue => false,
            _ => true,
        }
    }
}
