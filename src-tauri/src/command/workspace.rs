use super::super::config::app_config;
use crate::config::workspace_config::read_workspace_config;
use crate::config::workspace_config::workspace_config_path;
use crate::config::workspace_config::WorkspaceConfig;
use crate::config::workspace_config::WorkspaceConfigError;
use crate::utils::get_window_state;
use crate::utils::set_window_state;
use crate::utils::AppError;
use crate::utils::CommandResult;
use crate::utils::WindowState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceMergeInfo {
    pub id: String,
    pub directory: String,
    pub name: String,
    pub description: String,
    pub stacks: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub stacks: Option<HashMap<String, String>>,
}

#[derive(Debug, Error)]
pub enum WorkspaceCommandError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("app config error: {0}")]
    AppConfig(#[from] app_config::AppConfigError),
    #[error("app config error: {0}")]
    AppConfigCommand(#[from] super::app_config::AppConfigCommandError),
    #[error("workspace config error: {0}")]
    WorkspaceConfig(#[from] WorkspaceConfigError),
}
async fn create_workspace(directory: &str) -> Result<WorkspaceMergeInfo, WorkspaceCommandError> {
    let workspace = read_workspace_config(directory).await?;
    let workspace_result = super::app_config::add_workspace_to_app_config(directory).await?;
    return Ok(WorkspaceMergeInfo {
        id: workspace_result,
        directory: directory.to_string(),
        name: workspace.name,
        description: workspace.description,
        stacks: workspace.stacks.clone(),
    });
}

async fn load_workspace_merge_info(
    workspace_id: &str,
) -> Result<WorkspaceMergeInfo, WorkspaceCommandError> {
    let app_config = app_config::read_app_config().await?;

    // すでに登録されている場合は登録IDを返す
    let workspace_directory = app_config.workspaces.get(workspace_id);
    if workspace_directory.is_some() {
        let workspace_path = workspace_config_path(workspace_directory.unwrap())?;
        if workspace_path.exists() {
            let workspace_json = fs::read_to_string(&workspace_path).await?;
            let workspace = serde_json::from_str::<WorkspaceConfig>(&workspace_json)?;
            return Ok(WorkspaceMergeInfo {
                id: workspace_id.to_string(),
                directory: workspace_directory.unwrap().to_string(),
                name: workspace.name,
                description: workspace.description,
                stacks: workspace.stacks.clone(),
            });
        }
    }
    return Err(WorkspaceCommandError::App(AppError::new(
        "Failed to find Workspace",
    )));
}

async fn load_workspaces() -> Result<Vec<WorkspaceMergeInfo>, WorkspaceCommandError> {
    let app_config = app_config::read_app_config().await?;

    let mut workspaces = Vec::new();
    for (id, directory) in app_config.workspaces.iter() {
        let workspace = read_workspace_config(directory).await?;
        workspaces.push(WorkspaceMergeInfo {
            id: id.to_string(),
            directory: directory.to_string(),
            name: workspace.name,
            description: workspace.description,
            stacks: workspace.stacks.clone(),
        });
    }
    return Ok(workspaces);
}

pub async fn update_workspace(
    workspace_directory: &str,
    update_config: WorkspaceUpdate,
) -> Result<(), WorkspaceCommandError> {
    let workspace = read_workspace_config(workspace_directory).await?;
    let new_workspace = WorkspaceConfig {
        version: workspace.version,
        name: update_config.name.unwrap_or(workspace.name),
        description: update_config.description.unwrap_or(workspace.description),
        stacks: update_config.stacks.unwrap_or(workspace.stacks),
    };

    let workspace_path = workspace_config_path(workspace_directory)?;
    let workspace_json = serde_json::to_string(&new_workspace).unwrap();
    fs::write(&workspace_path, workspace_json).await?;
    return Ok(());
}

pub async fn open_workspace(
    handle: tauri::AppHandle,
    id: &str,
) -> Result<bool, WorkspaceCommandError> {
    let window = if id == "main" {
        tauri::WebviewWindowBuilder::new(
            &handle,
            "main".to_string(),
            tauri::WebviewUrl::App(PathBuf::from("workspaces")),
        )
        .title("formation-docs")
    } else {
        let workspace_info = load_workspace_merge_info(id).await?;
        let path = Path::new(workspace_info.directory.as_str());
        log::info!("Open: {}", path.to_str().unwrap());
        if !path.exists() {
            return Ok(false);
        }
        let window_id = format!("workspace-{}", id);
        set_window_state(
            window_id.clone(),
            WindowState {
                workspace_directory: path.to_str().unwrap().to_string(),
            },
        );
        tauri::WebviewWindowBuilder::new(
            &handle,
            window_id,
            tauri::WebviewUrl::App(PathBuf::from(format!("workspaces/{}", id))),
        )
        .title(workspace_info.name)
        .inner_size(1200.0, 900.0)
    }
    .build()
    .expect("failed to create new window");
    window.show().expect("failed to show window");
    return Ok(true);
}

#[tauri::command]
pub async fn create_workspace_command(
    directory: &str,
) -> Result<CommandResult<WorkspaceMergeInfo>, CommandResult> {
    match create_workspace(directory).await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to create Workspace")),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn load_workspace_merge_info_command(
    workspace_id: &str,
) -> Result<CommandResult<WorkspaceMergeInfo>, CommandResult> {
    match load_workspace_merge_info(workspace_id).await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to load Workspace")),
    }
}

#[tauri::command]
pub async fn load_workspaces_command(
) -> Result<CommandResult<Vec<WorkspaceMergeInfo>>, CommandResult> {
    match load_workspaces().await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to load Workspaces")),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_workspace_command(
    window: tauri::Window,
    workspace: WorkspaceUpdate,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    match update_workspace(window_state.workspace_directory.as_str(), workspace).await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to update Workspace")),
    }
}
