use std::fmt::{Display, Formatter, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct AppError {
    message: String,
}

impl AppError {
    pub fn new(message: &str) -> AppError {
        AppError {
            message: message.to_string(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.message)
    }
}
