use super::{base64, sha1::sha1};
use crate::{
    consts::headers::ws::{self, SEC_WEBSOCKET_KEY},
    http::{self, HttpRequest, HttpResponse, StatusCode},
    WebSocketMessage, WsHandlerFunc,
};
use core::str;
use std::{
    io::{Read, Write},
    net::TcpStream,
    usize,
};

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WsConnection {
    stream: TcpStream,
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
    pub fn send(&mut self, message: WebSocketMessage) -> Result<(), crate::Error> {
        let opcode = OpCode::from_msg(&message);

        let payload = match message {
            WebSocketMessage::Text(text) => Some(text.into_bytes()),
            WebSocketMessage::Bytes(vec) => Some(vec),
            WebSocketMessage::Ping(vec) => Some(vec),
            WebSocketMessage::Pong(vec) => Some(vec),
            WebSocketMessage::Close { code, reason } => {
                let size_hint = reason.as_ref().and_then(|x| Some(x.len())).unwrap_or(0) + 2;
                let mut payload = Vec::with_capacity(size_hint);

                if let Some(code) = code {
                    payload.extend(code.to_be_bytes());

                    if let Some(reason) = reason {
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

        let payload_len = match payload.as_ref().map(|x| x.len()) {
            Some(len @ ..=125) => Len::Single(len as u8),
            Some(len @ ..=0xFFFF) => Len::U16(len as u16),
            Some(len) => Len::U64(len as u64),
            None => Len::None,
        };
        header[1] = payload_len.payload_len_byte();

        // clear mask bit (server messages must not be masked)
        header[1] &= !0b10000000;

        self.stream.write_all(&header).unwrap();

        match payload_len {
            Len::U16(len) => self.stream.write_all(&len.to_be_bytes()).unwrap(),
            Len::U64(len) => self.stream.write_all(&len.to_be_bytes()).unwrap(),
            _ => {}
        }

        if let Some(payload) = payload {
            self.stream.write_all(&payload).unwrap();
        }

        Ok(())
    }

    pub fn recv(&mut self) -> Result<WebSocketMessage, crate::Error> {
        let mut message = None;

        loop {
            let frame = WebsocketFrame::parse(&mut self.stream)?;

            match frame.opcode {
                OpCode::Text => {
                    let string = String::from_utf8(frame.payload).unwrap();
                    message = Some(WebSocketMessage::Text(string));
                }
                OpCode::Binary => {
                    message = Some(WebSocketMessage::Bytes(frame.payload));
                }
                OpCode::Close => {
                    let (code, reason) = if !frame.payload.is_empty() {
                        let code: [u8; 2] = frame.payload.get(0..2).unwrap().try_into().unwrap();
                        let code = u16::from_be_bytes(code);

                        let mut payload = frame.payload;
                        payload.remove(0);
                        payload.remove(0);

                        (
                            Some(code),
                            if !payload.is_empty() {
                                String::from_utf8(payload).ok()
                            } else {
                                None
                            },
                        )
                    } else {
                        (None, None)
                    };

                    self.close(code, reason.clone())?;

                    message = Some(WebSocketMessage::Close { code, reason });

                    if !frame.fin {
                        panic!("illegal");
                    }
                }
                OpCode::Ping => {
                    self.send(WebSocketMessage::Pong(frame.payload.clone()))?;
                    message = Some(WebSocketMessage::Ping(frame.payload));

                    if !frame.fin {
                        panic!("illegal");
                    }
                }
                OpCode::Pong => {
                    message = Some(WebSocketMessage::Pong(frame.payload));

                    if !frame.fin {
                        panic!("illegal");
                    }
                }
                OpCode::Continue => match message.as_mut().expect("should always be set") {
                    WebSocketMessage::Text(text) => {
                        let new_text = str::from_utf8(&frame.payload).unwrap();
                        text.push_str(&new_text);
                    }
                    WebSocketMessage::Bytes(vec) => {
                        vec.extend(frame.payload);
                    }
                    WebSocketMessage::Close { code: _, reason: _ } => panic!("invalid"),
                    WebSocketMessage::Ping(_) => panic!("invalid"),
                    WebSocketMessage::Pong(_) => panic!("invalid"),
                },
            }

            // if this is the last frame so we can return the message
            if frame.fin {
                return Ok(message.expect("should always have been set"));
            }
        }
    }

    fn close(&mut self, code: Option<u16>, reason: Option<String>) -> Result<(), crate::Error> {
        self.send(WebSocketMessage::Close { code, reason })?;
        self.stream.shutdown(std::net::Shutdown::Both).unwrap();

        Ok(())
    }
}

fn xor(payload: &mut [u8], key: [u8; 4]) {
    payload
        .iter_mut()
        .enumerate()
        .for_each(|(i, d)| *d ^= key[i % key.len()])
}

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
    pub fn parse(code: u8) -> Option<OpCode> {
        match code {
            0x0 => Some(OpCode::Continue),
            0x1 => Some(OpCode::Text),
            0x2 => Some(OpCode::Binary),
            0x8 => Some(OpCode::Close),
            0x9 => Some(OpCode::Ping),
            0xA => Some(OpCode::Pong),
            _ => None,
        }
    }

    fn from_msg(msg: &WebSocketMessage) -> Self {
        match msg {
            WebSocketMessage::Text(_) => OpCode::Text,
            WebSocketMessage::Bytes(_) => OpCode::Binary,
            WebSocketMessage::Close { code: _, reason: _ } => OpCode::Close,
            WebSocketMessage::Ping(_) => OpCode::Ping,
            WebSocketMessage::Pong(_) => OpCode::Pong,
        }
    }

    fn is_control(&self) -> bool {
        match self {
            OpCode::Text => false,
            OpCode::Binary => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone)]
struct WebsocketFrame {
    fin: bool,
    opcode: OpCode,
    payload_len: u64,
    masking_key: Option<[u8; 4]>,
    payload: Vec<u8>,
}

impl WebsocketFrame {
    pub fn parse(stream: &mut TcpStream) -> Result<Self, crate::Error> {
        let mut header = [0; 1];
        stream.read_exact(&mut header).unwrap();
        let header = header[0];

        let fin = (header & 0b10000000) > 0;
        let opcode = header & 0b00001111;
        let opcode = OpCode::parse(opcode).unwrap();

        let mut header2 = [0; 1];
        stream.read_exact(&mut header2).unwrap();
        let header2 = header2[0];

        let mask = (header2 & 0b10000000) > 0;
        let payload_len = header2 & 0b01111111;

        // The payload len can be 7 bits, 2 bytes or 8 bytes
        let payload_len = match payload_len {
            ..=125 => payload_len as u64,
            126 => {
                let mut longer_len = [0; 2];
                stream.read_exact(&mut longer_len).unwrap();
                u16::from_be_bytes(longer_len) as u64
            }
            127..=u8::MAX => {
                let mut much_longer_len = [0; 8];
                stream.read_exact(&mut much_longer_len).unwrap();
                u64::from_be_bytes(much_longer_len)
            }
        };

        if opcode.is_control() && payload_len > 125 {
            panic!("illegal length");
        }

        let masking_key = if mask {
            let mut key = [0; 4];
            stream.read_exact(&mut key).unwrap();
            Some(key)
        } else {
            None
        };

        let mut payload = vec![0; payload_len as usize];
        stream.read_exact(&mut payload).unwrap();

        if let Some(masking_key) = masking_key {
            xor(&mut payload, masking_key);
        }

        assert_eq!(masking_key.is_some(), mask);

        Ok(WebsocketFrame {
            fin,
            opcode,
            payload_len,
            masking_key,
            payload,
        })
    }
}

#[derive(Debug)]
enum FrameType {
    Continue,
    Text(String),
}

pub fn websocket<S: Clone>(
    state: S,
    req: HttpRequest,
    handler: WsHandlerFunc<S>,
    mut stream: TcpStream,
) -> Result<(), crate::Error> {
    if req.headers.get(ws::UPGRADE).map(|x| x.to_lowercase()) != Some("websocket".to_owned()) {
        return Err(crate::Error::MissingOrInvalidWebsocketHeader {
            header: ws::UPGRADE,
        });
    }

    let Some(key) = req.headers.get(ws::SEC_WEBSOCKET_KEY) else {
        return Err(crate::Error::MissingOrInvalidWebsocketHeader {
            header: SEC_WEBSOCKET_KEY,
        });
    };

    let b64encoded = {
        let mut key = key.clone();
        key.push_str(WEBSOCKET_GUID);
        let sha = sha1(key.as_bytes());
        base64::encode(&sha)
    };

    println!("sending");

    let response = HttpResponse::builder()
        .status(StatusCode::SwitchingProtocols)
        .header(ws::SEC_WEBSOCKET_ACCEPT, b64encoded)
        .header(ws::CONNECTION, "Upgrade".to_owned())
        .header(ws::UPGRADE, "websocket".to_owned())
        .build();
    http::protocol::write_response(&mut stream, response)?;

    let ws_conn = WsConnection { stream };

    println!("Calling WS handler");
    handler(state, &req, ws_conn);

    Ok(())
}
