use crate::config::app_config::AppConfig;
use chrono::Utc;
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

async fn setup_app() -> Result<(), ResourceProviderError> {
    let mut app_config = AppConfig::read().await?;
    if app_config.initialized == false {
        cloudformation::schema::dl_resource_provider("us-east-1").await?;
        app_config.initialized = true;
        app_config.initialized_at = Utc::now().to_string();
        app_config.write().await?;
    }
    return Ok(());
}

#[tauri::command]
pub async fn setup_app_command() -> Result<(), ()> {
    match setup_app().await {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}
