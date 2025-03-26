use super::app_config;
use crate::utils::CommandResult;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
            name: name.to_string(),
            description: "".to_string(),
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

#[tauri::command]
pub async fn create_workspace(
    directory: &str,
) -> Result<CommandResult<WorkspaceMergeInfo>, CommandResult> {
    let name = match Path::new(directory).file_name() {
        Some(name) => name.to_str().unwrap(),
        None => {
            return Err(CommandResult::failed("Failed to get directory name"));
        }
    };
    let mut workspace = Workspace::new(name);
    let workspace_path = PathBuf::from(directory).join(WORKSPACE_FILE_NAME);
    if workspace_path.exists() {
        let workspace_json = match fs::read_to_string(&workspace_path).await {
            Ok(json) => json,
            Err(_) => {
                return Err(CommandResult::failed("Failed to read Workspace"));
            }
        };
        workspace = match serde_json::from_str::<Workspace>(&workspace_json) {
            Ok(workspace) => workspace,
            Err(_) => {
                return Err(CommandResult::failed("Failed to parse Workspace"));
            }
        };
    } else {
        let workspace_json = serde_json::to_string(&workspace).unwrap();
        match fs::write(&workspace_path, workspace_json).await {
            Ok(_) => {}
            Err(_) => {
                return Err(CommandResult::failed("Failed to save Workspace"));
            }
        }
    }
    let workspace_id = match app_config::add_workspace_to_app_config(directory).await {
        Ok(workspace_id) => workspace_id.value,
        Err(_) => {
            return Err(CommandResult::failed(
                "Failed to add Workspace to AppConfig",
            ));
        }
    };
    return Ok(CommandResult::success(WorkspaceMergeInfo {
        id: workspace_id,
        directory: directory.to_string(),
        name: workspace.name,
        description: workspace.description,
    }));
}

#[tauri::command(rename_all = "snake_case")]
pub async fn load_workspace(
    workspace_id: &str,
) -> Result<CommandResult<WorkspaceMergeInfo>, CommandResult> {
    let app_config = match app_config::read_app_config().await {
        Ok(result) => result.value,
        Err(_) => return Err(CommandResult::failed("Failed to read AppConfig")),
    };

    // すでに登録されている場合は登録IDを返す
    for workspace_info in app_config.workspaces.iter() {
        if workspace_info.id == workspace_id {
            let workspace_path =
                PathBuf::from(workspace_info.directory.as_str()).join(WORKSPACE_FILE_NAME);
            if workspace_path.exists() {
                let workspace_json = match fs::read_to_string(&workspace_path).await {
                    Ok(json) => json,
                    Err(_) => {
                        return Err(CommandResult::failed("Failed to read Workspace"));
                    }
                };
                return match serde_json::from_str::<Workspace>(&workspace_json) {
                    Ok(workspace) => Ok(CommandResult::success(WorkspaceMergeInfo {
                        id: workspace_id.to_string(),
                        directory: workspace_info.directory.clone(),
                        name: workspace.name,
                        description: workspace.description,
                    })),
                    Err(_) => {
                        return Err(CommandResult::failed("Failed to parse Workspace"));
                    }
                };
            } else {
                return Err(CommandResult::failed("Failed to find Workspace"));
            }
        }
    }
    return Err(CommandResult::failed("Failed to find Workspace"));
}

#[tauri::command]
pub async fn load_workspaces() -> Result<CommandResult<Vec<WorkspaceMergeInfo>>, CommandResult> {
    let app_config = match app_config::read_app_config().await {
        Ok(result) => result.value,
        Err(_) => return Err(CommandResult::failed("Failed to read AppConfig")),
    };

    let mut workspaces = Vec::new();
    for workspace_info in app_config.workspaces.iter() {
        let workspace_path =
            PathBuf::from(workspace_info.directory.as_str()).join(WORKSPACE_FILE_NAME);
        if workspace_path.exists() {
            let workspace_json = match fs::read_to_string(&workspace_path).await {
                Ok(json) => json,
                Err(_) => {
                    return Err(CommandResult::failed("Failed to read Workspace"));
                }
            };
            let workspace = match serde_json::from_str::<Workspace>(&workspace_json) {
                Ok(workspace) => workspace,
                Err(_) => {
                    return Err(CommandResult::failed("Failed to parse Workspace"));
                }
            };
            workspaces.push(WorkspaceMergeInfo {
                id: workspace_info.id.clone(),
                directory: workspace_info.directory.clone(),
                name: workspace.name,
                description: workspace.description,
            });
        }
    }
    return Ok(CommandResult::success(workspaces));
}
