use crate::config::app_config::app_config_path;
use crate::config::app_config::read_app_config;
use crate::config::app_config::AppConfig;
use crate::config::app_config::AppConfigError;
use crate::utils::AppError;
use crate::utils::CommandResult;
use std::collections::HashMap;
use thiserror::Error;
use tokio::fs;
use uuid::Uuid;

pub struct AppConfigUpdate {
    pub workspaces: Option<HashMap<String, String>>,
    pub initialized: Option<bool>,
    pub initialized_at: Option<String>,
}

#[derive(Debug, Error)]
pub enum AppConfigCommandError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("app config error: {0}")]
    AppConfig(#[from] AppConfigError),
}

pub async fn add_workspace_to_app_config(
    workspace_directory: &str,
) -> Result<String, AppConfigCommandError> {
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
    })
    .await?;
    return Ok(workspace_id);
}

pub async fn delete_workspace_from_app_config(
    workspace_id: &str,
) -> Result<(), AppConfigCommandError> {
    let mut app_config = read_app_config().await?;
    app_config.workspaces.remove(workspace_id);
    save_app_config(AppConfigUpdate {
        workspaces: Some(app_config.workspaces),
        initialized: None,
        initialized_at: None,
    })
    .await?;
    return Ok(());
}

pub async fn save_app_config(update_config: AppConfigUpdate) -> Result<(), AppConfigCommandError> {
    let app_config = read_app_config().await?;
    let new_app_config = AppConfig {
        version: app_config.version,
        workspaces: update_config.workspaces.unwrap_or(app_config.workspaces),
        initialized: update_config.initialized.unwrap_or(app_config.initialized),
        initialized_at: update_config
            .initialized_at
            .unwrap_or(app_config.initialized_at),
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
