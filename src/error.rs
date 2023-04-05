use std::{error::Error, fmt::Display, fmt::Formatter};

#[derive(Debug, PartialEq)]
pub struct MatchSpecError {
    pub message: String,
}

impl Error for MatchSpecError {}

impl Display for MatchSpecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
