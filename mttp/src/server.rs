use std::{
    collections::HashMap,
    io::Write,
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use crate::{
    consts::CHUNK_END,
    http::{protocol::handle_http, request::HttpRequest, response::HttpResponse, Method},
};

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
