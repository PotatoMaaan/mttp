use std::{
    collections::HashMap,
    io::{self, BufRead, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

const CHUNK_END: &[u8; 4] = b"\r\n\r\n";
pub const HEADER_CONTENT_LEN: &str = "Content-Length";

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Empty,
    InvalidHeader,
    NoUri,
    InvalidMethod { recieved: String },
    InvalidHeaderValue { header: String },
    TooLong { size: usize },
    BodyTooLong,
    BodyTooShort,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

pub fn start_server(addr: SocketAddr) {
    let socket = TcpListener::bind(addr).expect("Failed to start server");

    println!("Listening...");
    while let Ok((mut stream, addr)) = socket.accept() {
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        println!("New connection from: {:?}", &addr);
        std::thread::spawn(move || {
            let req = handle_http(&mut stream);
            dbg!(&req);

            let req = match req {
                Ok(v) => v,
                Err(e) => {
                    stream
                        .write_all(b"HTTP/1.1 400 Bad Request\r\n")
                        .expect("Failed to write to stream");
                    return;
                }
            };

            dbg!(&req);
        });
    }
    println!("Server exited");
}

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug)]
pub struct HttpRequest {
    pub method: Method,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

fn handle_http(stream: &mut TcpStream) -> Result<HttpRequest, Error> {
    let header_chunk = read_header(stream)?;
    let mut lines = header_chunk.lines();

    let Some(Ok(first)) = lines.next() else {
        return Err(Error::InvalidHeader);
    };
    let mut first_line = first.splitn(3, ' ');

    let method = match first_line.next() {
        Some("GET") => Method::Get,
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

    let body = if let Some(content_len) = headers.get(HEADER_CONTENT_LEN) {
        let content_len: usize = content_len.parse().map_err(|_| Error::InvalidHeaderValue {
            header: HEADER_CONTENT_LEN.to_owned(),
        })?;

        dbg!(&content_len);
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

fn read_header(stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
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

fn read_body(stream: &mut TcpStream, size: usize) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0; size];
    let read = stream.read(&mut buf)?;
    if read != size {
        return Err(Error::BodyTooShort);
    }

    Ok(buf)
}
