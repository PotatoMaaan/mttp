//! # NOT FOR PRODUCTION USE
//! This implementation is not complete and does not strictly adhear to the HTTP spec!

pub type Result = core::result::Result<HttpResponse, Box<dyn std::error::Error>>;

pub mod consts;
pub mod http;
pub mod websocket;

mod error;
mod server;
mod url;

pub use error::Error;
use http::HttpResponse;
pub use server::*;
