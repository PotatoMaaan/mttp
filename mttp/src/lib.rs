//! # NOT FOR PRODUCTION USE
//! This implementation is not complete and does not strictly adhear to the HTTP spec!

pub type Result = core::result::Result<HttpResponse, Box<dyn std::error::Error>>;

pub mod consts;
pub mod http;
pub mod url;

mod error;
mod server;

pub use error::Error;
use http::HttpResponse;
pub use server::*;
