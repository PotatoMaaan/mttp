use crate::http::{
    protocol::{parse_request, write_response},
    request::HttpRequest,
    response::HttpResponse,
    Method, StatusCode,
};
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

mod default_handlers;

#[derive(Debug)]
pub struct Server<State: 'static + Send + Sync> {
    state: Arc<State>,
    not_found_handler: Option<Handler<Arc<State>>>,
    method_not_allowd_handler: Option<Handler<Arc<State>>>,
    handlers: HashMap<String, Handler<Arc<State>>>,
}

#[derive(Debug, Clone)]
struct Handler<S: Clone> {
    handler: fn(S, HttpRequest) -> HttpResponse,
    method: Method,
}

impl<State: 'static + Send + Sync> Server<State> {
    pub fn new(state: State) -> Self {
        Self {
            handlers: HashMap::new(),
            state: Arc::new(state),
            not_found_handler: None,
            method_not_allowd_handler: None,
        }
    }

    pub fn get(&mut self, route: &str, handler: fn(Arc<State>, HttpRequest) -> HttpResponse) {
        self.handlers.insert(
            route.to_owned(),
            Handler {
                handler,
                method: Method::Get,
            },
        );
    }

    pub fn post(&mut self, route: &str, handler: fn(Arc<State>, HttpRequest) -> HttpResponse) {
        self.handlers.insert(
            route.to_owned(),
            Handler {
                handler,
                method: Method::Post,
            },
        );
    }

    pub fn not_found_handler(&mut self, handler: fn(Arc<State>, HttpRequest) -> HttpResponse) {
        self.not_found_handler = Some(Handler {
            handler,
            method: Method::Get,
        });
    }

    pub fn method_not_allowd_handler(
        &mut self,
        handler: fn(Arc<State>, HttpRequest) -> HttpResponse,
    ) {
        self.method_not_allowd_handler = Some(Handler {
            handler,
            method: Method::Get,
        });
    }

    pub fn start(self, addr: SocketAddr) {
        let socket = TcpListener::bind(addr).expect("Failed to start server");

        while let Ok((mut stream, addr)) = socket.accept() {
            let handlers = self.handlers.clone();
            let state = self.state.clone();
            let not_found_handler = self.not_found_handler.clone();
            let method_not_allowed_handler = self.method_not_allowd_handler.clone();

            std::thread::Builder::new()
                .name(format!("mttp worker for {}", addr))
                .spawn(move || {
                    let parsed_request = parse_request(&mut stream);

                    let response = match parsed_request {
                        Ok(parsed_request) => {
                            let not_found_handler = not_found_handler.unwrap_or(Handler {
                                handler: default_handlers::not_found::<State>,
                                method: Method::Get,
                            });

                            let method_not_allowed_handler =
                                method_not_allowed_handler.unwrap_or(Handler {
                                    handler: default_handlers::handler_method_not_allowed::<State>,
                                    method: Method::Get,
                                });

                            let handler = if let Some(handler) = handlers.get(&parsed_request.route)
                            {
                                if parsed_request.method == handler.method {
                                    handler
                                } else {
                                    &method_not_allowed_handler
                                }
                            } else {
                                &not_found_handler
                            };

                            // Actual handler gets run here
                            (handler.handler)(state, parsed_request)
                        }
                        Err(e) => HttpResponse::builder()
                            .status(StatusCode::BadRequest)
                            .text(format!("Error processing HTTP: {}", e))
                            .build(),
                    };

                    write_response(&mut stream, response).expect("Failed to write response");
                })
                .expect("Failed to spawn worker thread");
        }
    }
}
