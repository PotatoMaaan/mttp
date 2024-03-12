use crate::http::{HttpRequest, HttpResponse};
use std::sync::Arc;

pub fn not_found<Hs>(_: Arc<Hs>, _: HttpRequest) -> HttpResponse {
    HttpResponse::builder()
        .status(crate::http::StatusCode::NotFound)
        .text("The requested resource was not found on the server".to_owned())
        .build()
}

pub fn handler_method_not_allowed<Hs>(_: Arc<Hs>, _: HttpRequest) -> HttpResponse {
    HttpResponse::builder()
        .status(crate::http::StatusCode::MethodNotAllowed)
        .text("The method is not allowed for this resource".to_owned())
        .build()
}
