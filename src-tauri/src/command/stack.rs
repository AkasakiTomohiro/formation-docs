use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::utils::AppError;
use crate::utils::CommandResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

// フロントで利用する形
#[derive(Debug, Serialize, Deserialize)]
pub struct Stack {
    name: String,
    description_from_meta: Option<String>,
    description_from_stack: Option<String>,
}

#[derive(Debug, Error)]
enum LoadStackMetaError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

fn load_stack_meta(
    workspace_directory: &str,
    stack_name: &str,
) -> Result<Value, LoadStackMetaError> {
    // ${スタック名}.meta.jsonが存在するか確認
    let meta_path = format!("{}/{}.meta.json", workspace_directory, stack_name);
    let meta_path = Path::new(&meta_path);
    if !meta_path.exists() {
        // ${スタック名}.meta.jsonを作成する
        fs::File::create(&meta_path)?;
        // 空のJSONを作成
        let empty_json = "{}".to_string();
        fs::write(&meta_path, empty_json)?;
    }

    // ${スタック名}.meta.jsonを読み込む
    let meta_json = fs::read_to_string(&meta_path)?;
    let meta_json: Value = serde_json::from_str(&meta_json).unwrap_or_default();
    return Ok(meta_json);
}

#[derive(Debug, Error)]
enum LoadStacksError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    // FIXME: globmatchのエラーを含める
    // #[error("globmatch error: {0}")]
    // Globmatch(#[from] globmatch::Error),
}

/**
 * 1. globmatchを使って、workspace_directoryの直下にある*.template.jsonを取得する
 *  1. テンプレートファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_stack）
 *  2. ${スタック名}.meta.jsonを取得する（description_from_metaと各リソースとそのパラメータの説明文）
 *  3. メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
 */
fn load_stacks(workspace_directory: &str) -> Result<Vec<Stack>, LoadStacksError> {
    // globmatchを使って、workspace_directoryの直下にある*.template.jsonを取得する
    let builder = globmatch::Builder::new("./*.template.json").build(workspace_directory);
    let builder = match builder {
        Ok(builder) => builder,
        Err(_) => {
            return Err(LoadStacksError::App(AppError::new("Failed to load stacks")));
        }
    };
    let paths: Vec<_> = builder.into_iter().flatten().collect();
    let mut stacks = Vec::new();
    for path in paths {
        // テンプレートファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_stack）
        let template_json = fs::read_to_string(&path)?;
        let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
        let description_from_stack = template_json["Description"]
            .as_str()
            .and_then(|desc| Some(desc.to_string()));

        // ${スタック名}.meta.jsonを取得する（description_from_metaと各リソースとそのパラメータの説明文）
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .replace(".template.json", "");

        // メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
        let description_from_meta = match load_stack_meta(workspace_directory, &filename) {
            Ok(meta_json) => meta_json["description"]
                .as_str()
                .and_then(|desc: &str| Some(desc.to_string())),
            Err(_) => {
                return Err(LoadStacksError::App(AppError::new(
                    "Failed to load stack meta",
                )));
            }
        };

        // アプリ返却用のデータ構造作成
        stacks.push(Stack {
            name: filename,
            description_from_meta: description_from_meta,
            description_from_stack: description_from_stack,
        });
    }

    return Ok(stacks);
}

#[derive(Debug, Error)]
enum DeleteStacksError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
fn delete_stack(workspace_directory: &str, stack_name: &str) -> Result<(), DeleteStacksError> {
    fs::remove_file(format!(
        "{}/{}.template.json",
        workspace_directory, stack_name
    ))?;
    fs::remove_file(format!("{}/{}.meta.json", workspace_directory, stack_name))?;
    return Ok(());
}

#[derive(Debug, Error)]
enum ImportStacksError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
fn import_stack(workspace_directory: &str, stack_file_path: &str) -> Result<(), ImportStacksError> {
    let stack_file_path_buf = PathBuf::from(stack_file_path);
    let filename = stack_file_path_buf
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let copy_file_path = format!("{}/{}", workspace_directory, filename);
    if copy_file_path == stack_file_path {
        return Ok(());
    }
    fs::copy(stack_file_path, copy_file_path)?;
    return Ok(());
}

#[tauri::command(rename_all = "snake_case")]
pub fn load_stacks_command(
    workspace_directory: &str,
) -> Result<CommandResult<Vec<Stack>>, CommandResult> {
    return match load_stacks(workspace_directory) {
        Ok(stacks) => Ok(CommandResult::success(stacks)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_stack_command(
    workspace_directory: &str,
    stack_name: &str,
) -> Result<CommandResult<()>, CommandResult> {
    return match delete_stack(workspace_directory, stack_name) {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub fn import_stack_command(
    workspace_directory: &str,
    stack_file_path: &str,
) -> Result<CommandResult<()>, CommandResult> {
    return match import_stack(workspace_directory, stack_file_path) {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
