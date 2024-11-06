use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Empty,
    InvalidHeader,
    NoUri,
    InvalidMethod { recieved: String },
    InvalidHeaderValue { header: String },
    BodyTooShort { expt: usize, got: usize },
    UnsupportedVersion,
    MissingOrInvalidWebsocketHeader { header: &'static str },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "Io Error: {e}"),
            Error::Empty => write!(f, "A required field was empty"),
            Error::InvalidHeader => write!(f, "A header was invalid"),
            Error::NoUri => write!(f, "No URI was provided"),
            Error::InvalidMethod { recieved } => write!(f, "Invalid method '{recieved}'"),
            Error::InvalidHeaderValue { header } => {
                write!(f, "Header {header} contains an invalid value")
            }
            Error::BodyTooShort { expt, got } => {
                write!(f, "Body too short. Expected {} got {}", expt, got)
            }
            Error::UnsupportedVersion => write!(f, "The specified HTTP version is not supported"),
            Error::MissingOrInvalidWebsocketHeader { header } => {
                write!(
                    f,
                    "Missing or invalid header for websocket upgrade: {header}"
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}
