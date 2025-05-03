use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

/// An error we propagate in the module
#[derive(Debug, PartialEq)]
pub struct MahjongError {
    /// String message
    pub message: String,
}

impl MahjongError {
    /// Create a new MahjongError
    pub fn new(message: &str) -> Self {
        MahjongError {
            message: message.to_string(),
        }
    }
}

impl Display for MahjongError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "MahjongError : {}", self.message)
    }
}

impl Error for MahjongError {
    fn description(&self) -> &str {
        &self.message
    }
}
