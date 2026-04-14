use crate::api::context::api_context::ApiContext;
use crate::config::context::config_context::ConfigContext;
use crate::utils::context::app_context::AppContext;
use crate::utils::CommandResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResourceProviderError {
    #[error("app config error: {0}")]
    AppConfig(#[from] crate::config::app_config::AppConfigError),
    #[error("app config command error: {0}")]
    AppConfigCommand(#[from] crate::command::app_config::AppConfigCommandError),
    #[error("cloud formation error: {0}")]
    Cloudformation(#[from] super::super::api::cloudformation::schema::DlSchemaError),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupAppResult {
    pub new_version: Option<String>,
}

async fn setup_app(
    app_context: &AppContext,
    config_context: &ConfigContext,
    api_context: &ApiContext,
) -> Result<SetupAppResult, ResourceProviderError> {
    let mut app_config = config_context
        .app_config_io
        .read(app_context.file_system.clone())
        .await?;
    if app_config.initialized == false {
        api_context
            .cloudformation_schema
            .dl_resource_provider(&app_context, "us-east-1")
            .await?;
        app_config.initialized = true;
        app_config.initialized_at = Utc::now().to_rfc3339();
        config_context
            .app_config_io
            .write(app_config, app_context.file_system.clone())
            .await?;
    }
    let app_version = env!("CARGO_PKG_VERSION").to_string();
    println!("Version: {}", app_version);
    let latest_version = api_context
        .get_latest_version
        .get_latest_version(app_context)
        .await;
    println!("app_version: {}", app_version);
    println!("latest_version: {:?}", latest_version);
    if let Some(latest_version) = latest_version {
        if latest_version != app_version {
            println!("A new version is available: {}", latest_version);
            return Ok(SetupAppResult {
                new_version: Some(latest_version),
            });
        }
    }
    return Ok(SetupAppResult { new_version: None });
}

#[tauri::command]
#[coverage(off)]
pub async fn setup_app_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    api_context_state: State<'_, ApiContext>,
) -> Result<CommandResult<SetupAppResult>, CommandResult> {
    match setup_app(
        &app_context_state,
        &config_context_state,
        &api_context_state,
    )
    .await
    {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    }
}

#[cfg(test)]
#[coverage(off)]
mod setup_app_tests {
    use chrono::DateTime;

    use super::*;
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        api::{
            cloudformation::schema::DlSchemaError,
            context::{
                api_context::ApiContext,
                cloudformation_schema_trait::MockCloudformationSchemaTrait,
                get_latest_version_trait::MockGetLatestVersionTrait,
            },
        },
        config::{
            app_config::{AppConfig, AppConfigError},
            context::{
                app_config_trait::MockAppConfigTrait, config_context::ConfigContext,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
        },
        utils::{
            context::{app_context::AppContext, file::MockFileSystem, http_client::MockHttpClient},
            AppError,
        },
    };

    /// app_configが初期化されていない、かつ、latest_versionがapp_versionと違う場合、初期化されてlatest_versionを返すことを確認
    #[tokio::test]
    async fn setup_app_initialized() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: false, // 初期化されていない状態
                initialized_at: String::from(""),
                workspaces: HashMap::new(),
            };
            return Ok(app_config);
        });

        // AppConfig::writeのモック
        mock_app_config
            .expect_write()
            .withf(|app_config, _| {
                // app_config.initialized_atがRFC3339形式であることを確認
                match DateTime::parse_from_rfc3339(&app_config.initialized_at) {
                    Ok(_) => {
                        // app_config.initializedがtrueであることを確認
                        return app_config.initialized == true;
                    }
                    Err(_) => return false,
                };
            })
            .returning(|app_config, _| {
                return Ok(app_config);
            });

        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_workspace_config_io = MockWorkspaceConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let mut mock_cloudformation_schema = MockCloudformationSchemaTrait::new();
        let mut mock_get_latest_version = MockGetLatestVersionTrait::new();

        // CloudformationSchema::dl_resource_providerのモック
        mock_cloudformation_schema
            .expect_dl_resource_provider()
            .returning(|_, _| Ok(()));

        // GetLatestVersion::get_latest_versionのモック
        let latest_version = "1.1.1";
        mock_get_latest_version
            .expect_get_latest_version()
            .returning(|_| Some(latest_version.to_string()));

        let api_context = ApiContext {
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            get_latest_version: Arc::new(mock_get_latest_version),
        };

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let initialized = setup_app(&app_context, &config_context, &api_context).await;

        // ######### 検証 #########
        assert!(initialized.is_ok());
        assert_eq!(
            initialized.unwrap().new_version,
            Some(latest_version.to_string())
        );
    }

    /// app_configが初期化済み、かつ、latest_versionとapp_versionが同じ場合、何もせずに{ new_version: None }を返すことを確認
    #[tokio::test]
    async fn setup_app_not_initialized() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: true, // 初期化済みの状態
                initialized_at: String::from(""),
                workspaces: HashMap::new(),
            };
            return Ok(app_config);
        });

        // AppConfig::writeのモック
        mock_app_config.expect_write().times(0); // writeは呼ばれないことを確認

        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_workspace_config_io = MockWorkspaceConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let mut mock_cloudformation_schema = MockCloudformationSchemaTrait::new();
        let mut mock_get_latest_version = MockGetLatestVersionTrait::new();

        // CloudformationSchema::dl_resource_providerのモック
        mock_cloudformation_schema
            .expect_dl_resource_provider()
            .times(0); // dl_resource_providerは呼ばれないことを確認

        // GetLatestVersion::get_latest_versionのモック
        let latest_version = "0.1.2"; // TODO: env!("CARGO_PKG_VERSION")のモック方法を検討
        mock_get_latest_version
            .expect_get_latest_version()
            .returning(|_| Some(latest_version.to_string()));

        let api_context = ApiContext {
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            get_latest_version: Arc::new(mock_get_latest_version),
        };

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let initialized = setup_app(&app_context, &config_context, &api_context).await;

        // ######### 検証 #########
        assert!(initialized.is_ok());
        assert_eq!(initialized.unwrap().new_version, None);
    }

    /// latest_versionがNoneの場合、{ new_version: None }を返すことを確認
    #[tokio::test]
    async fn setup_app_latest_version_none() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: true, // 初期化済みの状態
                initialized_at: String::from(""),
                workspaces: HashMap::new(),
            };
            return Ok(app_config);
        });

        // AppConfig::writeのモック
        mock_app_config.expect_write().times(0); // writeは呼ばれないことを確認

        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_workspace_config_io = MockWorkspaceConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let mut mock_cloudformation_schema = MockCloudformationSchemaTrait::new();
        let mut mock_get_latest_version = MockGetLatestVersionTrait::new();

        // CloudformationSchema::dl_resource_providerのモック
        mock_cloudformation_schema
            .expect_dl_resource_provider()
            .times(0); // dl_resource_providerは呼ばれないことを確認

        // GetLatestVersion::get_latest_versionのモック
        mock_get_latest_version
            .expect_get_latest_version()
            .returning(|_| None); // latest_versionがNoneを返すように設定

        let api_context = ApiContext {
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            get_latest_version: Arc::new(mock_get_latest_version),
        };

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let initialized = setup_app(&app_context, &config_context, &api_context).await;

        // ######### 検証 #########
        assert!(initialized.is_ok());
        assert_eq!(initialized.unwrap().new_version, None);
    }

    /// readの戻り値がErrの場合、setup_appがErrを返すことを確認
    #[tokio::test]
    async fn setup_app_err_read() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            return Err(AppConfigError::App(AppError::new("read error"))); // readがErrを返すように設定
        });

        // AppConfig::writeのモック
        mock_app_config.expect_write().times(0); // writeは呼ばれないことを確認

        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_workspace_config_io = MockWorkspaceConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let mut mock_cloudformation_schema = MockCloudformationSchemaTrait::new();
        let mut mock_get_latest_version = MockGetLatestVersionTrait::new();

        // CloudformationSchema::dl_resource_providerのモック
        mock_cloudformation_schema
            .expect_dl_resource_provider()
            .times(0); // dl_resource_providerは呼ばれないことを確認

        // GetLatestVersion::get_latest_versionのモック
        mock_get_latest_version
            .expect_get_latest_version()
            .returning(|_| Some(String::from("1.1.1")));

        let api_context = ApiContext {
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            get_latest_version: Arc::new(mock_get_latest_version),
        };

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let initialized = setup_app(&app_context, &config_context, &api_context).await;

        // ######### 検証 #########
        assert!(initialized.is_err());
    }

    /// dl_resource_providerの戻り値がErrの場合、setup_appがErrを返すことを確認
    #[tokio::test]
    async fn setup_app_err_dl_resource_provider() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: false, // 初期化されていない状態
                initialized_at: String::from(""),
                workspaces: HashMap::new(),
            };
            return Ok(app_config);
        });

        // AppConfig::writeのモック
        mock_app_config.expect_write().times(0); // writeは呼ばれないことを確認

        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_workspace_config_io = MockWorkspaceConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let mut mock_cloudformation_schema = MockCloudformationSchemaTrait::new();
        let mut mock_get_latest_version = MockGetLatestVersionTrait::new();

        // CloudformationSchema::dl_resource_providerのモック
        mock_cloudformation_schema
            .expect_dl_resource_provider()
            .returning(|_, _| {
                // dl_resource_providerがErrを返すように設定
                return Err(DlSchemaError::App(AppError::new(
                    "dl_resource_provider error",
                )));
            });

        // GetLatestVersion::get_latest_versionのモック
        mock_get_latest_version
            .expect_get_latest_version()
            .returning(|_| Some(String::from("1.1.1")));

        let api_context = ApiContext {
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            get_latest_version: Arc::new(mock_get_latest_version),
        };

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let initialized = setup_app(&app_context, &config_context, &api_context).await;

        // ######### 検証 #########
        assert!(initialized.is_err());
    }

    /// writeの戻り値がErrの場合、setup_appがErrを返すことを確認
    #[tokio::test]
    async fn setup_app_err_write() {
        // ######### 準備 #########
        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: false, // 初期化されていない状態
                initialized_at: String::from(""),
                workspaces: HashMap::new(),
            };
            return Ok(app_config);
        });

        // AppConfig::writeのモック
        mock_app_config.expect_write().returning(|_, _| {
            // writeがErrを返すように設定
            return Err(AppConfigError::App(AppError::new("write error")));
        });

        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_workspace_config_io = MockWorkspaceConfigTrait::new();

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let mut mock_cloudformation_schema = MockCloudformationSchemaTrait::new();
        let mut mock_get_latest_version = MockGetLatestVersionTrait::new();

        // CloudformationSchema::dl_resource_providerのモック
        mock_cloudformation_schema
            .expect_dl_resource_provider()
            .returning(|_, _| Ok(()));

        // GetLatestVersion::get_latest_versionのモック
        mock_get_latest_version
            .expect_get_latest_version()
            .returning(|_| Some(String::from("1.1.1")));

        let api_context = ApiContext {
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            get_latest_version: Arc::new(mock_get_latest_version),
        };

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let initialized = setup_app(&app_context, &config_context, &api_context).await;

        // ######### 検証 #########
        assert!(initialized.is_err());
    }
}
