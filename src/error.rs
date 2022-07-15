use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct DCPError {
    details: String,
}

impl DCPError {
    pub fn new(msg: &str) -> DCPError {
        DCPError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for DCPError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for DCPError {
    fn description(&self) -> &str {
        &self.details
    }
}
