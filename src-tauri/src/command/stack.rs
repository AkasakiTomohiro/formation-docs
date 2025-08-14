use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
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
use super::workspace::WorkspaceUpdate;

// フロントで利用する形
#[derive(Debug, Serialize, Deserialize)]
pub struct Stack {
    id: String,
    name: String,
    description_from_meta: Option<String>,
    description_from_stack: Option<String>,
    exist: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackMeta {
    pub name: String,
    pub description: String,
    pub reasons: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackMetaReasonsUpdate {
    pub logical_id: String,
    pub reasons: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackMetaUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub reasons: Option<StackMetaReasonsUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StackPropertiesUpdate {
    pub logical_id: String,
    pub properties: HashMap<String, Value>,
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

fn load_stack_meta(workspace_directory: &str, stack_name: &str) -> Result<StackMeta, StackError> {
    // ${スタック名}.meta.jsonが存在するか確認
    let meta_path = format!("{}/{}.meta.json", workspace_directory, stack_name);
    let meta_path = Path::new(&meta_path);
    if !meta_path.exists() {
        // ${スタック名}.meta.jsonを作成する
        fs::File::create(&meta_path)?;
        // 空のJSONを作成
        let empty_json = StackMeta {
            name: stack_name.to_string(),
            description: String::new(),
            reasons: HashMap::new(),
        };
        let empty_json = serde_json::to_string(&empty_json).unwrap();
        fs::write(&meta_path, empty_json)?;
    }

    // ${スタック名}.meta.jsonを読み込む
    let meta_json = fs::read_to_string(&meta_path)?;
    let meta_json = serde_json::from_str::<StackMeta>(&meta_json)?;
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
    for (id, file_name) in workspace.stacks.iter() {
        // スタックを読み込む
        let stack = match load_stack_from_info(workspace_directory, id, file_name).await {
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
    let mut workspace = load_workspace(workspace_directory).await?;

    match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => {
            // スタックファイルとメタデータファイルを削除する
            if let Err(err) =
                fs::remove_file(format!("{}/{}", workspace_directory, stack_file_name))
            {
                match err.kind() {
                    ErrorKind::NotFound => {
                        // ファイルが見つからない場合は無視する
                    }
                    _ => {
                        return Err(StackError::App(AppError::new(
                            "Failed to delete stack file",
                        )));
                    }
                }
            };
            if let Err(err) = fs::remove_file(format!(
                "{}/{}",
                workspace_directory,
                stack_file_name
                    .clone()
                    .replace(".template.json", ".meta.json")
            )) {
                match err.kind() {
                    ErrorKind::NotFound => {
                        // ファイルが見つからない場合は無視する
                    }
                    _ => {
                        return Err(StackError::App(AppError::new(
                            "Failed to delete stack file",
                        )));
                    }
                }
            };
            // workspace.jsonのstacksから削除する
            workspace.stacks.remove(stack_id);
            update_workspace(
                workspace_directory,
                WorkspaceUpdate {
                    name: None,
                    description: None,
                    stacks: Some(workspace.stacks.clone()),
                },
            )
            .await?;
        }
        None => return Err(StackError::App(AppError::new("Stack not found"))),
    };

    return Ok(());
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

    // スタックが存在する場合は追加しない
    if stacks.values().any(|v| v == filename) {
        println!("Stack with name {} already exists.", filename);
        return Ok(());
    }

    stacks.insert(Uuid::new_v4().to_string(), filename.to_string());
    update_workspace(
        workspace_directory,
        WorkspaceUpdate {
            name: None,
            description: None,
            stacks: Some(stacks.clone()),
        },
    )
    .await?;

    return Ok(());
}

async fn load_stack_from_info(
    workspace_directory: &str,
    stack_id: &str,
    stack_file_name: &str,
) -> Result<Stack, StackError> {
    let template_path = format!("{}/{}", workspace_directory, stack_file_name);

    // テンプレートファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_stack）
    let template_json = fs::read_to_string(&template_path)?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let description_from_stack: Option<String> = template_json["Description"]
        .as_str()
        .and_then(|desc| Some(desc.to_string()));

    // メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
    let meta_json = match load_stack_meta(
        workspace_directory,
        stack_file_name
            .to_string()
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

    // アプリ返却用のデータ構造作成
    return Ok(Stack {
        id: stack_id.to_string().clone(),
        name: meta_json.name,
        description_from_meta: Some(meta_json.description),
        description_from_stack: description_from_stack,
        exist: true,
    });
}

async fn load_stack_from_id(
    workspace_directory: &str,
    stack_id: &str,
) -> Result<Stack, StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    return match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => {
            load_stack_from_info(workspace_directory, stack_id, stack_file_name).await
        }
        None => Err(StackError::App(AppError::new("Stack not found"))),
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub service_name: String,
    pub recourse_type: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSummary {
    pub id: String,
    pub section_group_name: String,
    pub resources: Vec<Resource>,
}

async fn load_template_summary(
    workspace_directory: &str,
) -> Result<Vec<TemplateSummary>, StackError> {
    let mut templates = Vec::new();
    let workspace = load_workspace(workspace_directory).await?;
    for (id, stack_file_name) in workspace.stacks.iter() {
        let meta = load_stack_meta(
            workspace_directory,
            stack_file_name.replace(".template.json", "").as_str(),
        )?;

        let template_path = format!("{}/{}", workspace_directory, stack_file_name);
        let template_json = fs::read_to_string(&template_path)?;
        let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
        let resources_json: Value = template_json["Resources"].clone();

        let mut resources = HashMap::<String, HashSet<String>>::new();
        for (_, resource_value) in resources_json
            .as_object()
            .unwrap_or(&serde_json::Map::new())
            .iter()
        {
            // TODO: AWSリソースのみを対象とする
            let type_value = resource_value["Type"].clone();
            let parts: Vec<&str> = type_value.as_str().unwrap().split("::").collect();
            let (_, service_name, resource_type): (&str, &str, &str) = match parts[..] {
                [a, b, c] => (a, b, c),
                _ => continue,
            };
            if resources.contains_key(service_name) {
                let original = resources.get_mut(service_name).unwrap();
                original.insert(resource_type.to_string());
            } else {
                let mut resource_types = HashSet::new();
                resource_types.insert(resource_type.to_string());
                resources.insert(service_name.to_string(), resource_types);
            }
        }

        templates.push(TemplateSummary {
            id: id.to_string(),
            section_group_name: meta.name,
            resources: resources
                .iter()
                .map(|(key, value)| {
                    let service_name = key.to_string();
                    let resource_types = value.clone();
                    Resource {
                        service_name,
                        recourse_type: resource_types.iter().map(|v| v.to_string()).collect(),
                    }
                })
                .collect(),
        });
    }

    return Ok(templates);
}

async fn get_stack_resource_list(
    workspace_directory: &str,
    stack_id: &str,
    service_name: &str,
    resource_name: &str,
) -> Result<Vec<String>, StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = format!("{}/{}", workspace_directory, stack_file_name);
    let template_json = fs::read_to_string(&template_path)?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let resources_json = template_json["Resources"].clone();
    let mut result: Vec<String> = Vec::new();
    let resource_type = format!("AWS::{}::{}", service_name, resource_name);
    for (resource_id, resource_value) in resources_json
        .as_object()
        .unwrap_or(&serde_json::Map::new())
        .iter()
    {
        let tmp_resource_type = resource_value["Type"].as_str().unwrap_or_default();
        if tmp_resource_type == resource_type {
            result.push(resource_id.to_string());
        }
    }

    return Ok(result);
}

async fn get_stack_resource_properties(
    workspace_directory: &str,
    stack_id: &str,
    logical_id: &str,
) -> Result<Value, StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = format!("{}/{}", workspace_directory, stack_file_name);
    let template_json = fs::read_to_string(&template_path)?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let properties_json = template_json["Resources"][logical_id]["Properties"].clone();
    if properties_json.is_null() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    return Ok(properties_json);
}

async fn get_stack_resource_properties_reasons(
    workspace_directory: &str,
    stack_id: &str,
    logical_id: &str,
) -> Result<Value, StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let meta_path = format!(
        "{}/{}.meta.json",
        workspace_directory,
        stack_file_name.replace(".template.json", "")
    );
    let meta_json = fs::read_to_string(&meta_path)?;
    let meta_json: Value = serde_json::from_str(&meta_json).unwrap_or_default();
    let reasons_json = meta_json["reasons"][logical_id].clone();
    if reasons_json.is_null() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    return Ok(reasons_json);
}

async fn get_stack_parameters(
    workspace_directory: &str,
    stack_id: &str,
) -> Result<Value, StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = format!("{}/{}", workspace_directory, stack_file_name);
    let template_json = fs::read_to_string(&template_path)?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let parameters_json = template_json["Parameters"].clone();
    if parameters_json.is_null() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    return Ok(parameters_json);
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackOutput {
    name: String,
    description: Option<String>,
    export_name: Option<String>,
    value: Value,
}

/// 対象スタックのOutputsを取得する
///
/// 対象スタックのOutputsがない場合は、空配列を返す
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
/// - `stack_id` - スタックのID
async fn get_stack_outputs(
    workspace_directory: &str,
    stack_id: &str,
) -> Result<Vec<StackOutput>, StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = format!("{}/{}", workspace_directory, stack_file_name);
    let template_json = fs::read_to_string(&template_path)?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let outputs_json = template_json["Outputs"].clone();

    let mut result: Vec<StackOutput> = Vec::new();
    if let Some(outputs) = outputs_json.as_object() {
        for (key, value) in outputs.iter() {
            let result_item = StackOutput {
                name: key.to_string(),
                description: value
                    .get("Description")
                    .and_then(|desc| desc.as_str())
                    .map(|s| s.to_string()),
                export_name: value
                    .get("Export")
                    .and_then(|export| export.get("Name"))
                    .and_then(|name| name.as_str())
                    .map(|str| str.to_string()),
                value: value["Value"].clone(),
            };
            result.push(result_item);
        }
    }

    return Ok(result);
}

async fn update_stack_meta(
    workspace_directory: &str,
    stack_id: &str,
    update_stack_meta: StackMetaUpdate,
) -> Result<(), StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    let stack_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name.replace(".template.json", ""),
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let stack_meta = load_stack_meta(workspace_directory, &stack_name)?;
    let new_stack_meta = StackMeta {
        name: update_stack_meta.name.unwrap_or(stack_meta.name),
        description: update_stack_meta
            .description
            .unwrap_or(stack_meta.description),
        reasons: match update_stack_meta.reasons {
            Some(update_meta) => {
                let mut new_reasons = stack_meta.reasons.clone();
                new_reasons.insert(update_meta.logical_id, update_meta.reasons);
                new_reasons
            }
            None => stack_meta.reasons,
        },
    };
    let stack_meta_path = format!("{}/{}.meta.json", workspace_directory, stack_name);
    let stack_meta_json = serde_json::to_string(&new_stack_meta).unwrap();
    fs::write(&stack_meta_path, stack_meta_json)?;
    return Ok(());
}

async fn update_stack_detail(
    workspace_directory: &str,
    stack_id: &str,
    update_stack_meta: StackMetaUpdate,
) -> Result<(), StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    let stack_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name.replace(".template.json", ""),
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let stack_meta = load_stack_meta(workspace_directory, &stack_name)?;
    let new_stack_meta = StackMeta {
        name: update_stack_meta.name.unwrap_or(stack_meta.name),
        description: update_stack_meta
            .description
            .unwrap_or(stack_meta.description),
        reasons: match update_stack_meta.reasons {
            Some(update_meta) => {
                let mut new_reasons = stack_meta.reasons.clone();
                new_reasons.insert(update_meta.logical_id, update_meta.reasons);
                new_reasons
            }
            None => stack_meta.reasons,
        },
    };
    let stack_meta_path = format!("{}/{}.meta.json", workspace_directory, stack_name);
    let stack_meta_json = serde_json::to_string(&new_stack_meta).unwrap();
    fs::write(&stack_meta_path, stack_meta_json)?;
    return Ok(());
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AWSServiceResource {
    pub service_name: String,
    pub recourse_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterAndResourceList {
    pub parameters: Vec<String>,
    pub resources: HashMap<String, AWSServiceResource>,
}

async fn load_parameter_and_resource_list(
    workspace_directory: &str,
    stack_id: &str,
) -> Result<ParameterAndResourceList, StackError> {
    let workspace = load_workspace(workspace_directory).await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = format!("{}/{}", workspace_directory, stack_file_name);
    let template_json = fs::read_to_string(&template_path)?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();

    let parameters_json = template_json["Parameters"].clone();
    let parameters: Vec<String> = match parameters_json.as_object() {
        Some(params) => params.keys().cloned().collect(),
        None => Vec::new(),
    };

    let resources_json = template_json["Resources"].clone();
    let mut resources: HashMap<String, AWSServiceResource> = HashMap::new();
    for (resource_id, resource_value) in resources_json
        .as_object()
        .unwrap_or(&serde_json::Map::new())
        .iter()
    {
        let tmp_resource_type = resource_value["Type"].as_str().unwrap_or_default();
        let parts: Vec<&str> = tmp_resource_type.split("::").collect();
        let (_, service_name, resource_type): (&str, &str, &str) = match parts[..] {
            [a, b, c] => (a, b, c),
            _ => continue,
        };
        resources.insert(
            resource_id.to_string(),
            AWSServiceResource {
                service_name: service_name.to_string(),
                recourse_type: resource_type.to_string(),
            },
        );
    }

    return Ok(ParameterAndResourceList {
        parameters,
        resources,
    });
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

#[tauri::command]
pub async fn load_template_summary_command(
    window: tauri::Window,
) -> Result<CommandResult<Vec<TemplateSummary>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_template_summary(window_state.workspace_directory.as_str()).await {
        Ok(templates) => Ok(CommandResult::success(templates)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_resource_list_command(
    window: tauri::Window,
    stack_id: &str,
    service_name: &str,
    resource_name: &str,
) -> Result<CommandResult<Vec<String>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_stack_resource_list(
        window_state.workspace_directory.as_str(),
        stack_id,
        service_name,
        resource_name,
    )
    .await
    {
        Ok(resources) => Ok(CommandResult::success(resources)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_resource_properties_command(
    window: tauri::Window,
    stack_id: &str,
    logical_id: &str,
) -> Result<CommandResult<Value>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_stack_resource_properties(
        window_state.workspace_directory.as_str(),
        stack_id,
        logical_id,
    )
    .await
    {
        Ok(properties) => Ok(CommandResult::success(properties)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_parameters_command(
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<Value>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_stack_parameters(window_state.workspace_directory.as_str(), stack_id).await {
        Ok(parameters) => Ok(CommandResult::success(parameters)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_outputs_command(
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<Vec<StackOutput>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_stack_outputs(window_state.workspace_directory.as_str(), stack_id).await {
        Ok(parameters) => Ok(CommandResult::success(parameters)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_stack_meta_command(
    window: tauri::Window,
    stack_id: &str,
    logical_id: &str,
    reasons: HashMap<String, String>,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match update_stack_meta(
        window_state.workspace_directory.as_str(),
        stack_id,
        StackMetaUpdate {
            name: None,
            description: None,
            reasons: Some(StackMetaReasonsUpdate {
                logical_id: logical_id.to_string(),
                reasons: reasons,
            }),
        },
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_resource_properties_reasons_command(
    window: tauri::Window,
    stack_id: &str,
    logical_id: &str,
) -> Result<CommandResult<Value>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_stack_resource_properties_reasons(
        window_state.workspace_directory.as_str(),
        stack_id,
        logical_id,
    )
    .await
    {
        Ok(reasons) => Ok(CommandResult::success(reasons)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_stack_detail_command(
    window: tauri::Window,
    stack_id: &str,
    name: &str,
    description: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match update_stack_detail(
        window_state.workspace_directory.as_str(),
        stack_id,
        StackMetaUpdate {
            name: Some(name.to_string()),
            description: Some(description.to_string()),
            reasons: None,
        },
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn load_parameter_and_resource_list_command(
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<ParameterAndResourceList>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_parameter_and_resource_list(
        window_state.workspace_directory.as_str(),
        stack_id,
    )
    .await
    {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
