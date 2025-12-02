use crate::config::context::config_context::ConfigContext;
use crate::utils::context::app_context::AppContext;
use chrono::Utc;
use tauri::State;
use thiserror::Error;

use super::super::api::cloudformation;

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
) -> Result<(), ResourceProviderError> {
    let mut app_config = config_context
        .app_config_io
        .read(app_context.file_system.clone())
        .await?;
    if app_config.initialized == false {
        cloudformation::schema::dl_resource_provider(&app_context, "us-east-1").await?;
        app_config.initialized = true;
        app_config.initialized_at = Utc::now().to_string();
        app_config.write(app_context.file_system.clone()).await?;
    }
    return Ok(());
}

#[tauri::command]
pub async fn setup_app_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
) -> Result<(), ()> {
    match setup_app(&app_context_state, &config_context_state).await {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}
