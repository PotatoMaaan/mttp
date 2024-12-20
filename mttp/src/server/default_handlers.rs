use super::{HttpHandlerFunc, HttpResult, RegisteredRoute};
use crate::http::{self, HttpRequest, HttpResponse, Method};
use std::{collections::HashMap, sync::Arc};

pub fn not_found<Hs>(_: Arc<Hs>, _: HttpRequest) -> HttpResult {
    Ok(HttpResponse::not_found())
}

pub fn method_not_allowed<Hs>(_: Arc<Hs>, _: HttpRequest) -> HttpResult {
    Ok(HttpResponse::builder()
        .status(crate::http::StatusCode::MethodNotAllowed)
        .text("The method is not allowed for this resource".to_owned())
        .build())
}

pub fn error(e: Box<dyn std::error::Error>) -> HttpResponse {
    println!("Error returned from handler: {e}");
    HttpResponse::builder()
        .status(crate::http::StatusCode::InternalServerError)
        .text("Oops, something went wrong".to_owned())
        .build()
}

pub fn make_default<S: Clone>(handler: HttpHandlerFunc<S>) -> RegisteredRoute<S> {
    RegisteredRoute {
        handler: super::HandlerType::Http(handler),
        specific_middlewares: Vec::new(),
        method: Method::Get,
        params: HashMap::new(),
    }
}
