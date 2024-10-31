use super::{header::HeaderMap, request::HttpRequest, HttpResponse, Method};
use crate::{
    consts::{headers::CONTENT_LEN, CHUNK_END, HTTP_VER_STR},
    url::parse_query_params_and_urldecode,
    Error,
};
use std::{
    collections::HashMap,
    io::{BufRead, Read, Write},
};

pub(crate) fn parse_request(stream: &mut impl Read) -> Result<HttpRequest, Error> {
    let header_chunk = read_header(stream)?;
    let mut lines = header_chunk.lines();

    let Some(Ok(first)) = lines.next() else {
        return Err(Error::InvalidHeader);
    };
    let mut first_line = first.splitn(4, ' ');

    let method = match first_line.next() {
        Some("GET") => Method::Get,
        Some("POST") => Method::Post,
        Some("PUT") => Method::Put,
        Some("DELETE") => Method::Delete,
        Some("PATCH") => Method::Patch,
        wrong_method => {
            return Err(Error::InvalidMethod {
                recieved: wrong_method.unwrap_or("").to_owned(),
            })
        }
    };

    let raw_uri = first_line.next().ok_or(Error::NoUri)?.to_owned();

    let http_ver = first_line.next().ok_or(Error::UnsupportedVersion)?;
    if http_ver != HTTP_VER_STR {
        return Err(Error::UnsupportedVersion);
    }

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

    let (only_uri, queryparams) = parse_query_params_and_urldecode(&raw_uri);

    Ok(HttpRequest {
        method,
        headers,
        body,
        route: only_uri.to_owned(),
        raw_route: raw_uri,
        params: queryparams,
    })
}

fn read_header(stream: &mut impl Read) -> Result<Vec<u8>, Error> {
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

fn read_body(stream: &mut impl Read, size: usize) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0; size];
    stream.read_exact(&mut buf)?;

    Ok(buf)
}

pub(crate) fn write_response(
    mut stream: impl Write,
    mut response: HttpResponse,
) -> Result<(), crate::Error> {
    if let Some(body) = &response.body {
        response
            .headers
            .values
            .insert(CONTENT_LEN.to_owned(), body.len().to_string());
    }

    stream.write_all(format!("{} {}", HTTP_VER_STR, response.status).as_bytes())?;

    if response.headers.values.len() != 0 {
        for (key, value) in response.headers.values {
            stream.write_all(format!("\r\n{key}: {value}").as_bytes())?;
        }
    }

    if let Some(body) = response.body {
        stream.write_all(CHUNK_END)?;
        stream.write_all(&body)?;
    }

    stream.write_all(CHUNK_END)?;

    Ok(())
}
