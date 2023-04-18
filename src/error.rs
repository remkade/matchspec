use pyo3::PyErr;
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

impl From<MatchSpecError> for PyErr {
    fn from(value: MatchSpecError) -> Self {
        pyo3::exceptions::PyValueError::new_err(value.message)
    }
}
