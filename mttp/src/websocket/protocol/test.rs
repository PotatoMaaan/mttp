use super::frame::{WebsocketFrame, WebsocketFrameRef};
use std::io::Cursor;

#[test]
fn test_frame_deser() {
    let mut stream = Cursor::new([
        0b10000001u8, // fin, opcode 1 (text)
        0b10011010,   // mask, payload len (26)
        0b11110000,   // masking key
        0b00001111,   // masking key
        0b11110000,   // masking key
        0b00001111,   // masking key
        184,          // payload \/
        106,
        156,
        99,
        159,
        47,
        167,
        96,
        130,
        99,
        148,
        47,
        184,
        106,
        156,
        99,
        159,
        47,
        167,
        96,
        130,
        99,
        148,
        46,
        209,
        46,
    ]);

    let parsed = WebsocketFrame::parse(&mut stream).unwrap();

    let reference = WebsocketFrameRef {
        fin: true,
        opcode: crate::websocket::protocol::OpCode::Text,
        payload: b"Hello World Hello World!!!",
    };

    assert_eq!(parsed, reference);
}

#[test]
fn test_frame_ser() {
    const TEXT: &[u8; 16] = b"Man I love Fauna";

    let frame = WebsocketFrameRef {
        fin: true,
        opcode: super::OpCode::Text,
        payload: TEXT,
    };

    let mut stream = Cursor::new(Vec::new());
    frame.write(&mut stream).unwrap();

    let mut reference = vec![0b10000001u8, 0b00010000];
    reference.extend(TEXT);

    stream.set_position(0);

    assert_eq!(stream.into_inner(), reference);
}
