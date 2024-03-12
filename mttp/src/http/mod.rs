pub(crate) mod header;
pub(crate) mod protocol;
pub(crate) mod request;
pub(crate) mod response;

pub use header::*;
pub use request::*;
pub use response::*;

#[derive(Debug, Clone, Copy)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}
