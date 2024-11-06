use super::{
    consts::CHUNK_SIZE,
    frame::{WebsocketFrame, WebsocketFrameRef},
    Close, CodeRange, OpCode, WebSocketMessage, WebSocketMessageRef,
};
use crate::websocket;
use core::str;
use std::{
    borrow::{Borrow, Cow},
    collections::VecDeque,
    io::Write,
    net::TcpStream,
};

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
    pub fn send(&mut self, message: &WebSocketMessageRef) -> Result<(), std::io::Error> {
        let opcode = message.opcode();

        let payload = match message {
            WebSocketMessageRef::Text(payload) => Cow::Borrowed(payload.as_bytes()),
            WebSocketMessageRef::Bytes(payload) => Cow::Borrowed(*payload),
            &WebSocketMessageRef::Close(Some(Close {
                ref code,
                ref reason,
            })) => {
                let mut payload = Vec::from(code.code().to_be_bytes());
                if let Some(reason) = reason {
                    payload.extend(reason.as_bytes());
                }

                Cow::Owned(payload)
            }
            WebSocketMessageRef::Close(None) => Cow::Owned(Vec::new()),
            WebSocketMessageRef::Ping(payload) => Cow::Borrowed(*payload),
            WebSocketMessageRef::Pong(payload) => Cow::Borrowed(*payload),
        };

        let frames = if !opcode.is_control() && payload.len() > CHUNK_SIZE {
            let mut frames = payload
                .chunks(CHUNK_SIZE)
                .map(|payload| WebsocketFrameRef {
                    fin: false,
                    opcode: OpCode::Continue,
                    payload,
                })
                .collect::<Vec<_>>();
            assert!(frames.len() >= 2);

            if let Some(first_frame) = frames.first_mut() {
                first_frame.opcode = opcode;
            }

            if let Some(last_frame) = frames.last_mut() {
                last_frame.fin = true;
            }

            frames
        } else {
            vec![WebsocketFrameRef {
                fin: true,
                opcode,
                payload: payload.borrow(),
            }]
        };

        dbg!(&frames.len());

        for frame in frames {
            dbg!(&frame.fin, frame.opcode, frame.payload.len());

            frame.write(&mut self.stream)?;
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
                            return Ok(WebSocketMessage::Text(String::from_utf8(
                                frame.payload.to_vec(),
                            )?));
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

                    self.send(&WebSocketMessageRef::Pong(&frame.payload))?;

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
        self.send(&WebSocketMessageRef::Close(close))?;
        self.stream.shutdown(std::net::Shutdown::Both)?;

        Ok(())
    }

    fn error(&mut self, error: &websocket::ProtocolError) {
        _ = self.close(Some(error.close()));
    }
}