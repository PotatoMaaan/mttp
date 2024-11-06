use crate::websocket;

/// All closing codes defined in the websockert RFC
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
#[allow(missing_docs)]
pub enum CloseReason {
    Normal = 1000,
    GoingAway = 1001,
    ProtocolError = 1002,
    UnacceptedData = 1003,
    Reserved = 1004,
    NoStatusCode = 1005,
    ClosedAbnormally = 1006,
    InconsistentData = 1007,
    PolicyViolated = 1008,
    TooBig = 1009,
    MissingExtension = 1010,
    ServerError = 1011,
    TlsFailure = 1015,
}

impl CloseReason {
    fn parse(code: u16) -> Result<Self, websocket::ProtocolError> {
        match code {
            1000 => Ok(CloseReason::Normal),
            1001 => Ok(CloseReason::GoingAway),
            1002 => Ok(CloseReason::ProtocolError),
            1003 => Ok(CloseReason::UnacceptedData),
            1007 => Ok(CloseReason::InconsistentData),
            1008 => Ok(CloseReason::PolicyViolated),
            1009 => Ok(CloseReason::TooBig),
            1010 => Ok(CloseReason::MissingExtension),
            1011 => Ok(CloseReason::ServerError),
            _ => Err(websocket::ProtocolError::InvalidCloseCode(code)),
        }
    }

    /// Gets the raw code
    pub fn code(&self) -> u16 {
        *self as u16
    }
}

///Range the close code falls into
#[derive(Debug, Clone)]
pub enum CodeRange {
    /// Codes defined by the websocket RFC
    Defined(CloseReason),

    /// Codes registered with IANA
    Registered(u16),

    /// Range reserved for private use
    Custom(u16),
}

impl CodeRange {
    /// Gets the raw code
    pub fn code(&self) -> u16 {
        match self {
            CodeRange::Defined(close_reason) => close_reason.code(),
            CodeRange::Registered(code) => *code,
            CodeRange::Custom(code) => *code,
        }
    }

    pub(crate) fn parse(code: u16) -> Result<Self, websocket::ProtocolError> {
        match code {
            1000..=2999 => CloseReason::parse(code).map(Self::Defined),
            3000..=3999 => Ok(Self::Registered(code)),
            4000..=4999 => Ok(Self::Custom(code)),
            _ => Err(websocket::ProtocolError::InvalidCloseCode(code)),
        }
    }
}

/// The contents of a closing Frame
#[derive(Debug, Clone)]
pub struct Close {
    /// The close code can be in different ranges
    pub code: CodeRange,
    /// An optional string describing why the connection was closed
    pub reason: Option<String>,
}

impl Close {
    /// Gets the raw code
    pub fn raw_code(&self) -> u16 {
        self.code.code()
    }
}
