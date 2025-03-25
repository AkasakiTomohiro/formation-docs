use crate::utils::CommandResult;
use dirs::config_local_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    id: String,
    directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    workspaces: Vec<WorkspaceInfo>,
}

const APP_CONFIG_DIRECTORY_NAME: &str = "formation-docs";
const APP_CONFIG_FILE_NAME: &str = "app_config.json";

#[tauri::command]
pub async fn read_app_config() -> Result<CommandResult<AppConfig>, CommandResult> {
    let config_local_dir = match config_local_dir() {
        Some(dir) => dir,
        None => {
            return Err(CommandResult::failed(
                "Failed to get local config directory",
            ))
        }
    };
    let app_config_path = config_local_dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(APP_CONFIG_FILE_NAME);
    match app_config_path.exists() {
        true => {
            let app_config_json = match std::fs::read_to_string(app_config_path) {
                Ok(json) => json,
                Err(_) => return Err(CommandResult::failed("Failed to read AppConfig")),
            };
            match serde_json::from_str(&app_config_json) {
                Ok(config) => return Ok(CommandResult::success(config)),
                Err(_) => return Err(CommandResult::failed("Failed to parse AppConfig")),
            };
        }
        false => {
            // Create the app config directory if it doesn't exist
            let app_config = AppConfig { workspaces: vec![] };
            let app_config_json = match serde_json::to_string(&app_config) {
                Ok(json) => json,
                Err(_) => return Err(CommandResult::failed("Failed to serialize AppConfig")),
            };
            match std::fs::write(app_config_path, app_config_json) {
                Ok(_) => return Ok(CommandResult::success(app_config)),
                Err(_) => return Err(CommandResult::failed("Failed to write AppConfig")),
            }
        }
    }
}
