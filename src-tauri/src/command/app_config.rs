use crate::utils::AppError;
use crate::utils::CommandResult;
use chrono::Utc;
use dirs::config_local_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use uuid::Uuid;

const APP_CONFIG_FILE_NAME: &str = "app_config.json";
pub const APP_CONFIG_DIRECTORY_NAME: &str = "formation-docs";

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    // workspace_id: workspace_directoryのマッピング
    pub workspaces: HashMap<String, String>,
    pub initialized: bool,
    pub initialized_at: String,
    pub aws_cli_commit_hash: Option<String>,
}
pub struct AppConfigUpdate {
    pub workspaces: Option<HashMap<String, String>>,
    pub initialized: Option<bool>,
    pub initialized_at: Option<String>,
    pub aws_cli_commit_hash: Option<Option<String>>,
}

impl AppConfig {
    pub fn new() -> Self {
        AppConfig {
            workspaces: HashMap::new(),
            initialized: false,
            initialized_at: Utc::now().to_string(),
            aws_cli_commit_hash: None,
        }
    }
}

fn app_config_path() -> Result<PathBuf, AppConfigError> {
    let dir = config_local_dir().ok_or(AppConfigError::App(AppError::new(
        "Failed to get local config directory",
    )))?;
    return Ok(dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(APP_CONFIG_FILE_NAME));
}

#[derive(Debug, Error)]
pub enum AppConfigError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub async fn read_app_config() -> Result<AppConfig, AppConfigError> {
    let app_config_path = app_config_path()?;
    match app_config_path.exists() {
        true => {
            let app_config_json = fs::read_to_string(app_config_path).await?;
            return Ok(serde_json::from_str::<AppConfig>(&app_config_json)?);
        }
        false => {
            // Create the app config directory if it doesn't exist
            let app_config = AppConfig::new();
            let app_config_json = serde_json::to_string(&app_config)?;
            fs::write(app_config_path, app_config_json).await?;
            return Ok(app_config);
        }
    }
}

pub async fn add_workspace_to_app_config(
    workspace_directory: &str,
) -> Result<String, AppConfigError> {
    let mut app_config = read_app_config().await?;

    // すでに登録されている場合は登録IDを返す
    for (id, directory) in app_config.workspaces.iter() {
        if directory == workspace_directory {
            return Ok(id.clone());
        }
    }

    let workspace_id = Uuid::new_v4().to_string();
    app_config.workspaces.insert(
        String::from(workspace_id.clone()),
        workspace_directory.to_string(),
    );
    save_app_config(AppConfigUpdate {
        workspaces: Some(app_config.workspaces),
        initialized: None,
        initialized_at: None,
        aws_cli_commit_hash: None,
    })
    .await?;
    return Ok(workspace_id);
}

pub async fn delete_workspace_from_app_config(workspace_id: &str) -> Result<(), AppConfigError> {
    let mut app_config = read_app_config().await?;
    app_config.workspaces.remove(workspace_id);
    save_app_config(AppConfigUpdate {
        workspaces: Some(app_config.workspaces),
        initialized: None,
        initialized_at: None,
        aws_cli_commit_hash: None,
    })
    .await?;
    return Ok(());
}

pub async fn save_app_config(update_config: AppConfigUpdate) -> Result<(), AppConfigError> {
    let app_config = read_app_config().await?;
    let new_app_config = AppConfig {
        workspaces: update_config.workspaces.unwrap_or(app_config.workspaces),
        initialized: update_config.initialized.unwrap_or(app_config.initialized),
        initialized_at: update_config
            .initialized_at
            .unwrap_or(app_config.initialized_at),
        aws_cli_commit_hash: update_config
            .aws_cli_commit_hash
            .unwrap_or(app_config.aws_cli_commit_hash),
    };
    let app_config_json = serde_json::to_string(&new_app_config)?;
    let app_config_path = app_config_path()?;
    fs::write(app_config_path, app_config_json).await?;
    return Ok(());
}

#[tauri::command]
pub async fn read_app_config_command() -> Result<CommandResult<AppConfig>, CommandResult> {
    return match read_app_config().await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to read AppConfig")),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_workspace_from_app_config_command(
    workspace_id: &str,
) -> Result<CommandResult<()>, CommandResult<String>> {
    return match delete_workspace_from_app_config(workspace_id).await {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
