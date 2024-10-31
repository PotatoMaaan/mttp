use super::{base64, sha1::sha1};
use crate::{
    consts::headers::ws::{self, SEC_WEBSOCKET_KEY},
    http::{self, HttpRequest, HttpResponse, StatusCode},
    WebSocketMessage, WsHandlerFunc,
};
use core::str;
use std::{
    io::{BufRead, Read, Write},
    net::TcpStream,
};

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub struct WsConnection {
    stream: TcpStream,
}

#[derive(Debug, PartialEq)]
enum TypeLock {
    Text,
    Binary,
    None,
}

impl WsConnection {
    pub fn send(&mut self, message: WebSocketMessage) -> Result<usize, crate::Error> {
        todo!()
    }

    pub fn revc(&mut self) -> Result<WebSocketMessage, crate::Error> {
        let mut message = WebSocketMessage::Close;

        loop {
            let frame = WebsocketFrame::parse(&mut self.stream)?;

            match frame.opcode {
                OpCode::Text => {
                    let string = String::from_utf8(frame.payload).unwrap();
                    message = WebSocketMessage::Text(string);

                    if frame.fin {
                        return Ok(message);
                    }
                }
                OpCode::Binary => {
                    message = WebSocketMessage::Bytes(frame.payload);

                    if frame.fin {
                        return Ok(message);
                    }
                }
                OpCode::Close => {
                    message = WebSocketMessage::Close;

                    if frame.fin {
                        return Ok(message);
                    } else {
                        panic!("illegal");
                    }
                }
                OpCode::Ping => {
                    message = WebSocketMessage::Ping;

                    write!(self.stream, "pong").unwrap();
                    if frame.fin {
                        return Ok(message);
                    } else {
                        panic!("illegal");
                    }
                }
                OpCode::Pong => {
                    if !frame.fin {
                        panic!("illegal");
                    }
                }
                OpCode::Continue => match &mut message {
                    WebSocketMessage::Text(text) => {
                        let new_text = str::from_utf8(&frame.payload).unwrap();
                        text.push_str(&new_text);
                    }
                    WebSocketMessage::Bytes(vec) => todo!(),
                    WebSocketMessage::Close => todo!(),
                    WebSocketMessage::Ping => todo!(),
                    WebSocketMessage::Pong => todo!(),
                },
            }
        }

        todo!()
    }
}

fn xor(payload: &mut [u8], key: [u8; 4]) {
    payload
        .iter_mut()
        .enumerate()
        .for_each(|(i, d)| *d ^= key[i % key.len()])
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Continue,
    Text,
    Binary,
    Close,
    Ping,
    Pong,
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
}

#[derive(Debug)]
struct WebsocketFrame {
    fin: bool,
    opcode: OpCode,
    payload_len: u64,
    masking_key: Option<[u8; 4]>,
    payload: Vec<u8>,
}

impl WebsocketFrame {
    pub fn parse(stream: &mut TcpStream) -> Result<Self, crate::Error> {
        let mut header: [u8; 2] = [0; 2];
        stream.read_exact(&mut header).unwrap();

        let fin = (header[0] & 0b10000000) > 0;
        let opcode = header[0] & 0b00001111;
        let opcode = OpCode::parse(opcode).unwrap();

        let mask = (header[1] & 0b10000000) > 0;

        let payload_len = header[1] & 0b01111111;

        // The payload can be 7 bits, 2 bytes or 8 bytes
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

        dbg!(&payload_len);

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
    if req.headers.get(ws::UPGRADE).map(String::as_str) != Some("websocket") {
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
