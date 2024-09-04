//! # NOT FOR PRODUCTION USE
//! This implementation is not complete and does not strictly adhear to the HTTP spec!

pub mod consts;
pub mod http;
pub mod url;

mod error;
mod server;

pub use error::Error;
pub use server::*;
