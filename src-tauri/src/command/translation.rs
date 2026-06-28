use std::sync::Arc;

use crate::{
    config::app_config::APP_CONFIG_DIRECTORY_NAME,
    utils::{context::app_context::AppContext, CommandResult},
};
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
    let path = dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(TRANSLATION_DIR)
        .join(&lang)
        .join(format!(
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

async fn save_translation(
    file_system: Arc<dyn FileSystem>,
    lang: String,
    service_name: String,
    resource_type: String,
    translation: Value,
) -> Result<(), TranslationError> {
    let dir = file_system
        .config_local_dir()
        .ok_or(TranslationError::App(AppError::new(
            "Failed to get local config directory",
        )))?;
    let dir_path = dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(TRANSLATION_DIR)
        .join(&lang);
    if dir_path.clone().exists() == false {
        file_system
            .create_dir_all(dir_path.clone().as_path())
            .await?;
    }

    let file_path = dir_path.join(format!(
        "{}-{}.json",
        service_name.to_lowercase(),
        resource_type.to_lowercase()
    ));
    let contents = serde_json::to_string(&translation)?;
    file_system
        .write_file(&file_path.to_path_buf(), contents.as_bytes())
        .await?;

    Ok(())
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

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn save_translation_command(
    app_context_state: State<'_, AppContext>,
    lang: String,
    service_name: String,
    resource_type: String,
    translation: Value,
) -> Result<CommandResult<()>, CommandResult> {
    let app_context = app_context_state.inner();
    match save_translation(
        app_context.file_system.clone(),
        lang,
        service_name,
        resource_type,
        translation,
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(_) => Err(CommandResult::failed("Failed to save translation.")),
    }
}
