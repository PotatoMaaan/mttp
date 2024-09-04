use mttp::http::{HttpRequest, HttpResponse};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{
        atomic::{self, AtomicU64},
        Arc,
    },
};

struct State {
    counter: AtomicU64,
}

const WEB_DIR: &str = "web";

fn main() {
    let mut server = mttp::Server::new(State {
        counter: AtomicU64::new(1),
    });

    server.get("/hello", hello);
    server.get("/person/:id/info/:faktenlage/fake", person);
    server.post("/echo", echo);

    server.not_found_handler(fileserver);

    server.start("127.0.0.1:5000".parse().unwrap()).unwrap();
}

fn hello(state: Arc<State>, _: HttpRequest, _: HashMap<String, String>) -> HttpResponse {
    let count = state.counter.fetch_add(1, atomic::Ordering::SeqCst);

    println!("Hello from hello handler");

    HttpResponse::builder()
        .text(format!("Hello {}", count))
        .build()
}

fn person(_: Arc<State>, _: HttpRequest, params: HashMap<String, String>) -> HttpResponse {
    let person_id = params.get("id").expect("handler param not registered");
    let Ok(person_id) = person_id.parse::<u64>() else {
        return HttpResponse::builder()
            .status(mttp::http::StatusCode::BadRequest)
            .text("Invalid person ID format".to_owned())
            .build();
    };

    HttpResponse::builder()
        .text(format!("Hello Person {person_id}"))
        .build()
}

fn echo(_: Arc<State>, req: HttpRequest, _: HashMap<String, String>) -> HttpResponse {
    println!("Hello from echo handler");

    HttpResponse::builder()
        .body(req.body)
        .headers(req.headers)
        .build()
}

fn fileserver(_: Arc<State>, req: HttpRequest, _: HashMap<String, String>) -> HttpResponse {
    let safe_route = req.route.replace("../", "");
    let safe_route = safe_route.strip_prefix('/').unwrap_or(&safe_route);

    let final_path = PathBuf::from(WEB_DIR).join(safe_route);

    match fs::read(final_path) {
        Ok(file) => HttpResponse::builder().bytes(file).build(),
        Err(_) => HttpResponse::not_found(),
    }
}
