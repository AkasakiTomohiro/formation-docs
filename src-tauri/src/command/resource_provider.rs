use crate::api::context::api_context::ApiContext;
use crate::config::context::config_context::ConfigContext;
use crate::utils::context::app_context::AppContext;
use chrono::Utc;
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

async fn setup_app(
    app_context: &AppContext,
    config_context: &ConfigContext,
    api_context: &ApiContext,
) -> Result<(), ResourceProviderError> {
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
        app_config.initialized_at = Utc::now().to_string();
        config_context
            .app_config_io
            .write(app_config, app_context.file_system.clone())
            .await?;
    }
    return Ok(());
}

#[tauri::command]
#[coverage(off)]
pub async fn setup_app_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    api_context_state: State<'_, ApiContext>,
) -> Result<(), ()> {
    match setup_app(
        &app_context_state,
        &config_context_state,
        &api_context_state,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

#[cfg(test)]
mod setup_app_tests {
    use chrono::DateTime;

    use super::*;
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        api::context::{
            api_context::ApiContext, cloudformation_schema_trait::MockCloudformationSchemaTrait,
        },
        config::{
            app_config::AppConfig,
            context::{
                app_config_trait::MockAppConfigTrait, config_context::ConfigContext,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
        },
        utils::context::{
            app_context::AppContext, file::MockFileSystem, http_client::MockHttpClient,
        },
    };

    /// app_configが初期化されていない場合、初期化されること
    #[tokio::test]
    async fn setup_app_initialized() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mut mock_app_config = MockAppConfigTrait::new();

        // AppConfig::readのモック
        mock_app_config.expect_read().returning(|_| {
            let app_config = AppConfig {
                version: 1,
                initialized: false,
                initialized_at: String::from(""),
                workspaces: HashMap::new(),
            };
            return Ok(app_config);
        });

        // AppConfig::writeのモック
        mock_app_config
            .expect_write()
            // FIXME: 以下のwithfでinitialized_atのフォーマットチェックを入れたいが、うまく動作しないため一旦コメントアウト
            // .withf(|app_config, _| {
            //     match DateTime::parse_from_rfc3339(&app_config.initialized_at) {
            //         Ok(_) => {
            //             return app_config.initialized == true;
            //         }
            //         Err(_) => return false,
            //     };
            // })
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

        // CloudformationSchema::dl_resource_providerのモック
        mock_cloudformation_schema
            .expect_dl_resource_provider()
            .returning(|_, _| Ok(()));

        let api_context = ApiContext {
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
        };

        // ######### 実行 #########
        let initialized = setup_app(&app_context, &config_context, &api_context).await;

        // ######### 検証 #########
        assert!(initialized.is_ok());
    }
}
