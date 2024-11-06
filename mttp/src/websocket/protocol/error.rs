use super::{consts::MAX_RECV_FRAME_SIZE, Close, CloseReason};
use std::{fmt::Display, string::FromUtf8Error};

#[derive(Debug)]
pub enum Error {
    Protocol(ProtocolError),
    Local(std::io::Error),
}

#[derive(Debug)]
pub enum ProtocolError {
    ControlPayloadTooLarge(u64),
    PayloadTooLarge(u64),
    UnmaskedClientMessage,
    ReservedBitsSet,
    InvalidOpcode(u8),
    AttemptToStartNewMessageWithoutFin,
    ContinueWithoutStart,
    ControlFrameNotFin,
    InvalidCloseFrame,
    InvalidCloseCode(u16),
    InvalidUtf8(FromUtf8Error),
}

impl ProtocolError {
    pub(crate) fn err(self) -> Error {
        Error::Protocol(self)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::Protocol(ProtocolError::InvalidUtf8(value))
    }
}

impl From<ProtocolError> for Error {
    fn from(value: ProtocolError) -> Self {
        Self::Protocol(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Local(value)
    }
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::ControlPayloadTooLarge(size) => {
                write!(
                    f,
                    "Control frame had a too large payload (allowed: 125, got: {})",
                    size
                )
            }
            ProtocolError::UnmaskedClientMessage => write!(f, "Client message was not masked"),
            ProtocolError::ReservedBitsSet => write!(
                f,
                "Reserved bits were set when no extension protocol using these bits was negotiated"
            ),
            ProtocolError::InvalidOpcode(code) => {
                write!(f, "Recieved invalid opcode: {:01X}", code)
            }
            ProtocolError::AttemptToStartNewMessageWithoutFin => write!(
                f,
                "Attempted to start a new message without finishing an existing one"
            ),
            ProtocolError::ContinueWithoutStart => {
                write!(f, "Sent continue frame without having started a message")
            }
            ProtocolError::ControlFrameNotFin => {
                write!(f, "Sent a control frame without fin bit set")
            }
            ProtocolError::InvalidCloseFrame => write!(f, "Recieved an invalid close frame"),
            ProtocolError::InvalidCloseCode(code) => write!(f, "Invalid close code: {}", code),
            ProtocolError::InvalidUtf8(err) => write!(f, "Sent invalid UTF-8: {err}"),
            ProtocolError::PayloadTooLarge(len) => write!(
                f,
                "The payload was too large for this implementation: {} (max: {})",
                len, MAX_RECV_FRAME_SIZE
            ),
        }
    }
}

impl ProtocolError {
    pub fn close(&self) -> Close {
        Close {
            code: super::CodeRange::Defined(match self {
                ProtocolError::InvalidUtf8(_) => CloseReason::InconsistentData,
                ProtocolError::PayloadTooLarge(_) => CloseReason::TooBig,
                _ => CloseReason::ProtocolError,
            }),
            reason: Some(format!("{self}")),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Protocol(err) => {
                write!(f, "Client didn't follow websocket protocol: {err}")
            }
            Error::Local(err) => write!(f, "IO Error while operating on websocket: {}", err),
        }
    }
}
