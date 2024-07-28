use std::convert::Infallible;
use std::num::ParseIntError;
use tokio::sync::mpsc::error::SendError;

/// Represents an error that can occur while running the screens program.
#[derive(Debug)]
pub enum ScreenError {
    /// IO error.
    FileError(String),
    /// Parsing error.
    ParseError(String),
    /// Port not found error.
    PortNotFound,
    /// TCP connection error that occurs when the screens cannot connect to any checkout gateway.
    ControllerConnectionError,
    /// Channel error.
    ChannelError(String),
    /// Message not supported error. This error occurs when the message received is not supported.
    MessageNotSupported,
}

impl From<std::io::Error> for ScreenError {
    /// Converts an IO error into a ScreenError.
    fn from(err: std::io::Error) -> ScreenError {
        ScreenError::FileError(err.to_string())
    }
}

impl From<serde_json::Error> for ScreenError {
    /// Converts a serde json error into a ScreenError.
    fn from(err: serde_json::Error) -> ScreenError {
        ScreenError::FileError(err.to_string())
    }
}

impl From<ParseIntError> for ScreenError {
    /// Converts a parsing error into a ScreenError.
    fn from(err: ParseIntError) -> ScreenError {
        ScreenError::ParseError(err.to_string())
    }
}

impl From<Infallible> for ScreenError {
    /// Converts a infallible error into a ScreenError.
    fn from(err: Infallible) -> ScreenError {
        ScreenError::ParseError(err.to_string())
    }
}

impl From<SendError<u16>> for ScreenError {
    /// Converts a tokio SendError error into a ScreenError.
    fn from(err: SendError<u16>) -> ScreenError {
        ScreenError::ChannelError(err.to_string())
    }
}
