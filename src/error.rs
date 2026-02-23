//! Error types for the ypbank library.
use std::fmt;

/// All errors that can occur during parsing or serialization of transaction records.
#[derive(Debug)]
pub enum BankFormatError {
    /// An IO error occurred while reading or writing.
    Io(std::io::Error),
    /// A CSV parsing error occurred.
    Csv(csv::Error),
    /// A general parse error with a description.
    Parse(String),
    /// The binary data is invalid or corrupted.
    InvalidBinary(String),
}

impl fmt::Display for BankFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BankFormatError::Io(e) => write!(f, "IO error: {}", e),
            BankFormatError::Csv(e) => write!(f, "CSV error: {}", e),
            BankFormatError::Parse(msg) => write!(f, "Parse error: {}", msg),
            BankFormatError::InvalidBinary(msg) => write!(f, "Invalid binary format: {}", msg),
        }
    }
}

impl std::error::Error for BankFormatError {}

impl From<std::io::Error> for BankFormatError {
    fn from(e: std::io::Error) -> Self {
        BankFormatError::Io(e)
    }
}
