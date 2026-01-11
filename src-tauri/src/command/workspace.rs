use crate::command::context::command_context::CommandContext;
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
    command_context: &CommandContext,
    directory: &str,
) -> Result<WorkspaceMergeInfo, WorkspaceCommandError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), directory)
        .await?;
    let workspace_result = command_context
        .app_config
        .add_workspace_to_app_config(app_context, config_context, directory)
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
    config_context
        .workspace_config_io
        .write(
            workspace,
            app_context.file_system.clone(),
            workspace_directory,
        )
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

#[coverage(off)]
#[tauri::command]
pub async fn create_workspace_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    command_context_state: State<'_, CommandContext>,
    directory: &str,
) -> Result<CommandResult<WorkspaceMergeInfo>, CommandResult> {
    match create_workspace(
        &app_context_state,
        &config_context_state,
        &command_context_state,
        directory,
    )
    .await
    {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to create Workspace")),
    }
}

#[coverage(off)]
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

#[coverage(off)]
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

#[coverage(off)]
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

#[cfg(test)]
#[coverage(off)]
mod create_workspace_tests {
    use std::sync::Arc;

    use crate::{
        command::{
            app_config::AppConfigCommandError,
            context::{
                app_config_command_trait::MockAppConfigCommandTrait,
                cloudformation_schema_trait::MockCloudFormationSchemaTrait,
                manual_management_resource_trait::MockManualManagementResourceTrait,
                stack_command_trait::MockStackCommandTrait,
            },
        },
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };

    use super::*;

    /// ワークスペースの作成に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io.expect_read().returning(|_, _| {
            Ok(WorkspaceConfig {
                version: 1,
                name: "Test Workspace".to_string(),
                description: "This is a test workspace.".to_string(),
                stacks: HashMap::new(),
            })
        });
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mut mock_app_config_command = MockAppConfigCommandTrait::new();
        mock_app_config_command
            .expect_add_workspace_to_app_config()
            .returning(|_, _, _| Ok("workspace_id".to_string()));
        let mock_stack_command = MockStackCommandTrait::new();
        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = create_workspace(
            &app_context,
            &config_context,
            &command_context,
            "test_directory",
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let workspace_info = result.unwrap();
        assert_eq!(workspace_info.id, "workspace_id");
        assert_eq!(workspace_info.directory, "test_directory");
        assert_eq!(workspace_info.name, "Test Workspace");
        assert_eq!(workspace_info.description, "This is a test workspace.");
        assert!(workspace_info.stacks.is_empty());
    }

    /// workspace_config の読み込みに失敗する場合、Err を返すこと
    #[tokio::test]
    async fn fail_read_workspace_config() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io.expect_read().returning(|_, _| {
            Err(WorkspaceConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to read workspace config",
            )))
        });
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mock_app_config_command = MockAppConfigCommandTrait::new();
        let mock_stack_command = MockStackCommandTrait::new();
        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = create_workspace(
            &app_context,
            &config_context,
            &command_context,
            "test_directory",
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// app_config へのワークスペース追加に失敗する場合、Err を返すこと
    #[tokio::test]
    async fn fail_add_workspace_to_app_config() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io.expect_read().returning(|_, _| {
            Ok(WorkspaceConfig {
                version: 1,
                name: "Test Workspace".to_string(),
                description: "This is a test workspace.".to_string(),
                stacks: HashMap::new(),
            })
        });
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mut mock_app_config_command = MockAppConfigCommandTrait::new();
        mock_app_config_command
            .expect_add_workspace_to_app_config()
            .returning(|_, _, _| {
                Err(AppConfigCommandError::AppConfig(AppConfigError::Io(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to add workspace to app config",
                    ),
                )))
            });
        let mock_stack_command = MockStackCommandTrait::new();
        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = create_workspace(
            &app_context,
            &config_context,
            &command_context,
            "test_directory",
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod load_workspace_merge_info_tests {
    use std::sync::Arc;

    use crate::{
        config::{
            app_config::AppConfig,
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };

    use super::*;

    /// app_config に指定したワークスペースが既に存在する場合、ワークスペース情報を返すこと
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let workspace_id = "workspace_id";
        mock_app_config_io.expect_read().returning(|_| {
            let mut workspaces = HashMap::new();
            workspaces.insert(workspace_id.to_string(), "workspace_directory".to_string());
            Ok(AppConfig {
                version: 1,
                workspaces,
                initialized: true,
                initialized_at: "1".to_string(),
            })
        });
        mock_workspace_config_io.expect_read().returning(|_, _| {
            let mut stacks = HashMap::new();
            stacks.insert("stack1".to_string(), "stack 1".to_string());
            Ok(WorkspaceConfig {
                version: 2,
                description: "workspace description".to_string(),
                name: "test workspace".to_string(),
                stacks,
            })
        });
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        // ######### 実行 #########
        let result = load_workspace_merge_info(&app_context, &config_context, workspace_id).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let workspace_info = result.unwrap();
        assert_eq!(workspace_info.id, workspace_id);
        assert_eq!(workspace_info.directory, "workspace_directory");
        assert_eq!(workspace_info.name, "test workspace");
        assert_eq!(workspace_info.description, "workspace description");
        assert_eq!(workspace_info.stacks.len(), 1);
        assert_eq!(workspace_info.stacks.get("stack1").unwrap(), "stack 1");
    }

    /// app_config に指定したワークスペースが存在しない場合、Err を返すこと
    #[tokio::test]
    async fn fail_not_exist_workspace() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let workspace_id = "workspace_id";
        mock_app_config_io.expect_read().returning(|_| {
            let mut workspaces = HashMap::new();
            workspaces.insert(workspace_id.to_string(), "workspace_directory".to_string());
            Ok(AppConfig {
                version: 1,
                workspaces,
                initialized: true,
                initialized_at: "1".to_string(),
            })
        });
        mock_workspace_config_io.expect_read().times(0);
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        // ######### 実行 #########
        let result =
            load_workspace_merge_info(&app_context, &config_context, "not_exist_workspace_id")
                .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// app_config の読み込みに失敗する場合、Err を返すこと
    #[tokio::test]
    async fn fail_read_app_config() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let workspace_id = "workspace_id";
        mock_app_config_io.expect_read().returning(|_| {
            Err(AppConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to read app config",
            )))
        });
        mock_workspace_config_io.expect_read().times(0);
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        // ######### 実行 #########
        let result = load_workspace_merge_info(&app_context, &config_context, workspace_id).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// workspace_config の読み込みに失敗する場合、Err を返すこと
    #[tokio::test]
    async fn fail_read_workspace_config() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let workspace_id = "workspace_id";
        mock_app_config_io.expect_read().returning(|_| {
            let mut workspaces = HashMap::new();
            workspaces.insert(workspace_id.to_string(), "workspace_directory".to_string());
            Ok(AppConfig {
                version: 1,
                workspaces,
                initialized: true,
                initialized_at: "1".to_string(),
            })
        });
        mock_workspace_config_io.expect_read().returning(|_, _| {
            Err(WorkspaceConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to read workspace config",
            )))
        });
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        // ######### 実行 #########
        let result = load_workspace_merge_info(&app_context, &config_context, workspace_id).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }
}
