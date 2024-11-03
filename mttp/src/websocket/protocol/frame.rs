use super::OpCode;
use crate::websocket;
use std::{io::Read, net::TcpStream};

#[derive(Debug, Clone)]
pub struct WebsocketFrame {
    pub fin: bool,
    pub opcode: OpCode,
    pub payload: Vec<u8>,
}

fn xor(payload: &mut [u8], key: [u8; 4]) {
    payload
        .iter_mut()
        .enumerate()
        .for_each(|(i, d)| *d ^= key[i % key.len()])
}

impl WebsocketFrame {
    pub fn parse(stream: &mut TcpStream) -> Result<Self, websocket::Error> {
        let mut header = [0; 2];
        stream.read_exact(&mut header)?;

        let fin = (header[0] & 0b10000000) > 0;
        let opcode = header[0] & 0b00001111;
        let opcode = OpCode::parse(opcode)?;

        let rsv_bits = header[0] & 0b01110000;
        if rsv_bits != 0 {
            return Err(websocket::ProtocolError::ReservedBitsSet.err());
        }

        let mask = (header[1] & 0b10000000) > 0;
        let payload_len = header[1] & 0b01111111;

        if !mask {
            return Err(websocket::ProtocolError::UnmaskedClientMessage.err());
        }

        // The payload len can be 7 bits, 2 bytes or 8 bytes
        let payload_len = match payload_len {
            ..=125 => payload_len as u64,
            126 => {
                let mut longer_len = [0; 2];
                stream.read_exact(&mut longer_len)?;
                u16::from_be_bytes(longer_len) as u64
            }
            127..=u8::MAX => {
                let mut much_longer_len = [0; 8];
                stream.read_exact(&mut much_longer_len)?;
                u64::from_be_bytes(much_longer_len)
            }
        };

        if opcode.is_control() && payload_len > 125 {
            return Err(websocket::ProtocolError::ControlPayloadTooLarge(payload_len).err());
        }

        let masking_key = if mask {
            let mut key = [0; 4];
            stream.read_exact(&mut key)?;
            Some(key)
        } else {
            None
        };

        let mut payload = vec![0; payload_len as usize];
        stream.read_exact(&mut payload)?;

        if let Some(masking_key) = masking_key {
            xor(&mut payload, masking_key);
        }

        assert_eq!(masking_key.is_some(), mask);

        Ok(WebsocketFrame {
            fin,
            opcode,
            payload,
        })
    }
}
