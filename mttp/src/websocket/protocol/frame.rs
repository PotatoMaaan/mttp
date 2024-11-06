use super::OpCode;
use crate::websocket;
use std::{
    borrow::Cow,
    io::{Read, Write},
    net::TcpStream,
};

#[derive(Debug, Clone)]
pub struct WebsocketFrame<'payload> {
    pub fin: bool,
    pub opcode: OpCode,
    pub payload: Cow<'payload, [u8]>,
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

fn xor(payload: &mut [u8], key: [u8; 4]) {
    payload
        .iter_mut()
        .enumerate()
        .for_each(|(i, d)| *d ^= key[i % key.len()])
}

impl<'payload> WebsocketFrame<'payload> {
    pub fn write(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        let mut header = [0u8; 2];
        header[0] = self.opcode as u8;

        // set fin bit (we don't ever split messages across frames)
        header[0] |= 0b10000000;

        // clear reserved bits
        header[0] &= !0b01110000;

        let payload_len = match self.payload.len() {
            len @ ..=125 => Len::Single(len as u8),
            len @ ..=0xFFFF => Len::U16(len as u16),
            len => Len::U64(len as u64),
        };
        header[1] = payload_len.payload_len_byte();

        // clear mask bit (server messages must not be masked)
        header[1] &= !0b10000000;

        stream.write_all(&header)?;

        match payload_len {
            Len::U16(len) => stream.write_all(&len.to_be_bytes())?,
            Len::U64(len) => stream.write_all(&len.to_be_bytes())?,
            _ => {}
        }

        stream.write_all(&self.payload)?;

        Ok(())
    }

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
            payload: Cow::Owned(payload),
        })
    }
}
