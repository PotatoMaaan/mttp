use crate::http::{
    protocol::{parse_request, write_response},
    request::HttpRequest,
    response::HttpResponse,
    Method, StatusCode,
};
use routing::{build_dynamic_routes, router};
use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener},
    sync::{atomic::AtomicI64, Arc},
};

mod default_handlers;
mod public_funcs;
mod routing;

#[derive(Debug)]
pub struct Server<State: 'static + Send + Sync> {
    state: Arc<State>,
    not_found_handler: RegisteredRoute<Arc<State>>,
    method_not_allowd_handler: RegisteredRoute<Arc<State>>,
    error_handler: fn(error: Box<dyn std::error::Error>) -> HttpResponse,
    handlers: Handlers<State>,
    thread_counter: Arc<AtomicI64>,
    middlewares: Vec<MiddlewareFunc<Arc<State>>>,
    inspector: fn(&HttpResponse),
}

type Handlers<State> = HashMap<String, RegisteredRoute<Arc<State>>>;
type HandlerFunc<S> = fn(S, HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>>;
type MiddlewareFunc<S> = fn(S, &mut HttpRequest) -> MiddlewareResult;

#[derive(Debug, Clone)]
struct RegisteredRoute<S: Clone> {
    handler: HandlerFunc<S>,
    specific_middlewares: Vec<MiddlewareFunc<S>>,
    method: Method,
    params: HashMap<String, String>,
}

#[derive(Debug)]
pub enum MiddlewareResult {
    Continue,
    Abort(HttpResponse),
}

impl<State: 'static + Send + Sync> Server<State> {
    pub fn start(self, addr: SocketAddr) -> std::io::Result<()> {
        println!("Binding mttp server to http://{}", addr);
        let socket = TcpListener::bind(addr)?;

        let dynamic_routes = Arc::new(build_dynamic_routes(self.handlers));
        println!("[mttp] {} routes registered", dynamic_routes.len());

        while let Ok((mut stream, addr)) = socket.accept() {
            let state = self.state.clone();
            let not_found_handler = self.not_found_handler.clone();
            let method_not_allowed_handler = self.method_not_allowd_handler.clone();
            let dynamic_routes = dynamic_routes.clone();
            let middlewares = self.middlewares.clone();
            let error_handler = self.error_handler.clone();

            let thread_id = self
                .thread_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            std::thread::Builder::new()
                .name(format!("mttp worker thread #{thread_id} for {}", addr))
                .spawn(move || {
                    let parsed_request = parse_request(&mut stream);
                    let handler_attempt = match parsed_request {
                        Ok(mut parsed_request) => {
                            let handler = router(
                                &dynamic_routes,
                                not_found_handler,
                                method_not_allowed_handler,
                                &parsed_request,
                            );

                            let mut middlewares = middlewares;
                            middlewares.extend(handler.specific_middlewares);

                            parsed_request.params.extend(handler.params);

                            if let Some(abort) = middlewares
                                .into_iter()
                                .map(|middleware| middleware(state.clone(), &mut parsed_request))
                                .find_map(|x| match x {
                                    MiddlewareResult::Continue => None,
                                    MiddlewareResult::Abort(abort) => Some(abort),
                                })
                            {
                                Ok(abort)
                            } else {
                                // Actual handler gets run here
                                (handler.handler)(state.clone(), parsed_request)
                            }
                        }
                        Err(e) => Ok(HttpResponse::builder()
                            .status(StatusCode::BadRequest)
                            .text(format!("Error processing HTTP: {}", e))
                            .build()),
                    };

                    let final_response = match handler_attempt {
                        Ok(v) => v,
                        Err(e) => error_handler(e),
                    };

                    (self.inspector)(&final_response);
                    write_response(&mut stream, final_response).expect("Failed to write response");
                })?;
        }

        println!("Stopping server");
        Ok(())
    }
}
