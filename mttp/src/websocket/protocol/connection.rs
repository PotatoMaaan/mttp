use super::{frame::WebsocketFrame, Close, CodeRange, OpCode, WebSocketMessage};
use crate::websocket;
use std::{collections::VecDeque, io::Write, net::TcpStream};

#[derive(Debug)]
/// Represents a Websocket connection to a client
pub struct WsConnection {
    stream: TcpStream,
    message_buffer: VecDeque<WebSocketMessage>,
}

#[derive(Debug)]
pub enum TypeLock {
    /// This cannot be a string (see autobahn case 5.6)
    Text(Vec<u8>),
    Binary(Vec<u8>),
    None,
}

enum Len {
    None,
    Single(u8),
    U16(u16),
    U64(u64),
}

impl Len {
    fn payload_len_byte(&self) -> u8 {
        match self {
            Len::None => 0,
            Len::Single(len) => *len,
            Len::U16(_) => 126,
            Len::U64(_) => 127,
        }
    }
}

impl WsConnection {
    pub(crate) fn new(stream: TcpStream, message_buffer: VecDeque<WebSocketMessage>) -> Self {
        Self {
            stream,
            message_buffer,
        }
    }

    /// Sends a message to the client
    ///
    /// This implementation currently does not support splitting messages across multiple frames,
    /// so it's advisable to avoid sending very large messages, as some clients may refuse such large frames.
    pub fn send(&mut self, message: WebSocketMessage) -> Result<(), std::io::Error> {
        let opcode = OpCode::from_msg(&message);

        let payload = match message {
            WebSocketMessage::Text(text) => Some(text.into_bytes()),
            WebSocketMessage::Bytes(vec) => Some(vec),
            WebSocketMessage::Ping(vec) => Some(vec),
            WebSocketMessage::Pong(vec) => Some(vec),
            WebSocketMessage::Close(close) => {
                let mut payload = Vec::with_capacity(125);

                if let Some(close) = close {
                    payload.extend(close.raw_code().to_be_bytes());

                    if let Some(reason) = close.reason {
                        payload.extend(reason.into_bytes());
                    }
                }

                match payload.is_empty() {
                    false => Some(payload),
                    true => None,
                }
            }
        };

        let mut header = [0u8; 2];
        header[0] = opcode as u8;

        // set fin bit (we don't ever split messages across frames)
        header[0] |= 0b10000000;

        // clear reserved bits
        header[0] &= !0b01110000;

        let payload_len = match payload.as_ref().map(Vec::len) {
            Some(len @ ..=125) => Len::Single(len as u8),
            Some(len @ ..=0xFFFF) => Len::U16(len as u16),
            Some(len) => Len::U64(len as u64),
            None => Len::None,
        };
        header[1] = payload_len.payload_len_byte();

        // clear mask bit (server messages must not be masked)
        header[1] &= !0b10000000;

        self.stream.write_all(&header)?;

        match payload_len {
            Len::U16(len) => self.stream.write_all(&len.to_be_bytes())?,
            Len::U64(len) => self.stream.write_all(&len.to_be_bytes())?,
            _ => {}
        }

        if let Some(payload) = payload {
            self.stream.write_all(&payload)?;
        }

        Ok(())
    }

    /// Recieves a message from the client
    ///
    /// # Note
    /// Due to the framing of the websocket messages, this method performs internal
    /// buffering of control messages such as `ping`. If, for example, a client sends
    /// a text message split across multiple frames with a ping message in between,
    /// this method will first return the text message and then the ping message.
    pub fn recv(&mut self) -> Result<WebSocketMessage, websocket::Error> {
        self.recv_inner().map_err(|err| {
            match &err {
                websocket::Error::Protocol(protocol_error) => {
                    self.error(&protocol_error);
                }
                websocket::Error::Local(_) => {}
            }
            err
        })
    }

    fn recv_inner(&mut self) -> Result<WebSocketMessage, websocket::Error> {
        if let Some(msg) = self.message_buffer.pop_front() {
            return Ok(msg);
        }

        let mut type_lock = TypeLock::None;

        loop {
            let frame = WebsocketFrame::parse(&mut self.stream)?;

            match frame.opcode {
                OpCode::Text => match type_lock {
                    TypeLock::None => {
                        if frame.fin {
                            return Ok(WebSocketMessage::Text(String::from_utf8(frame.payload)?));
                        } else {
                            type_lock = TypeLock::Text(frame.payload);
                        }
                    }
                    _ => {
                        return Err(
                            websocket::ProtocolError::AttemptToStartNewMessageWithoutFin.err()
                        )
                    }
                },
                OpCode::Binary => match type_lock {
                    TypeLock::None => {
                        if frame.fin {
                            return Ok(WebSocketMessage::Bytes(frame.payload));
                        } else {
                            type_lock = TypeLock::Binary(frame.payload);
                        }
                    }
                    _ => {
                        return Err(
                            websocket::ProtocolError::AttemptToStartNewMessageWithoutFin.err()
                        )
                    }
                },
                OpCode::Close => {
                    if !frame.fin {
                        return Err(websocket::ProtocolError::ControlFrameNotFin.err());
                    }

                    let close = if !frame.payload.is_empty() {
                        let code: [u8; 2] = frame
                            .payload
                            .get(0..2)
                            .and_then(|x| x.try_into().ok())
                            .ok_or(websocket::ProtocolError::InvalidCloseFrame)?;
                        let code = u16::from_be_bytes(code);

                        let mut payload = frame.payload;
                        payload.drain(0..2);

                        Some(Close {
                            code: CodeRange::parse(code)?,
                            reason: if payload.is_empty() {
                                None
                            } else {
                                Some(String::from_utf8(payload)?)
                            },
                        })
                    } else {
                        None
                    };

                    self.close(close.clone())?;

                    return Ok(WebSocketMessage::Close(close));
                }
                OpCode::Ping => {
                    if !frame.fin {
                        return Err(websocket::ProtocolError::ControlFrameNotFin.err());
                    }

                    self.send(WebSocketMessage::Pong(frame.payload.clone()))?;

                    self.message_buffer
                        .push_back(WebSocketMessage::Ping(frame.payload));
                }
                OpCode::Pong => {
                    if !frame.fin {
                        return Err(websocket::ProtocolError::ControlFrameNotFin.err());
                    }

                    self.message_buffer
                        .push_back(WebSocketMessage::Ping(frame.payload));
                }
                OpCode::Continue => {
                    match &mut type_lock {
                        TypeLock::Text(vec) => {
                            vec.extend(frame.payload);
                        }
                        TypeLock::Binary(vec) => {
                            vec.extend(frame.payload);
                        }
                        TypeLock::None => {
                            return Err(websocket::ProtocolError::ContinueWithoutStart.err());
                        }
                    };

                    if frame.fin {
                        return Ok(match type_lock {
                            TypeLock::Text(vec) => WebSocketMessage::Text(String::from_utf8(vec)?),
                            TypeLock::Binary(vec) => WebSocketMessage::Bytes(vec),
                            TypeLock::None => unreachable!(),
                        });
                    }
                }
            }
        }
    }

    fn close(&mut self, close: Option<Close>) -> Result<(), std::io::Error> {
        self.send(WebSocketMessage::Close(close))?;
        self.stream.shutdown(std::net::Shutdown::Both)?;

        Ok(())
    }

    fn error(&mut self, error: &websocket::ProtocolError) {
        _ = self.close(Some(error.close()));
    }
}
