use crate::config::app_config::AppConfig;
use crate::config::app_config::AppConfigError;
use crate::config::context::config_context::ConfigContext;
use crate::utils::context::app_context::AppContext;
use crate::utils::AppError;
use crate::utils::CommandResult;
use tauri::State;
use thiserror::Error;
use uuid::Uuid;

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
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
) -> Result<String, AppConfigCommandError> {
    let mut app_config = config_context
        .app_config_io
        .read(app_context.file_system.clone())
        .await?;
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
    config_context
        .app_config_io
        .write(app_config, app_context.file_system.clone())
        .await?;
    return Ok(workspace_id);
}

async fn delete_workspace_from_app_config(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_id: &str,
) -> Result<(), AppConfigCommandError> {
    let mut app_config = config_context
        .app_config_io
        .read(app_context.file_system.clone())
        .await?;
    app_config.workspaces.remove(workspace_id);
    config_context
        .app_config_io
        .write(app_config, app_context.file_system.clone())
        .await?;
    return Ok(());
}

#[coverage(off)]
#[tauri::command]
pub async fn read_app_config_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
) -> Result<CommandResult<AppConfig>, CommandResult> {
    return match config_context_state
        .app_config_io
        .read(app_context_state.file_system.clone())
        .await
    {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to read AppConfig")),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_workspace_from_app_config_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    workspace_id: &str,
) -> Result<CommandResult<()>, CommandResult<String>> {
    return match delete_workspace_from_app_config(
        &app_context_state,
        &config_context_state,
        workspace_id,
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[cfg(test)]
#[coverage(off)]
mod app_config_command_tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        config::context::{
            app_config_trait::MockAppConfigTrait,
            stack_meta_config_trait::MockStackMetaConfigTrait,
            workspace_config_trait::MockWorkspaceConfigTrait,
        },
        utils::context::{clock::MockClock, file::MockFileSystem, http_client::MockHttpClient},
    };

    use super::*;

    /// AppConfigにworkspaceを新規作成できることを確認
    #[tokio::test]
    async fn add_workspace_to_app_config_success() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        let workspace_directory = "test_workspace_directory";

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: true,
                initialized_at: String::from("2023-01-01T00:00:00Z"),
                workspaces: HashMap::new(),
                cf_schema_downloaded_at: String::from("2023-01-01T00:00:00Z"),
            };
            Ok(app_config)
        });

        // AppConfig::writeのモック
        mock_app_config
            .expect_write()
            .returning(|config, _| Ok(config));

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_clock = MockClock::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
            clock: Arc::new(mock_clock),
        };

        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            add_workspace_to_app_config(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        // workspace_idがUUID形式であることを確認
        let result_workspace_id = result.unwrap();
        assert!(Uuid::parse_str(&result_workspace_id).is_ok());
    }

    /// AppConfigにworkspaceを追加できることを確認
    #[tokio::test]
    async fn add_workspace_to_app_config_add_workspace() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        let workspace_directory = "test_workspace_directory";
        let workspace_id: &str = "existing_workspace_id";
        let new_workspace_directory = "new_workspace_directory";

        let mut workspaces = HashMap::new();
        workspaces.insert(
            String::from(workspace_id),
            String::from(workspace_directory),
        );

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(move |_| {
            let app_config = AppConfig {
                version: 1,
                initialized: true,
                initialized_at: String::from("2023-01-01T00:00:00Z"),
                cf_schema_downloaded_at: String::from("2023-01-01T00:00:00Z"),
                workspaces: workspaces.clone(),
            };
            Ok(app_config)
        });

        // AppConfig::writeのモック
        mock_app_config
            .expect_write()
            .returning(|config, _| Ok(config));

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_clock = MockClock::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
            clock: Arc::new(mock_clock),
        };

        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            add_workspace_to_app_config(&app_context, &config_context, new_workspace_directory)
                .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        // workspace_idがUUID形式であることを確認
        let result_workspace_id = result.unwrap();
        assert!(Uuid::parse_str(&result_workspace_id).is_ok());
    }

    /// すでに登録されているworkspaceの場合、既存のIDが返されることを確認
    #[tokio::test]
    async fn add_workspace_to_app_config_existing_workspace() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        let workspace_directory = "test_workspace_directory";
        let workspace_id = "existing_workspace_id";

        let mut workspaces = HashMap::new();
        workspaces.insert(
            String::from(workspace_id),
            String::from(workspace_directory),
        );

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(move |_| {
            let app_config = AppConfig {
                version: 1,
                initialized: true,
                initialized_at: String::from("2023-01-01T00:00:00Z"),
                workspaces: workspaces.clone(),
                cf_schema_downloaded_at: String::from("2023-01-01T00:00:00Z"),
            };
            Ok(app_config)
        });

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_clock = MockClock::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
            clock: Arc::new(mock_clock),
        };

        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            add_workspace_to_app_config(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let result_workspace_id = result.unwrap();
        assert!(result_workspace_id == workspace_id);
    }

    /// AppConfig::readが失敗した場合、エラーが返されることを確認
    #[tokio::test]
    async fn add_workspace_to_app_config_read_err() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config
            .expect_read()
            .returning(|_| Err(AppConfigError::App(AppError::new("Read error"))));

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_clock = MockClock::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
            clock: Arc::new(mock_clock),
        };

        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let workspace_directory = "test_workspace_directory";

        // ######### 実行 #########
        let result =
            add_workspace_to_app_config(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// AppConfig::writeが失敗した場合、エラーが返されることを確認
    #[tokio::test]
    async fn add_workspace_to_app_config_write_err() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: true,
                initialized_at: String::from("2023-01-01T00:00:00Z"),
                workspaces: HashMap::new(),
                cf_schema_downloaded_at: String::from("2023-01-01T00:00:00Z"),
            };
            Ok(app_config)
        });

        // AppConfig::writeのモック
        mock_app_config
            .expect_write()
            .returning(|_, _| Err(AppConfigError::App(AppError::new("Write error"))));

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_clock = MockClock::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
            clock: Arc::new(mock_clock),
        };

        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let workspace_directory = "test_workspace_directory";

        // ######### 実行 #########
        let result =
            add_workspace_to_app_config(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// AppConfigからworkspaceを削除できることを確認
    #[tokio::test]
    async fn delete_workspace_from_app_config_success() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        let workspace_directory = "test_workspace_directory";
        let workspace_id = "test_workspace_id";

        let mut workspaces = HashMap::new();
        workspaces.insert(
            String::from(workspace_id),
            String::from(workspace_directory),
        );

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: true,
                initialized_at: String::from("2023-01-01T00:00:00Z"),
                workspaces: HashMap::new(),
                cf_schema_downloaded_at: String::from("2023-01-01T00:00:00Z"),
            };
            Ok(app_config)
        });

        // AppConfig::writeのモック
        mock_app_config
            .expect_write()
            .returning(|config, _| Ok(config));

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_clock = MockClock::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
            clock: Arc::new(mock_clock),
        };

        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_workspace_from_app_config(&app_context, &config_context, workspace_id).await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// AppConfig::readが失敗した場合、エラーが返されることを確認
    #[tokio::test]
    async fn delete_workspace_from_app_config_read_err() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        let workspace_directory = "test_workspace_directory";
        let workspace_id = "test_workspace_id";

        let mut workspaces = HashMap::new();
        workspaces.insert(
            String::from(workspace_id),
            String::from(workspace_directory),
        );

        // AppConfig::readのモック
        mock_app_config
            .expect_read()
            .returning(|_| Err(AppConfigError::App(AppError::new("Read error"))));

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_clock = MockClock::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
            clock: Arc::new(mock_clock),
        };

        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_workspace_from_app_config(&app_context, &config_context, workspace_id).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// AppConfig::writeが失敗した場合、エラーが返されることを確認
    #[tokio::test]
    async fn delete_workspace_from_app_config_write_err() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        let workspace_directory = "test_workspace_directory";
        let workspace_id = "test_workspace_id";

        let mut workspaces = HashMap::new();
        workspaces.insert(
            String::from(workspace_id),
            String::from(workspace_directory),
        );

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: true,
                initialized_at: String::from("2023-01-01T00:00:00Z"),
                workspaces: HashMap::new(),
                cf_schema_downloaded_at: String::from("2023-01-01T00:00:00Z"),
            };
            Ok(app_config)
        });

        // AppConfig::writeのモック
        mock_app_config
            .expect_write()
            .returning(|_, _| Err(AppConfigError::App(AppError::new("Write error"))));

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_clock = MockClock::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
            clock: Arc::new(mock_clock),
        };

        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_workspace_from_app_config(&app_context, &config_context, workspace_id).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }
}
