use std::{fs::File, io::Write, path::PathBuf, sync::Arc};

use crate::{
    config::app_config::APP_CONFIG_DIRECTORY_NAME,
    utils::{context::app_context::AppContext, CommandResult},
};
use serde_json::Value;
use tauri::State;
use tauri_plugin_dialog::DialogExt;
use thiserror::Error;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use crate::utils::{context::file::FileSystem, AppError};

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("zip error: {0:?}")]
    Zip(#[from] zip::result::ZipError),
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

async fn export_translation_file(
    app_handler: tauri::AppHandle,
    file_system: Arc<dyn FileSystem>,
    lang: String,
) -> Result<bool, TranslationError> {
    let dir = file_system
        .config_local_dir()
        .ok_or(TranslationError::App(AppError::new(
            "Failed to get local config directory",
        )))?;
    let translation_dir_path = dir
        .join(APP_CONFIG_DIRECTORY_NAME)
        .join(TRANSLATION_DIR)
        .join(&lang);
    match translation_dir_path.try_exists() {
        Ok(exists) => {
            // 翻訳ディレクトリが存在しない場合は、エクスポートを中止してfalseを返す
            if !exists {
                return Ok(false);
            }
        }
        Err(_) => {
            return Ok(false);
        }
    }

    // ファイル保存ダイアログを表示して、ユーザーに保存先を選択させる
    app_handler
        .dialog()
        .file()
        .set_file_name("translation.zip")
        .save_file(move |path| {
            if let Some(path) = path {
                println!("保存先のパス: {:?}", path);

                let output_path = match path {
                    tauri_plugin_dialog::FilePath::Path(path) => path,
                    _ => {
                        eprintln!("Failed to export translation zip: Invalid file path");
                        return;
                    }
                };

                // 非同期処理を実行するために、分離した関数をspawnで呼び出す
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = write_translation_zip(
                        file_system.clone(),
                        translation_dir_path.clone(),
                        output_path,
                    )
                    .await
                    {
                        eprintln!("Failed to export translation zip: {err}");
                    }
                });
            }
        });

    Ok(true)
}

async fn write_translation_zip(
    file_system: Arc<dyn FileSystem>,
    translation_dir_path: PathBuf,
    output_path: PathBuf,
) -> Result<(), TranslationError> {
    let zip_file = File::create(output_path)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in file_system.read_dir(&translation_dir_path).await? {
        if entry.is_file() {
            let mut file = file_system.open(&entry)?;
            let mut contents = Vec::new();
            std::io::copy(&mut file, &mut contents)?;
            let file_name = entry
                .file_name()
                .ok_or_else(|| AppError::new("Failed to get file name"))?
                .to_string_lossy() // OsStringをStringに変換
                .into_owned();
            zip.start_file(file_name, options)?;
            zip.write_all(&contents)?;
        }
    }
    zip.finish()?;

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

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn export_translation_file_command(
    app_context_state: State<'_, AppContext>,
    app_handler: tauri::AppHandle,
    lang: String,
) -> Result<CommandResult<bool>, CommandResult> {
    let app_context = app_context_state.inner();
    match export_translation_file(app_handler, app_context.file_system.clone(), lang).await {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(_) => Err(CommandResult::failed("Failed to export translation file.")),
    }
}
