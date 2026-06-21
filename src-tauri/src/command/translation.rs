use std::sync::Arc;

use crate::utils::{context::app_context::AppContext, CommandResult};
use serde_json::Value;
use tauri::State;
use thiserror::Error;

use crate::utils::{context::file::FileSystem, AppError};

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

const TRANSLATION_DIR: &str = "translations";

async fn get_translation(
    file_system: Arc<dyn FileSystem>,
    lang: String,
    service_name: String,
    resource_type: String,
) -> Result<Value, TranslationError> {
    let dir = file_system
        .config_local_dir()
        .ok_or(TranslationError::App(AppError::new(
            "Failed to get local config directory",
        )))?;
    let path = dir.join(TRANSLATION_DIR).join(&lang).join(format!(
        "{}-{}.json",
        service_name.to_lowercase(),
        resource_type.to_lowercase()
    ));

    match path.try_exists() {
        Ok(exists) => {
            if !exists {
                return Ok(Value::Object(serde_json::Map::new()));
            }
        }
        Err(_) => {
            return Ok(Value::Object(serde_json::Map::new()));
        }
    }
    let content = file_system.read_file(&path).await?;
    let json: Value = serde_json::from_str(&content)?;
    return Ok(json);
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_translation_command(
    app_context_state: State<'_, AppContext>,
    lang: String,
    service_name: String,
    resource_type: String,
) -> Result<CommandResult<Value>, CommandResult> {
    let app_context = app_context_state.inner();
    match get_translation(
        app_context.file_system.clone(),
        lang,
        service_name,
        resource_type,
    )
    .await
    {
        Ok(translation) => Ok(CommandResult::success(translation)),
        Err(_) => Err(CommandResult::failed("Failed to get translation.")),
    }
}
