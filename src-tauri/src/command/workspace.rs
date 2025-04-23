use super::app_config;
use crate::utils::AppError;
use crate::utils::CommandResult;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

const WORKSPACE_FILE_NAME: &str = "workspace.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub description: String,
}

impl Workspace {
    pub fn new(name: &str) -> Self {
        Workspace {
            name: name.to_string().chars().take(256).collect(),
            description: "".to_string().chars().take(256).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceMergeInfo {
    pub id: String,
    pub directory: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("app config error: {0}")]
    AppConfig(#[from] app_config::AppConfigError),
}
async fn create_workspace(directory: &str) -> Result<WorkspaceMergeInfo, WorkspaceError> {
    let name = Path::new(directory).file_name().and_then(|f| f.to_str());
    if name.is_none() {
        return Err(WorkspaceError::App(AppError::new("Invalid directory")));
    }
    let name = name.unwrap();
    let mut workspace = Workspace::new(name);
    let workspace_path = PathBuf::from(directory).join(WORKSPACE_FILE_NAME);
    if workspace_path.exists() {
        let workspace_json = fs::read_to_string(&workspace_path).await?;
        workspace = serde_json::from_str::<Workspace>(&workspace_json)?;
    } else {
        let workspace_json = serde_json::to_string(&workspace).unwrap();
        fs::write(&workspace_path, workspace_json).await?;
    }
    let workspace_result = app_config::add_workspace_to_app_config(directory).await?;
    return Ok(WorkspaceMergeInfo {
        id: workspace_result,
        directory: directory.to_string(),
        name: workspace.name,
        description: workspace.description,
    });
}

async fn load_workspace(workspace_id: &str) -> Result<WorkspaceMergeInfo, WorkspaceError> {
    let app_config = app_config::read_app_config().await?;

    // すでに登録されている場合は登録IDを返す
    for workspace_info in app_config.workspaces.iter() {
        if workspace_info.id == workspace_id {
            let workspace_path =
                PathBuf::from(workspace_info.directory.as_str()).join(WORKSPACE_FILE_NAME);
            if workspace_path.exists() {
                let workspace_json = fs::read_to_string(&workspace_path).await?;
                let workspace = serde_json::from_str::<Workspace>(&workspace_json)?;
                return Ok(WorkspaceMergeInfo {
                    id: workspace_id.to_string(),
                    directory: workspace_info.directory.clone(),
                    name: workspace.name,
                    description: workspace.description,
                });
            }
        }
    }
    return Err(WorkspaceError::App(AppError::new(
        "Failed to find Workspace",
    )));
}

async fn load_workspaces() -> Result<Vec<WorkspaceMergeInfo>, WorkspaceError> {
    let app_config = app_config::read_app_config().await?;

    let mut workspaces = Vec::new();
    for workspace_info in app_config.workspaces.iter() {
        let workspace_path =
            PathBuf::from(workspace_info.directory.as_str()).join(WORKSPACE_FILE_NAME);
        if workspace_path.exists() {
            let workspace_json = fs::read_to_string(&workspace_path).await?;
            let workspace = serde_json::from_str::<Workspace>(&workspace_json)?;
            workspaces.push(WorkspaceMergeInfo {
                id: workspace_info.id.clone(),
                directory: workspace_info.directory.clone(),
                name: workspace.name,
                description: workspace.description,
            });
        }
    }
    return Ok(workspaces);
}

async fn update_workspace(workspace_id: &str, workspace: Workspace) -> Result<(), WorkspaceError> {
    let app_config = app_config::read_app_config().await?;

    for workspace_info in app_config.workspaces.iter() {
        if workspace_info.id == workspace_id {
            let workspace_path =
                PathBuf::from(workspace_info.directory.as_str()).join(WORKSPACE_FILE_NAME);
            let workspace_json = serde_json::to_string(&workspace).unwrap();
            fs::write(&workspace_path, workspace_json).await?;
            return Ok(());
        }
    }
    return Err(WorkspaceError::App(AppError::new(
        "Failed to find Workspace",
    )));
}

pub async fn open_workspace(handle: tauri::AppHandle, id: &str) -> Result<bool, WorkspaceError> {
    let window = if id == "main" {
        tauri::WebviewWindowBuilder::new(
            &handle,
            "main".to_string(),
            tauri::WebviewUrl::App(PathBuf::from("workspaces")),
        )
        .title("formation-docs")
    } else {
        let workspace_info = load_workspace(id).await?;
        let path = Path::new(workspace_info.directory.as_str());
        log::info!("Open: {}", path.to_str().unwrap());
        if !path.exists() {
            return Ok(false);
        }
        tauri::WebviewWindowBuilder::new(
            &handle,
            format!("workspace-{}", id),
            tauri::WebviewUrl::App(PathBuf::from(format!("workspaces/{}", id))),
        )
        .title(workspace_info.name)
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
pub async fn load_workspace_command(
    workspace_id: &str,
) -> Result<CommandResult<WorkspaceMergeInfo>, CommandResult> {
    match load_workspace(workspace_id).await {
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
    workspace_id: &str,
    workspace: Workspace,
) -> Result<CommandResult<()>, CommandResult> {
    match update_workspace(workspace_id, workspace).await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to update Workspace")),
    }
}
