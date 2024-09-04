use crate::http::{HttpRequest, HttpResponse};
use std::{collections::HashMap, sync::Arc};

pub fn not_found<Hs>(_: Arc<Hs>, _: HttpRequest, _: HashMap<String, String>) -> HttpResponse {
    HttpResponse::not_found()
}

pub fn handler_method_not_allowed<Hs>(
    _: Arc<Hs>,
    _: HttpRequest,
    _: HashMap<String, String>,
) -> HttpResponse {
    HttpResponse::builder()
        .status(crate::http::StatusCode::MethodNotAllowed)
        .text("The method is not allowed for this resource".to_owned())
        .build()
}
