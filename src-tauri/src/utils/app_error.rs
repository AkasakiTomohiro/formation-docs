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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn app_error_new() {
        // ######### 準備 #########
        let error_message = "error message;";

        // ######### 実行 #########
        let app_error = AppError::new(error_message);

        // ######### 検証 #########
        assert_eq!(app_error.to_string(), error_message);
    }

    #[test]
    fn app_error_display() {
        // ######### 準備 #########
        let error_message = "test error message";
        let app_error = AppError::new(error_message);

        // ######### 実行 #########
        let formatted = format!("Error: {}", app_error);

        // ######### 検証 #########
        assert_eq!(formatted, "Error: test error message");
    }
}
