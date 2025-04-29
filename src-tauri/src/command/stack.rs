use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::utils::get_window_state;
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
enum LoadStackError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

fn load_stack_meta(workspace_directory: &str, stack_name: &str) -> Result<Value, LoadStackError> {
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

/**
 * 1. globmatchを使って、workspace_directoryの直下にある*.template.jsonを取得する
 *  1. テンプレートファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_stack）
 *  2. ${スタック名}.meta.jsonを取得する（description_from_metaと各リソースとそのパラメータの説明文）
 *  3. メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
 */
fn load_stacks(workspace_directory: &str) -> Result<Vec<Stack>, LoadStackError> {
    // globmatchを使って、workspace_directoryの直下にある*.template.jsonを取得する
    let builder = globmatch::Builder::new("./*.template.json").build(workspace_directory);
    let builder = match builder {
        Ok(builder) => builder,
        Err(_) => {
            return Err(LoadStackError::App(AppError::new("Failed to load stacks")));
        }
    };
    let paths: Vec<_> = builder.into_iter().flatten().collect();
    let mut stacks = Vec::new();
    for path in paths {
        // スタック名を取得する
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .replace(".template.json", "");

        // スタックを読み込む
        let stack = match load_stack(workspace_directory, &filename) {
            Ok(stack) => stack,
            Err(_) => {
                return Err(LoadStackError::App(AppError::new("Failed to load stack")));
            }
        };

        stacks.push(stack);
    }

    return Ok(stacks);
}

fn delete_stack(workspace_directory: &str, stack_name: &str) -> Result<(), LoadStackError> {
    fs::remove_file(format!(
        "{}/{}.template.json",
        workspace_directory, stack_name
    ))?;
    fs::remove_file(format!("{}/{}.meta.json", workspace_directory, stack_name))?;
    return Ok(());
}

fn import_stack(workspace_directory: &str, stack_file_path: &str) -> Result<(), LoadStackError> {
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

    if filename.ends_with(".yaml") || filename.ends_with(".yml") {
        // YAMLファイルをJSONに変換して保存
        let yaml_content = fs::read_to_string(stack_file_path)?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;
        let json_content = serde_json::to_string_pretty(&yaml_value)?;
        let json_file_path = copy_file_path
            .replace(".yaml", ".json")
            .replace(".yml", ".json");
        fs::write(json_file_path, json_content)?;
    } else {
        // 通常のコピー処理
        fs::copy(stack_file_path, copy_file_path)?;
    }

    return Ok(());
}

fn load_stack(workspace_directory: &str, stack_name: &str) -> Result<Stack, LoadStackError> {
    let template_path = format!("{}/{}.template.json", workspace_directory, stack_name);

    // テンプレートファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_stack）
    let template_json = fs::read_to_string(&template_path)?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let description_from_stack = template_json["Description"]
        .as_str()
        .and_then(|desc| Some(desc.to_string()));

    // メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
    let description_from_meta = match load_stack_meta(workspace_directory, stack_name) {
        Ok(meta_json) => meta_json["description"]
            .as_str()
            .and_then(|desc: &str| Some(desc.to_string())),
        Err(_) => {
            return Err(LoadStackError::App(AppError::new(
                "Failed to load stack meta",
            )));
        }
    };

    // アプリ返却用のデータ構造作成
    return Ok(Stack {
        name: stack_name.to_string(),
        description_from_meta: description_from_meta,
        description_from_stack: description_from_stack,
    });
}

#[tauri::command]
pub fn load_stacks_command(
    window: tauri::Window,
) -> Result<CommandResult<Vec<Stack>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_stacks(window_state.workspace_directory.as_str()) {
        Ok(stacks) => Ok(CommandResult::success(stacks)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_stack_command(
    window: tauri::Window,
    stack_name: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match delete_stack(window_state.workspace_directory.as_str(), stack_name) {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub fn import_stack_command(
    window: tauri::Window,
    stack_file_path: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match import_stack(window_state.workspace_directory.as_str(), stack_file_path) {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub fn load_stack_command(
    window: tauri::Window,
    stack_name: &str,
) -> Result<CommandResult<Stack>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_stack(window_state.workspace_directory.as_str(), stack_name) {
        Ok(stack) => Ok(CommandResult::success(stack)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
