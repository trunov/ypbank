use std::fmt;

#[derive(Debug)]
pub enum BankFormatError {
    Io(std::io::Error),
    Csv(csv::Error),
    Parse(String),
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