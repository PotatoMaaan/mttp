use mttp::http::{HttpRequest, HttpResponse, StatusCode};
use std::sync::{
    atomic::{self, AtomicU64},
    Arc,
};

struct State {
    counter: AtomicU64,
}

fn main() {
    let mut server = mttp::Server::new(State {
        counter: AtomicU64::new(0),
    });

    server.get("/hello", hello);
    server.post("/echo", echo);

    server.not_found_handler(|_, _| {
        HttpResponse::builder()
            .status(StatusCode::NotFound)
            .text("You picked the wrong house fool!".to_owned())
            .build()
    });

    server.start("127.0.0.1:5000".parse().unwrap());
}

fn hello(state: Arc<State>, _: HttpRequest) -> HttpResponse {
    let count = state.counter.fetch_add(1, atomic::Ordering::SeqCst);

    println!("Hello from hello handler");

    HttpResponse::builder()
        .text(format!("Hello {}", count))
        .build()
}

fn echo(_: Arc<State>, req: HttpRequest) -> HttpResponse {
    println!("Hello from echo handler");

    HttpResponse::builder()
        .body(req.body)
        .headers(req.headers)
        .build()
}
