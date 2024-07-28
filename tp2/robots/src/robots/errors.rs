use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
pub enum RobotError {
    /// IO error
    FileError(String),
    /// Parsing error
    ParseError(String),
    /// args length is less than 2
    InvalidArguments,
    /// Error when trying to release a flavour
    ReleaseFlavourError,
}

impl From<std::io::Error> for RobotError {
    /// Converts an IO error into a RobotError.
    fn from(err: std::io::Error) -> RobotError {
        RobotError::FileError(err.to_string())
    }
}

impl From<ParseIntError> for RobotError {
    /// Converts a parsing error into a RobotError.
    fn from(err: ParseIntError) -> RobotError {
        RobotError::ParseError(err.to_string())
    }
}
