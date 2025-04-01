use crate::utils::CommandResult;
use chrono::{DateTime, NaiveDateTime, Utc};
use dirs::config_local_dir;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};
use tokio::fs;
use uuid::Uuid;

const APP_CONFIG_FILE_NAME: &str = "app_config.json";
pub const APP_CONFIG_DIRECTORY_NAME: &str = "formation-docs";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceInfo {
    pub id: String,
    pub directory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub workspaces: Vec<WorkspaceInfo>,
    pub initialized: bool,
    pub initialized_at: String,
}

impl AppConfig {
    pub fn new() -> Self {
        AppConfig {
            workspaces: vec![],
            initialized: false,
            initialized_at: Utc::now().to_string(),
        }
    }

    pub fn initialized(&self) -> DateTime<Utc> {
        return NaiveDateTime::from_str(self.initialized_at.as_str())
            .unwrap()
            .and_utc();
    }
}

fn app_config_path() -> Option<PathBuf> {
    return match config_local_dir() {
        Some(dir) => Some(
            dir.join(APP_CONFIG_DIRECTORY_NAME)
                .join(APP_CONFIG_FILE_NAME),
        ),
        None => None,
    };
}

#[tauri::command]
pub async fn read_app_config() -> Result<CommandResult<AppConfig>, CommandResult> {
    let app_config_path = match app_config_path() {
        Some(dir) => dir,
        None => {
            return Err(CommandResult::failed(
                "Failed to get local config directory",
            ))
        }
    };
    match app_config_path.exists() {
        true => {
            let app_config_json = match fs::read_to_string(app_config_path).await {
                Ok(json) => json,
                Err(_) => return Err(CommandResult::failed("Failed to read AppConfig")),
            };
            return match serde_json::from_str::<AppConfig>(&app_config_json) {
                Ok(config) => Ok(CommandResult::success(config)),
                Err(_) => Err(CommandResult::failed("Failed to parse AppConfig")),
            };
        }
        false => {
            // Create the app config directory if it doesn't exist
            let app_config = AppConfig::new();
            let app_config_json = match serde_json::to_string(&app_config) {
                Ok(json) => json,
                Err(_) => return Err(CommandResult::failed("Failed to serialize AppConfig")),
            };
            return match fs::write(app_config_path, app_config_json).await {
                Ok(_) => Ok(CommandResult::success(app_config)),
                Err(_) => Err(CommandResult::failed("Failed to write AppConfig")),
            };
        }
    }
}

pub async fn add_workspace_to_app_config(
    workspace_directory: &str,
) -> Result<CommandResult<String>, CommandResult<String>> {
    let app_config = match read_app_config().await {
        Ok(result) => result.value,
        Err(_) => return Err(CommandResult::failed("Failed to read AppConfig")),
    };

    // すでに登録されている場合は登録IDを返す
    for workspace in app_config.workspaces.iter() {
        if workspace.directory == workspace_directory {
            return Ok(CommandResult::success(workspace.id.clone()));
        }
    }

    let workspace_id = Uuid::new_v4().to_string();
    let workspace_info = WorkspaceInfo {
        id: workspace_id.clone(),
        directory: workspace_directory.to_string(),
    };
    let app_config = AppConfig {
        workspaces: app_config
            .workspaces
            .clone()
            .iter()
            .chain([workspace_info].iter())
            .cloned()
            .collect(),
        initialized: app_config.initialized,
        initialized_at: app_config.initialized_at,
    };
    let app_config_json = match serde_json::to_string(&app_config) {
        Ok(json) => json,
        Err(_) => return Err(CommandResult::failed("Failed to serialize AppConfig")),
    };
    let app_config_path = match app_config_path() {
        Some(dir) => dir,
        None => {
            return Err(CommandResult::failed(
                "Failed to get local config directory",
            ))
        }
    };
    return match fs::write(app_config_path, app_config_json).await {
        Ok(_) => Ok(CommandResult::success(workspace_id)),
        Err(_) => Err(CommandResult::failed("Failed to write AppConfig")),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_workspace_from_app_config(
    workspace_id: &str,
) -> Result<CommandResult<()>, CommandResult<String>> {
    let app_config = match read_app_config().await {
        Ok(result) => result.value,
        Err(_) => return Err(CommandResult::failed("Failed to read AppConfig")),
    };
    let workspaces = app_config
        .workspaces
        .iter()
        .filter(|workspace| workspace.id != workspace_id)
        .cloned()
        .collect();
    let app_config = AppConfig {
        workspaces: workspaces,
        initialized: app_config.initialized,
        initialized_at: app_config.initialized_at,
    };
    let app_config_json = match serde_json::to_string(&app_config) {
        Ok(json) => json,
        Err(_) => return Err(CommandResult::failed("Failed to serialize AppConfig")),
    };
    let app_config_path = match app_config_path() {
        Some(dir) => dir,
        None => {
            return Err(CommandResult::failed(
                "Failed to get local config directory",
            ))
        }
    };
    return match fs::write(app_config_path, app_config_json).await {
        Ok(_) => Ok(CommandResult::success(())),
        Err(_) => Err(CommandResult::failed("Failed to write AppConfig")),
    };
}

pub async fn initialized_app_config() -> Result<CommandResult<()>, CommandResult> {
    let app_config = match read_app_config().await {
        Ok(result) => result.value,
        Err(_) => return Err(CommandResult::failed("Failed to read AppConfig")),
    };
    let app_config = AppConfig {
        workspaces: app_config.workspaces,
        initialized: true,
        initialized_at: Utc::now().to_string(),
    };
    let app_config_json = match serde_json::to_string(&app_config) {
        Ok(json) => json,
        Err(_) => return Err(CommandResult::failed("Failed to serialize AppConfig")),
    };
    let app_config_path = match app_config_path() {
        Some(dir) => dir,
        None => {
            return Err(CommandResult::failed(
                "Failed to get local config directory",
            ))
        }
    };
    return match fs::write(app_config_path, app_config_json).await {
        Ok(_) => Ok(CommandResult::success(())),
        Err(_) => Err(CommandResult::failed("Failed to write AppConfig")),
    };
}
