use std::{
    net::SocketAddrV4,
    str::FromStr,
    sync::{atomic::AtomicU64, Arc},
};

use mttp::{HttpRequest, HttpResponse};

struct State {
    counter: AtomicU64,
}

fn main() {
    let mut server = mttp::Server::new(State {
        counter: AtomicU64::new(0),
    });

    server.get("/hello", hello);
    server.post("/echo", echo);

    server.start(std::net::SocketAddr::V4(
        SocketAddrV4::from_str("127.0.0.1:5000").unwrap(),
    ));
}

fn hello(state: Arc<State>, req: HttpRequest) -> HttpResponse {
    let new_count = state
        .counter
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    dbg!(&req.headers.cookies());

    println!("HANDLER HELLO");

    HttpResponse::from_text(200, "Ok", format!("Hello {}", new_count))
}

fn echo(state: Arc<State>, req: HttpRequest) -> HttpResponse {
    println!("HANDLER ECHO");

    if let Some(body) = req.body {
        HttpResponse::from_bytes(200, "Ok", body)
    } else {
        HttpResponse::from_bytes(200, "Ok", Vec::new())
    }
}
