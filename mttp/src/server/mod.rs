use routing::{build_dynamic_routes, router};

use crate::http::{
    protocol::{parse_request, write_response},
    request::HttpRequest,
    response::HttpResponse,
    Method, StatusCode,
};
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener},
    sync::{atomic::AtomicI64, Arc},
};

mod default_handlers;
mod routing;

type Handlers<State> = HashMap<String, Handler<Arc<State>>>;

#[derive(Debug)]
pub struct Server<State: 'static + Send + Sync> {
    state: Arc<State>,
    not_found_handler: Option<Handler<Arc<State>>>,
    method_not_allowd_handler: Option<Handler<Arc<State>>>,
    handlers: Handlers<State>,
    thread_counter: Arc<AtomicI64>,
}

type HandlerFunc<S> = fn(S, HttpRequest, HashMap<String, String>) -> HttpResponse;

#[derive(Debug, Clone)]
struct Handler<S: Clone> {
    handler: HandlerFunc<S>,
    method: Method,
    params: HashMap<String, String>,
}

impl<State: 'static + Send + Sync> Server<State> {
    pub fn new(state: State) -> Self {
        Self {
            handlers: HashMap::new(),
            state: Arc::new(state),
            not_found_handler: None,
            method_not_allowd_handler: None,
            thread_counter: Arc::new(AtomicI64::new(0)),
        }
    }

    pub fn get(&mut self, route: &str, handler: HandlerFunc<Arc<State>>) {
        self.handlers.insert(
            route.to_owned(),
            Handler {
                handler,
                method: Method::Get,
                params: HashMap::new(),
            },
        );
    }

    pub fn post(&mut self, route: &str, handler: HandlerFunc<Arc<State>>) {
        self.handlers.insert(
            route.to_owned(),
            Handler {
                handler,
                method: Method::Post,
                params: HashMap::new(),
            },
        );
    }

    pub fn not_found_handler(&mut self, handler: HandlerFunc<Arc<State>>) {
        self.not_found_handler = Some(Handler {
            handler,
            method: Method::Get,
            params: HashMap::new(),
        });
    }

    pub fn method_not_allowd_handler(&mut self, handler: HandlerFunc<Arc<State>>) {
        self.method_not_allowd_handler = Some(Handler {
            handler,
            method: Method::Get,
            params: HashMap::new(),
        });
    }

    pub fn start(self, addr: SocketAddr) -> std::io::Result<()> {
        println!("Binding mttp server to {}", addr);
        let socket = TcpListener::bind(addr)?;

        let dynamic_routes = Arc::new(build_dynamic_routes(self.handlers));
        println!("[mttp] {} routes registered", dynamic_routes.len());

        while let Ok((mut stream, addr)) = socket.accept() {
            let state = self.state.clone();
            let not_found_handler = self.not_found_handler.clone();
            let method_not_allowed_handler = self.method_not_allowd_handler.clone();
            let dynamic_routes = dynamic_routes.clone();

            let thread_id = self
                .thread_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::thread::Builder::new()
                .name(format!("mttp worker thread #{thread_id} for {}", addr))
                .spawn(move || {
                    let parsed_request = parse_request(&mut stream);

                    let response = match parsed_request {
                        Ok(parsed_request) => {
                            let handler = router(
                                &dynamic_routes,
                                not_found_handler,
                                method_not_allowed_handler,
                                &parsed_request,
                            );

                            // Actual handler gets run here
                            (handler.handler)(state, parsed_request, handler.params)
                        }
                        Err(e) => HttpResponse::builder()
                            .status(StatusCode::BadRequest)
                            .text(format!("Error processing HTTP: {}", e))
                            .build(),
                    };

                    write_response(&mut stream, response).expect("Failed to write response");
                })?;
        }

        println!("Stopping server");
        Ok(())
    }
}
