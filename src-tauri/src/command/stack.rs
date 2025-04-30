use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::utils::get_window_state;
use crate::utils::AppError;
use crate::utils::CommandResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use super::workspace::load_workspace;
use super::workspace::update_workspace;
use super::workspace::StackInfo;
use super::workspace::Workspace;

// フロントで利用する形
#[derive(Debug, Serialize, Deserialize)]
pub struct Stack {
    id: String,
    name: String,
    description_from_meta: Option<String>,
    description_from_stack: Option<String>,
    exist: bool,
}

#[derive(Debug, Error)]
enum StackError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("workspace error: {0}")]
    Workspace(#[from] super::workspace::WorkspaceError),
}

fn load_stack_meta(workspace_directory: &str, stack_name: &str) -> Result<Value, StackError> {
    // ${スタック名}.meta.jsonが存在するか確認
    let meta_path = format!("{}/{}.meta.json", workspace_directory, stack_name);
    let meta_path = Path::new(&meta_path);
    if !meta_path.exists() {
        // ${スタック名}.meta.jsonを作成する
        fs::File::create(&meta_path)?;
        // 空のJSONを作成
        let empty_json = format!("{{\"name\":\"{}\"}}", stack_name);
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
async fn load_stacks(workspace_directory: &str) -> Result<Vec<Stack>, StackError> {
    // TODO: ディレクトリ配下のファイルをglobmatchで取得して、`stacks`に存在しないファイルも自動的に取り込む機能を追加する
    // globmatchを使って、workspace_directoryの直下にある*.template.jsonを取得する
    // let builder = globmatch::Builder::new("./*.template.json").build(workspace_directory);
    // let builder = match builder {
    //     Ok(builder) => builder,
    //     Err(_) => {
    //         return Err(LoadStackError::App(AppError::new("Failed to load stacks")));
    //     }
    // };
    // let paths: Vec<_> = builder.into_iter().flatten().collect();
    let workspace = load_workspace(workspace_directory).await?;
    let mut stacks = Vec::new();
    for stack in workspace.stacks.iter() {
        // スタックを読み込む
        let stack = match load_stack_from_info(workspace_directory, stack).await {
            Ok(stack) => stack,
            Err(err) => {
                print!("error: {:?}", err);
                return Err(StackError::App(AppError::new("Failed to load stack")));
            }
        };

        stacks.push(stack);
    }

    return Ok(stacks);
}

async fn delete_stack(workspace_directory: &str, stack_id: &str) -> Result<(), StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    for stack in workspace.stacks.iter() {
        if stack.id == stack_id {
            // スタックファイルとメタデータファイルを削除する
            fs::remove_file(format!("{}/{}", workspace_directory, stack.stack_file_name))?;
            fs::remove_file(format!(
                "{}/{}",
                workspace_directory,
                stack
                    .stack_file_name
                    .clone()
                    .replace(".template.json", ".meta.json")
            ))?;

            // workspace.jsonのstacksから削除する
            let stacks = workspace
                .stacks
                .iter()
                .filter(|s| s.id != stack_id)
                .cloned()
                .collect();
            let workspace = Workspace {
                name: workspace.name,
                description: workspace.description,
                stacks,
            };
            update_workspace(workspace_directory, workspace).await?;
            return Ok(());
        }
    }
    return Err(StackError::App(AppError::new("Stack not found")));
}

async fn import_stack(workspace_directory: &str, stack_file_path: &str) -> Result<(), StackError> {
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

    // workspace.jsonのstacksに追加する
    let workspace = load_workspace(workspace_directory).await?;
    let mut stacks = workspace.stacks.clone();
    stacks.push(StackInfo {
        id: Uuid::new_v4().to_string(),
        stack_file_name: filename.to_string(),
    });
    let workspace = Workspace {
        name: workspace.name,
        description: workspace.description,
        stacks,
    };
    update_workspace(workspace_directory, workspace).await?;

    return Ok(());
}

async fn load_stack_from_info(
    workspace_directory: &str,
    stack_info: &StackInfo,
) -> Result<Stack, StackError> {
    let template_path = format!("{}/{}", workspace_directory, stack_info.stack_file_name);

    // テンプレートファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_stack）
    let template_json = fs::read_to_string(&template_path)?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let description_from_stack: Option<String> = template_json["Description"]
        .as_str()
        .and_then(|desc| Some(desc.to_string()));

    // メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
    let meta_json = match load_stack_meta(
        workspace_directory,
        stack_info
            .stack_file_name
            .clone()
            .replace(".template.json", "")
            .as_str(),
    ) {
        Ok(meta_json) => meta_json,
        Err(_) => {
            return Err(StackError::App(AppError::new("Failed to load stack meta")));
        }
    };
    println!("meta_json: {:?}", meta_json);
    let stack_name = meta_json["name"]
        .as_str()
        .and_then(|name: &str| Some(name.to_string()))
        .unwrap_or(stack_info.stack_file_name.clone());
    let description_from_meta = meta_json["description"]
        .as_str()
        .and_then(|desc: &str| Some(desc.to_string()));

    // アプリ返却用のデータ構造作成
    return Ok(Stack {
        id: stack_info.id.clone(),
        name: stack_name,
        description_from_meta: description_from_meta,
        description_from_stack: description_from_stack,
        exist: true,
    });
}

async fn load_stack_from_id(
    workspace_directory: &str,
    stack_id: &str,
) -> Result<Stack, StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    for stack in workspace.stacks.iter() {
        if stack.id == stack_id {
            return load_stack_from_info(workspace_directory, stack).await;
        }
    }
    return Err(StackError::App(AppError::new("Stack not found")));
}

#[tauri::command]
pub async fn load_stacks_command(
    window: tauri::Window,
) -> Result<CommandResult<Vec<Stack>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_stacks(window_state.workspace_directory.as_str()).await {
        Ok(stacks) => Ok(CommandResult::success(stacks)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_stack_command(
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match delete_stack(window_state.workspace_directory.as_str(), stack_id).await {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_stack_command(
    window: tauri::Window,
    stack_file_path: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match import_stack(window_state.workspace_directory.as_str(), stack_file_path).await {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn load_stack_command(
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<Stack>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_stack_from_id(window_state.workspace_directory.as_str(), stack_id).await {
        Ok(stack) => Ok(CommandResult::success(stack)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
