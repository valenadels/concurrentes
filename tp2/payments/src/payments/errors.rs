use std::num::ParseIntError;

/// Represents an error that can occur while running the Payments program.
#[derive(Debug)]
pub enum PaymentError {
    /// IO error.
    FileError(String),
    /// Parsing error.
    ParseError(String),
    /// Invalid message ID (1,2,3,4,5).
    InvalidMessageId,
    /// Actor error when getting orders
    ActorError(String),
}

impl From<std::io::Error> for PaymentError {
    /// Converts an IO error into a PaymentError.
    fn from(err: std::io::Error) -> PaymentError {
        PaymentError::FileError(err.to_string())
    }
}

impl From<ParseIntError> for PaymentError {
    /// Converts a parsing error into a PaymentError.
    fn from(err: ParseIntError) -> PaymentError {
        PaymentError::ParseError(err.to_string())
    }
}
