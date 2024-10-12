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
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            Error::Io(e) => format!("Io Error: {e}"),
            Error::Empty => format!("A required field was empty"),
            Error::InvalidHeader => format!("A header was invalid"),
            Error::NoUri => format!("No URI was provided"),
            Error::InvalidMethod { recieved } => format!("Invalid method '{recieved}'"),
            Error::InvalidHeaderValue { header } => {
                format!("Header {header} contains an invalid value")
            }
            Error::BodyTooShort { expt, got } => {
                format!("Body too short. Expected {} got {}", expt, got)
            }
            Error::UnsupportedVersion => format!("The specified HTTP versioon is not supported"),
        };

        write!(f, "{}", txt)
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
