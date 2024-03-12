use std::{
    collections::HashMap,
    io::{self, BufRead, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
};

const CHUNK_END: &[u8; 4] = b"\r\n\r\n";
pub const HEADER_CONTENT_LEN: &str = "Content-Length";
pub const HEADER_COOKIES: &str = "Cookie";

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

#[derive(Debug)]
pub struct Server<State: 'static + Send + Sync> {
    state: Arc<State>,
    get_handlers: HashMap<String, fn(Arc<State>, HttpRequest) -> HttpResponse>,
    post_handlers: HashMap<String, fn(Arc<State>, HttpRequest) -> HttpResponse>,
}

impl<State: 'static + Send + Sync> Server<State> {
    pub fn new(state: State) -> Self {
        Self {
            get_handlers: HashMap::new(),
            post_handlers: HashMap::new(),
            state: Arc::new(state),
        }
    }

    pub fn get(&mut self, route: &str, handler: fn(Arc<State>, HttpRequest) -> HttpResponse) {
        self.get_handlers.insert(route.to_owned(), handler);
    }

    pub fn post(&mut self, route: &str, handler: fn(Arc<State>, HttpRequest) -> HttpResponse) {
        self.post_handlers.insert(route.to_owned(), handler);
    }

    pub fn start(self, addr: SocketAddr) {
        let socket = TcpListener::bind(addr).expect("Failed to start server");

        println!("Listening...");
        while let Ok((mut stream, addr)) = socket.accept() {
            // stream
            //     .set_read_timeout(Some(Duration::from_secs(5)))
            //     .unwrap();
            println!("New connection from: {:?}", &addr);

            let get_handlers = self.get_handlers.clone();
            let post_handlers = self.post_handlers.clone();
            let state = self.state.clone();
            std::thread::spawn(move || {
                let req = handle_http(&mut stream);

                let req = match req {
                    Ok(v) => v,
                    Err(e) => {
                        stream
                            .write_all(b"HTTP/1.1 400 Bad Request\r\n")
                            .expect("Failed to write to stream");
                        println!("ERR: {:?}", e);
                        return;
                    }
                };

                let handler = match req.method {
                    Method::Get => get_handlers.get(&req.uri),
                    Method::Post => post_handlers.get(&req.uri),
                    _ => None,
                };

                if let Some(handler) = handler {
                    let res = handler(state, req);

                    stream
                        .write_all(format!("\r\nHTTP/1.1 {} {}", res.status, res.msg).as_bytes())
                        .expect("Failed to send res");

                    if let Some(body) = res.body {
                        stream.write_all(CHUNK_END).expect("Failed to write to res");
                        stream.write_all(&body).expect("Failed to write to res");
                    }

                    stream.write_all(CHUNK_END).expect("Failed to write to res");
                }
            });
        }
        println!("Server exited");
    }
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
    pub headers: HeaderSet,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct HeaderSet {
    pub values: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
}

impl HeaderSet {
    pub fn content_length(&self) -> Option<usize> {
        if let Some(value) = self.values.get(HEADER_CONTENT_LEN) {
            value.parse().ok()
        } else {
            None
        }
    }

    pub fn cookies(&self) -> HashMap<&str, &str> {
        let Some(cookies_str) = self.values.get(HEADER_COOKIES) else {
            return HashMap::new();
        };

        if let Some(cookies) = cookies_str
            .split("; ")
            .map(|x| x.split_once('='))
            .collect::<Option<HashMap<_, _>>>()
        {
            cookies
        } else {
            HashMap::new()
        }
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub msg: String,
    pub header: HeaderSet,
    pub body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn from_text(status: u16, msg: &str, body: String) -> Self {
        Self {
            status,
            msg: msg.to_owned(),
            header: HeaderSet {
                values: HashMap::new(),
            },
            body: Some(body.into_bytes()),
        }
    }

    pub fn from_bytes(status: u16, msg: &str, body: Vec<u8>) -> Self {
        Self {
            status,
            msg: msg.to_owned(),
            header: HeaderSet {
                values: HashMap::new(),
            },
            body: Some(body),
        }
    }

    pub fn from_status(status: u16, msg: &str) -> Self {
        Self {
            status,
            msg: msg.to_owned(),
            header: HeaderSet {
                values: HashMap::new(),
            },
            body: None,
        }
    }
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
    let headers = HeaderSet { values: headers };

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
