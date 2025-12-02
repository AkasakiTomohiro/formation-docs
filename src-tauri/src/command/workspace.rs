use crate::config::app_config::AppConfigError;
use crate::config::context::config_context::ConfigContext;
use crate::config::workspace_config::WorkspaceConfigError;
use crate::utils::context::app_context::AppContext;
use crate::utils::get_window_state;
use crate::utils::set_window_state;
use crate::utils::AppError;
use crate::utils::CommandResult;
use crate::utils::WindowState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{Manager, State};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceMergeInfo {
    pub id: String,
    pub directory: String,
    pub name: String,
    pub description: String,
    pub stacks: HashMap<String, String>,
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
    AppConfig(#[from] AppConfigError),
    #[error("app config error: {0}")]
    AppConfigCommand(#[from] super::app_config::AppConfigCommandError),
    #[error("workspace config error: {0}")]
    WorkspaceConfig(#[from] WorkspaceConfigError),
}

async fn create_workspace(
    app_context: &AppContext,
    config_context: &ConfigContext,
    directory: &str,
) -> Result<WorkspaceMergeInfo, WorkspaceCommandError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), directory)
        .await?;
    let workspace_result =
        super::app_config::add_workspace_to_app_config(app_context, config_context, directory)
            .await?;
    return Ok(WorkspaceMergeInfo {
        id: workspace_result,
        directory: directory.to_string(),
        name: workspace.name,
        description: workspace.description,
        stacks: workspace.stacks.clone(),
    });
}

async fn load_workspace_merge_info(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_id: &str,
) -> Result<WorkspaceMergeInfo, WorkspaceCommandError> {
    let app_config = config_context
        .app_config_io
        .read(app_context.file_system.clone())
        .await?;

    // すでに登録されている場合は登録IDを返す
    let workspace_directory = app_config.workspaces.get(workspace_id);
    if workspace_directory.is_some() {
        let workspace = config_context
            .workspace_config_io
            .read(
                app_context.file_system.clone(),
                workspace_directory.unwrap(),
            )
            .await?;
        return Ok(WorkspaceMergeInfo {
            id: workspace_id.to_string(),
            directory: workspace_directory.unwrap().to_string(),
            name: workspace.name,
            description: workspace.description,
            stacks: workspace.stacks.clone(),
        });
    }
    return Err(WorkspaceCommandError::App(AppError::new(
        "Failed to find Workspace",
    )));
}

async fn load_workspaces(
    app_context: &AppContext,
    config_context: &ConfigContext,
) -> Result<Vec<WorkspaceMergeInfo>, WorkspaceCommandError> {
    let app_config = config_context
        .app_config_io
        .read(app_context.file_system.clone())
        .await?;

    let mut workspaces = Vec::new();
    for (id, directory) in app_config.workspaces.iter() {
        let workspace = config_context
            .workspace_config_io
            .read(app_context.file_system.clone(), directory)
            .await?;
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
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    name: &str,
    description: &str,
) -> Result<(), WorkspaceCommandError> {
    let mut workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    workspace.name = name.to_string();
    workspace.description = description.to_string();
    workspace
        .write(app_context.file_system.clone(), workspace_directory)
        .await?;
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
        let app_context = handle.state::<AppContext>();
        let config_context = handle.state::<ConfigContext>();
        let file_system = app_context.file_system.clone();
        let workspace_info = load_workspace_merge_info(&app_context, &config_context, id).await?;
        let path = Path::new(workspace_info.directory.as_str());
        log::info!("Open: {}", path.to_str().unwrap());
        if !file_system.path_exists(path) {
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
            tauri::WebviewUrl::App(PathBuf::from("workspaces").join(id)),
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
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    directory: &str,
) -> Result<CommandResult<WorkspaceMergeInfo>, CommandResult> {
    match create_workspace(&app_context_state, &config_context_state, directory).await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to create Workspace")),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn load_workspace_merge_info_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    workspace_id: &str,
) -> Result<CommandResult<WorkspaceMergeInfo>, CommandResult> {
    match load_workspace_merge_info(&app_context_state, &config_context_state, workspace_id).await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to load Workspace")),
    }
}

#[tauri::command]
pub async fn load_workspaces_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
) -> Result<CommandResult<Vec<WorkspaceMergeInfo>>, CommandResult> {
    match load_workspaces(&app_context_state, &config_context_state).await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to load Workspaces")),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_workspace_details_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    window: tauri::Window,
    name: &str,
    description: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    match update_workspace(
        &app_context_state,
        &config_context_state,
        window_state.workspace_directory.as_str(),
        name,
        description,
    )
    .await
    {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to update Workspace")),
    }
}
