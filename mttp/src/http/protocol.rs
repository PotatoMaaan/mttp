use super::{header::HeaderMap, request::HttpRequest, Method};
use crate::{consts::CHUNK_END, Error};
use std::{
    collections::HashMap,
    io::{BufRead, Read},
    net::TcpStream,
};

pub(crate) fn handle_http(stream: &mut TcpStream) -> Result<HttpRequest, Error> {
    let header_chunk = read_header(stream)?;
    let mut lines = header_chunk.lines();

    let Some(Ok(first)) = lines.next() else {
        return Err(Error::InvalidHeader);
    };
    let mut first_line = first.splitn(3, ' ');

    let method = match first_line.next() {
        Some("GET") => Method::Get,
        Some("POST") => Method::Post,
        wrong_method => {
            return Err(Error::InvalidMethod {
                recieved: wrong_method.unwrap_or("").to_owned(),
            })
        }
    };

    let uri = first_line.next().ok_or(Error::NoUri)?.to_owned();

    let mut headers = HashMap::new();
    for header_line in lines {
        let header_line = header_line?;
        let (key, value) = header_line.split_once(": ").ok_or(Error::InvalidHeader)?;
        headers.insert(key.to_owned(), value.to_owned());
    }
    let headers = HeaderMap { values: headers };

    let body = if let Some(content_len) = headers.content_length() {
        Some(read_body(stream, content_len)?)
    } else {
        None
    };

    Ok(HttpRequest {
        method,
        uri,
        headers,
        body,
    })
}

pub(crate) fn read_header(stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
    let mut total: Vec<u8> = Vec::with_capacity(128);

    let mut current = [0; 4];

    loop {
        let mut latest = [0];
        let got = stream.read(&mut latest)?;
        if got == 0 {
            break;
        }
        total.push(latest[0]);

        current[0] = current[1];
        current[1] = current[2];
        current[2] = current[3];
        current[3] = latest[0];

        if &current == CHUNK_END {
            total.truncate(total.len() - CHUNK_END.len());
            return Ok(total);
        }
    }

    Ok(total)
}

pub(crate) fn read_body(stream: &mut TcpStream, size: usize) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0; size];
    let read = stream.read(&mut buf)?;
    if read != size {
        return Err(Error::BodyTooShort);
    }

    Ok(buf)
}
