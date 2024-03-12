use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Empty,
    InvalidHeader,
    NoUri,
    InvalidMethod { recieved: String },
    InvalidHeaderValue { header: String },
    TooLong { size: usize },
    BodyTooLong,
    BodyTooShort,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}
