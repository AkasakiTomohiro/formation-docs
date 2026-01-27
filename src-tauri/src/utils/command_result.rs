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

#[cfg(test)]
#[coverage(off)]
mod tests {
    use super::*;

    #[test]
    fn command_result_success() {
        // ######### 準備 #########
        let value = "test value".to_string();

        // ######### 実行 #########
        let result = CommandResult::success(value.clone());

        // ######### 検証 #########
        assert_eq!(result.success, true);
        assert_eq!(result.value, value);
    }

    #[test]
    fn command_result_failed() {
        // ######### 準備 #########
        let value = "error occurred";

        // ######### 実行 #########
        let result = CommandResult::failed(value);

        // ######### 検証 #########
        assert_eq!(result.success, false);
        assert_eq!(result.value, value.to_string());
    }
}
