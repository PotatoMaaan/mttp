use mttp::{
    http::{HttpRequest, HttpResponse},
    MiddlewareResult,
};
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
    users: HashMap<String, String>,
}

const WEB_DIR: &str = "web";

fn main() {
    let mut server = mttp::Server::new(State {
        counter: AtomicU64::new(1),
        users: HashMap::from([(String::from("abc123"), String::from("user1"))]),
    });

    server.get("/hello", hello, vec![]);
    server.get("/only/with/auth", only_with_auth, vec![mw_auth]);
    server.get("/person/:id/info", person, vec![]);
    server.post("/echo", echo, vec![]);

    server.middleware(mw_log);

    server.error_handler(error_handler);
    server.not_found_handler(fileserver);
    server.inspector(inspector);

    server.start("127.0.0.1:5000".parse().unwrap()).unwrap();
}

fn error_handler(e: Box<dyn std::error::Error>) -> HttpResponse {
    println!("handler failed: {e}");
    HttpResponse::builder()
        .text("Something went wrong".to_owned())
        .status(mttp::http::StatusCode::InternalServerError)
        .build()
}

// gets run after all handlers have run.
// can be used to inspect the final response before it gets sent to the client
fn inspector(_res: &HttpResponse) {}

// Simply logs all incoming requests
fn mw_log(_: Arc<State>, req: &mut HttpRequest) -> MiddlewareResult {
    println!("Got request: {}", req.route);
    MiddlewareResult::Continue
}

// Ensures an endpoint is protected by auth
fn mw_auth(state: Arc<State>, req: &mut HttpRequest) -> MiddlewareResult {
    if let Some(token) = req.headers.values.get("auth") {
        if let Some(username) = state.users.get(token) {
            println!("passing {username} with token {token}");
            req.params.insert("_username".to_owned(), username.clone());
            return MiddlewareResult::Continue;
        }
    };
    MiddlewareResult::Abort(
        HttpResponse::builder()
            .status(mttp::http::StatusCode::Forbidden)
            .text("Not logged in".to_owned())
            .build(),
    )
}

// A basic handler showing how many users have already visited
fn hello(state: Arc<State>, _: HttpRequest) -> mttp::Result {
    let count = state.counter.fetch_add(1, atomic::Ordering::SeqCst);

    println!("Hello from hello handler");

    Ok(HttpResponse::builder()
        .text(format!("Hello {}", count))
        .build())
}

// This handler is only accessable when the user is logged in
fn only_with_auth(_: Arc<State>, req: HttpRequest) -> mttp::Result {
    let username = req
        .params
        .get("_username")
        .expect("Username not registered in mw");

    Ok(HttpResponse::builder()
        .text(format!("Requsted by user: {}", username))
        .build())
}

// Demo on how to get parameters from a route
fn person(_: Arc<State>, req: HttpRequest) -> mttp::Result {
    let person_id = req.params.get("id").expect("handler param not registered");
    let Ok(person_id) = person_id.parse::<u64>() else {
        return Ok(HttpResponse::builder()
            .status(mttp::http::StatusCode::BadRequest)
            .text("Invalid person ID format".to_owned())
            .build());
    };

    Ok(HttpResponse::builder()
        .text(format!("Hello Person {person_id}"))
        .build())
}

// Returns body and headers to the requester
fn echo(_: Arc<State>, req: HttpRequest) -> mttp::Result {
    println!("Hello from echo handler");

    Ok(HttpResponse::builder()
        .body(req.body)
        .headers(req.headers)
        .build())
}

// Fileserver serving as a fallback
fn fileserver(_: Arc<State>, req: HttpRequest) -> mttp::Result {
    let safe_route = req.raw_route.replace("../", "");
    let safe_route = safe_route.strip_prefix('/').unwrap_or(&safe_route);

    let safe_route = if safe_route.is_empty() {
        "index.html"
    } else {
        safe_route
    };

    let final_path = PathBuf::from(WEB_DIR).join(safe_route);

    match fs::read(final_path) {
        Ok(file) => Ok(HttpResponse::builder().bytes(file).build()),
        Err(_) => Ok(HttpResponse::not_found()),
    }
}
