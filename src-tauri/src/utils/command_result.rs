use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult<T: Serialize = String> {
    pub success: bool,
    pub value: T,
}

impl<T: Serialize> CommandResult<T> {
    pub fn success(value: T) -> CommandResult<T> {
        CommandResult {
            success: true,
            value: value,
        }
    }
}
impl CommandResult<String> {
    pub fn failed(value: &str) -> CommandResult<String> {
        CommandResult {
            success: false,
            value: value.to_string(),
        }
    }
}
