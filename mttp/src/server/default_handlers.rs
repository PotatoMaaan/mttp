use crate::http::{HttpRequest, HttpResponse, Method};
use std::{collections::HashMap, sync::Arc};

use super::{HandlerFunc, RegisteredRoute};

pub fn not_found<Hs>(_: Arc<Hs>, _: HttpRequest) -> crate::Result {
    Ok(HttpResponse::not_found())
}

pub fn method_not_allowed<Hs>(_: Arc<Hs>, _: HttpRequest) -> crate::Result {
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

pub fn make_default<S: Clone>(handler: HandlerFunc<S>) -> RegisteredRoute<S> {
    RegisteredRoute {
        handler,
        specific_middlewares: Vec::new(),
        method: Method::Get,
        params: HashMap::new(),
    }
}
