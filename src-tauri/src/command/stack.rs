use std::fs;

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
pub enum LoadStacksError {
    #[error("app error: {0}")]
    App(#[from] AppError),
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
        let template_json = match fs::read_to_string(&path) {
            Ok(json) => json,
            Err(_) => {
                return Err(LoadStacksError::App(AppError::new(
                    "Failed to read template.json",
                )));
            }
        };
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
        let meta_path = path.with_file_name(format!("{}.meta.json", filename));
        let meta_json = match fs::read_to_string(&meta_path) {
            Ok(json) => json,
            Err(_) => {
                return Err(LoadStacksError::App(AppError::new(
                    "Failed to read meta.json",
                )));
            }
        };
        let meta_json: Value = serde_json::from_str(&meta_json).unwrap_or_default();

        // メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
        let description_from_meta = meta_json["description"]
            .as_str()
            .and_then(|desc: &str| Some(desc.to_string()));

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
pub enum DeleteStacksError {
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
