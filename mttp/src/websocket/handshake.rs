use crate::{
    http::{self, HttpRequest, HttpResponse, StatusCode},
    websocket::{
        base64,
        consts::headers::{CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE},
        protocol::consts::WEBSOCKET_GUID,
        sha1::sha1,
        WsConnection,
    },
};
use std::{collections::VecDeque, net::TcpStream};

/// Initiates a websocket handshake on a stream, calling the specified handler when complete
pub fn websocket_handshake(
    req: &HttpRequest,
    mut stream: TcpStream,
) -> Result<WsConnection, crate::Error> {
    if req.headers.get(UPGRADE).map(|x| x.to_lowercase()) != Some("websocket".to_owned()) {
        return Err(crate::Error::MissingOrInvalidWebsocketHeader { header: UPGRADE });
    }

    let Some(key) = req.headers.get(SEC_WEBSOCKET_KEY) else {
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

    let response = HttpResponse::builder()
        .status(StatusCode::SwitchingProtocols)
        .header(SEC_WEBSOCKET_ACCEPT, b64encoded)
        .header(CONNECTION, "Upgrade".to_owned())
        .header(UPGRADE, "websocket".to_owned())
        .build();
    http::protocol::write_response(&mut stream, response)?;

    let ws_conn = WsConnection::new(stream, VecDeque::new());

    Ok(ws_conn)
}
