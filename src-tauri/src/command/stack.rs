use std::collections::HashMap;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::command::context::command_context::CommandContext;
use crate::config::context::config_context::ConfigContext;
use crate::config::stack_meta_config::StackMetaConfigError;
use crate::config::workspace_config::WorkspaceConfigError;
use crate::utils::context::app_context::AppContext;
use crate::utils::get_window_state;
use crate::utils::AppError;
use crate::utils::CommandResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use thiserror::Error;
use uuid::Uuid;

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
pub enum StackError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("workspace config error: {0}")]
    WorkspaceConfig(#[from] WorkspaceConfigError),
    #[error("workspace command error: {0}")]
    WorkspaceCommand(#[from] super::workspace::WorkspaceCommandError),
    #[error("stack meta config error: {0}")]
    StackMetaConfig(#[from] StackMetaConfigError),
}

/**
 * 1. globmatchを使って、workspace_directoryの直下にある*.template.jsonを取得する
 *  1. テンプレートファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_stack）
 *  2. ${スタック名}.meta.jsonを取得する（description_from_metaと各リソースとそのパラメータの説明文）
 *  3. メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
 */
async fn load_stacks(
    app_context: &AppContext,
    config_context: &ConfigContext,
    command_context: &CommandContext,
    workspace_directory: &str,
) -> Result<Vec<Stack>, StackError> {
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
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let mut stacks = Vec::new();
    for (id, file_name) in workspace.stacks.iter() {
        // スタックを読み込む
        let stack = match command_context
            .stack
            .load_stack_from_info(
                app_context,
                config_context,
                workspace_directory,
                id,
                file_name,
            )
            .await
        {
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

async fn delete_stack(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
) -> Result<(), StackError> {
    let mut workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;

    match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => {
            let stack_file_path = PathBuf::from(workspace_directory).join(stack_file_name);
            // スタックファイルとメタデータファイルを削除する
            if let Err(err) = app_context.file_system.remove_file(&stack_file_path).await {
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
            let file_path = PathBuf::from(workspace_directory).join(
                stack_file_name
                    .clone()
                    .replace(".template.json", ".meta.json"),
            );
            if let Err(err) = app_context.file_system.remove_file(&file_path).await {
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
            config_context
                .workspace_config_io
                .write(
                    workspace,
                    app_context.file_system.clone(),
                    workspace_directory,
                )
                .await?;
        }
        None => return Err(StackError::App(AppError::new("Stack not found"))),
    };

    return Ok(());
}

async fn import_stack(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_file_path: &str,
) -> Result<(), StackError> {
    let stack_file_path_buf = PathBuf::from(stack_file_path);
    let filename = stack_file_path_buf
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let mut copy_file_path = PathBuf::from(workspace_directory).join(filename);

    // インポート元と先が同じファイルパスでない場合のみコピー
    if copy_file_path.to_str().unwrap() != stack_file_path {
        if filename.ends_with(".yaml") || filename.ends_with(".yml") {
            // YAMLファイルをJSONに変換して保存
            let yaml_content = app_context
                .file_system
                .read_file(&stack_file_path_buf)
                .await?;
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;
            let json_content = serde_json::to_string_pretty(&yaml_value)?;
            copy_file_path.set_extension("json");
            app_context
                .file_system
                .write_file(&copy_file_path, json_content.as_bytes())
                .await?;
        } else {
            // 通常のコピー処理
            app_context
                .file_system
                .copy_file(&stack_file_path_buf, &copy_file_path)
                .await?;
        }
    }

    // workspace.jsonのstacksに追加する
    let mut workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;

    // スタックが存在する場合は追加しない
    if workspace.stacks.values().any(|v| v == filename) {
        println!("Stack with name {} already exists.", filename);
        return Ok(());
    }

    workspace
        .stacks
        .insert(Uuid::new_v4().to_string(), filename.to_string());
    config_context
        .workspace_config_io
        .write(
            workspace,
            app_context.file_system.clone(),
            workspace_directory,
        )
        .await?;

    return Ok(());
}

pub async fn load_stack_from_info(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
    stack_file_name: &str,
) -> Result<Stack, StackError> {
    let template_path = PathBuf::from(workspace_directory).join(stack_file_name);

    // テンプレートファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_stack）
    let template_json = app_context.file_system.read_file(&template_path).await?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let description_from_stack: Option<String> = template_json["Description"]
        .as_str()
        .and_then(|desc| Some(desc.to_string()));

    // メタファイルのdescriptionフィールドを取得する。Optionalな場合もある。（description_from_meta）
    let meta_json = match config_context
        .stack_meta_config_io
        .read(
            app_context.file_system.clone(),
            workspace_directory,
            stack_file_name
                .to_string()
                .clone()
                .replace(".template.json", "")
                .as_str(),
        )
        .await
    {
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
    app_context: &AppContext,
    config_context: &ConfigContext,
    command_context: &CommandContext,
    workspace_directory: &str,
    stack_id: &str,
) -> Result<Stack, StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    return match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => {
            command_context
                .stack
                .load_stack_from_info(
                    app_context,
                    config_context,
                    workspace_directory,
                    stack_id,
                    stack_file_name,
                )
                .await
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
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
) -> Result<Vec<TemplateSummary>, StackError> {
    let mut templates = Vec::new();
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    for (id, stack_file_name) in workspace.stacks.iter() {
        let meta = config_context
            .stack_meta_config_io
            .read(
                app_context.file_system.clone(),
                workspace_directory,
                stack_file_name.replace(".template.json", "").as_str(),
            )
            .await?;

        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        let template_json = app_context.file_system.read_file(&template_path).await?;
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
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
    service_name: &str,
    resource_name: &str,
) -> Result<Vec<String>, StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
    let template_json = app_context.file_system.read_file(&template_path).await?;
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
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
    logical_id: &str,
) -> Result<Value, StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
    let template_json = app_context.file_system.read_file(&template_path).await?;
    let template_json: Value = serde_json::from_str(&template_json).unwrap_or_default();
    let properties_json = template_json["Resources"][logical_id]["Properties"].clone();
    if properties_json.is_null() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    return Ok(properties_json);
}

async fn get_stack_resource_properties_reasons(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
    logical_id: &str,
) -> Result<Value, StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let meta_path = PathBuf::from(format!(
        "{}/{}.meta.json",
        workspace_directory,
        stack_file_name.replace(".template.json", "")
    ));
    let meta_json = app_context.file_system.read_file(&meta_path).await?;
    let meta_json: Value = serde_json::from_str(&meta_json).unwrap_or_default();
    let reasons_json = meta_json["reasons"][logical_id].clone();
    if reasons_json.is_null() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    return Ok(reasons_json);
}

async fn get_stack_parameters(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
) -> Result<Value, StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
    let template_json = app_context.file_system.read_file(&template_path).await?;
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
pub async fn get_stack_outputs(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
) -> Result<Vec<StackOutput>, StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
    let template_json = app_context.file_system.read_file(&template_path).await?;
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

/// ワークスペース内の全スタックのOutputsを取得する
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
async fn get_all_stack_outputs(
    app_context: &AppContext,
    config_context: &ConfigContext,
    command_context: &CommandContext,
    workspace_directory: &str,
) -> Result<HashMap<String, String>, StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    // {Outputsのexport_name: スタックID}の形で返す
    let mut result: HashMap<String, String> = HashMap::new();
    for (stack_id, _) in workspace.stacks.iter() {
        let outputs = command_context
            .stack
            .get_stack_outputs(app_context, config_context, workspace_directory, stack_id)
            .await?;
        for output in outputs.iter().filter(|o| o.export_name.is_some()) {
            result.insert(
                output.export_name.as_ref().unwrap().to_string(),
                stack_id.to_string(),
            );
        }
    }
    return Ok(result);
}

async fn update_stack_reasons(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
    logical_id: &str,
    reasons: HashMap<String, String>,
) -> Result<(), StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let stack_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name.replace(".template.json", ""),
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let mut stack_meta = config_context
        .stack_meta_config_io
        .read(
            app_context.file_system.clone(),
            workspace_directory,
            &stack_name,
        )
        .await?;
    stack_meta.reasons.insert(logical_id.to_string(), reasons);
    config_context
        .stack_meta_config_io
        .write(
            stack_meta,
            app_context.file_system.clone(),
            workspace_directory,
            &stack_name,
        )
        .await?;
    return Ok(());
}

async fn update_stack_detail(
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
    name: &str,
    description: &str,
) -> Result<(), StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let stack_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name.replace(".template.json", ""),
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let mut stack_meta = config_context
        .stack_meta_config_io
        .read(
            app_context.file_system.clone(),
            workspace_directory,
            &stack_name,
        )
        .await?;
    stack_meta.name = name.to_string();
    stack_meta.description = description.to_string();
    stack_meta = config_context
        .stack_meta_config_io
        .write(
            stack_meta,
            app_context.file_system.clone(),
            workspace_directory,
            &stack_name,
        )
        .await?;
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
    app_context: &AppContext,
    config_context: &ConfigContext,
    workspace_directory: &str,
    stack_id: &str,
) -> Result<ParameterAndResourceList, StackError> {
    let workspace = config_context
        .workspace_config_io
        .read(app_context.file_system.clone(), workspace_directory)
        .await?;
    let stack_file_name = match workspace.stacks.get(stack_id) {
        Some(stack_file_name) => stack_file_name,
        None => {
            return Err(StackError::App(AppError::new("Stack not found")));
        }
    };
    let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
    let template_json = app_context.file_system.read_file(&template_path).await?;
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

#[coverage(off)]
#[tauri::command]
pub async fn load_stacks_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    command_context_state: State<'_, CommandContext>,
    window: tauri::Window,
) -> Result<CommandResult<Vec<Stack>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_stacks(
        &app_context_state,
        &config_context_state,
        &command_context_state,
        window_state.workspace_directory.as_str(),
    )
    .await
    {
        Ok(stacks) => Ok(CommandResult::success(stacks)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_stack_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match delete_stack(
        &app_context_state,
        &config_context_state,
        window_state.workspace_directory.as_str(),
        stack_id,
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn import_stack_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    window: tauri::Window,
    stack_file_path: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match import_stack(
        &app_context_state,
        &config_context_state,
        window_state.workspace_directory.as_str(),
        stack_file_path,
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn load_stack_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    command_context_state: State<'_, CommandContext>,
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<Stack>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_stack_from_id(
        &app_context_state,
        &config_context_state,
        &command_context_state,
        window_state.workspace_directory.as_str(),
        stack_id,
    )
    .await
    {
        Ok(stack) => Ok(CommandResult::success(stack)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command]
pub async fn load_template_summary_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    window: tauri::Window,
) -> Result<CommandResult<Vec<TemplateSummary>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_template_summary(
        &app_context_state,
        &config_context_state,
        window_state.workspace_directory.as_str(),
    )
    .await
    {
        Ok(templates) => Ok(CommandResult::success(templates)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_resource_list_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
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
        &app_context_state,
        &config_context_state,
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

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_resource_properties_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
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
        &app_context_state,
        &config_context_state,
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

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_parameters_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<Value>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_stack_parameters(
        &app_context_state,
        &config_context_state,
        window_state.workspace_directory.as_str(),
        stack_id,
    )
    .await
    {
        Ok(parameters) => Ok(CommandResult::success(parameters)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_outputs_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    command_context_state: State<'_, CommandContext>,
    window: tauri::Window,
    stack_id: &str,
) -> Result<CommandResult<Vec<StackOutput>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match command_context_state
        .stack
        .get_stack_outputs(
            &app_context_state,
            &config_context_state,
            window_state.workspace_directory.as_str(),
            stack_id,
        )
        .await
    {
        Ok(parameters) => Ok(CommandResult::success(parameters)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_all_stack_outputs_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
    command_context_state: State<'_, CommandContext>,
    window: tauri::Window,
) -> Result<CommandResult<HashMap<String, String>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_all_stack_outputs(
        &app_context_state,
        &config_context_state,
        &command_context_state,
        window_state.workspace_directory.as_str(),
    )
    .await
    {
        Ok(outputs) => Ok(CommandResult::success(outputs)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn update_stack_meta_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
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
    return match update_stack_reasons(
        &app_context_state,
        &config_context_state,
        window_state.workspace_directory.as_str(),
        stack_id,
        logical_id,
        reasons,
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_stack_resource_properties_reasons_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
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
        &app_context_state,
        &config_context_state,
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

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn update_stack_detail_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
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
        &app_context_state,
        &config_context_state,
        window_state.workspace_directory.as_str(),
        stack_id,
        name,
        description,
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn load_parameter_and_resource_list_command(
    app_context_state: State<'_, AppContext>,
    config_context_state: State<'_, ConfigContext>,
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
        &app_context_state,
        &config_context_state,
        window_state.workspace_directory.as_str(),
        stack_id,
    )
    .await
    {
        Ok(result) => Ok(CommandResult::success(result)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[cfg(test)]
#[coverage(off)]
mod load_stacks_tests {
    use std::sync::Arc;

    use crate::{
        command::context::{
            app_config_command_trait::MockAppConfigCommandTrait,
            cloudformation_schema_trait::MockCloudFormationSchemaTrait,
            manual_management_resource_trait::MockManualManagementResourceTrait,
            stack_command_trait::MockStackCommandTrait,
        },
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };

    use super::*;

    /// stacksの取得に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_command_app_config = MockAppConfigCommandTrait::new();
        let mut mock_command_stack = MockStackCommandTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack";
        let mut stacks = HashMap::new();
        // 最後の一文字を変えて2つのスタックを用意
        stacks.insert(format!("{}1", stack_id), format!("{}1", stack_file_name));
        stacks.insert(format!("{}2", stack_id), format!("{}2", stack_file_name));

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // load_stack_from_infoのモック設定
        let description_from_meta = "description_from_meta sample";
        let description_from_stack = "description_from_stack sample";
        mock_command_stack
            .expect_load_stack_from_info()
            .times(2)
            .withf(
                move |_app_context, _config_context, workspace_dir, id, file_name| {
                    workspace_dir == workspace_directory
                        && (id == format!("{}1", stack_id.to_string(),)
                            || id == format!("{}2", stack_id.to_string(),))
                        && (file_name == format!("{}1", stack_file_name)
                            || file_name == format!("{}2", stack_file_name))
                },
            )
            .returning({
                move |_app_context, _config_context, _workspace_dir, id, file_name| {
                    Ok(Stack {
                        id: id.to_string(),
                        name: file_name.to_string(),
                        description_from_meta: Some(description_from_meta.to_string()),
                        description_from_stack: Some(description_from_stack.to_string()),
                        exist: true,
                    })
                }
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let command_context = CommandContext {
            app_config: Arc::new(mock_command_app_config),
            stack: Arc::new(mock_command_stack),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = load_stacks(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        // stacksが正しく取得できていること
        assert!(result.is_ok());
        let mut stacks = result.unwrap();
        assert_eq!(stacks.len(), 2);
        stacks.sort_by(|a, b| a.id.cmp(&b.id)); // IDでソートして順序を固定
        for (i, stack) in stacks.iter().enumerate() {
            let index = i + 1;
            assert_eq!(stack.id, format!("{}{}", stack_id, index));
            assert_eq!(stack.name, format!("{}{}", stack_file_name, index));
            assert_eq!(
                stack.description_from_meta,
                Some(description_from_meta.to_string())
            );
            assert_eq!(
                stack.description_from_stack,
                Some(description_from_stack.to_string())
            );
            assert_eq!(stack.exist, true);
        }
    }

    // workspaceにスタックがない場合、空配列が返ること
    #[tokio::test]
    async fn success_no_stacks() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_command_app_config = MockAppConfigCommandTrait::new();
        let mock_command_stack = MockStackCommandTrait::new();

        let workspace_directory = "test/sample_workspace";

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: HashMap::new(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let command_context = CommandContext {
            app_config: Arc::new(mock_command_app_config),
            stack: Arc::new(mock_command_stack),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = load_stacks(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        // 空配列が返ること
        assert!(result.is_ok());
        let stacks = result.unwrap();
        assert_eq!(stacks.len(), 0);
    }

    // workspace_configのreadに失敗した場合、エラーになること
    #[tokio::test]
    async fn workspace_config_read_error() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_command_app_config = MockAppConfigCommandTrait::new();
        let mock_command_stack = MockStackCommandTrait::new();

        let workspace_directory = "test/sample_workspace";

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Err(WorkspaceConfigError::App(AppError::new(
                    "Failed to read workspace config",
                )))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let command_context = CommandContext {
            app_config: Arc::new(mock_command_app_config),
            stack: Arc::new(mock_command_stack),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = load_stacks(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        // エラーになること
        assert!(result.is_err());
    }

    // load_stack_from_infoでエラーが発生した場合、エラーになること
    #[tokio::test]
    async fn load_stack_from_info_error() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_command_app_config = MockAppConfigCommandTrait::new();
        let mut mock_command_stack = MockStackCommandTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack";
        let stack_file_name_json = format!("{}.template.json", stack_file_name);
        let mut stacks = HashMap::new();
        stacks.insert(
            format!("{}1", stack_id),
            format!("{}1", stack_file_name_json),
        );
        stacks.insert(
            format!("{}2", stack_id),
            format!("{}2", stack_file_name_json),
        );

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // load_stack_from_infoのモック設定
        mock_command_stack.expect_load_stack_from_info().returning({
            move |_app_context, _config_context, _workspace_dir, id, file_name| {
                Err(StackError::App(AppError::new(
                    format!(
                        "Failed to load stack from info: id={}, file_name={}",
                        id, file_name
                    )
                    .as_str(),
                )))
            }
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let command_context = CommandContext {
            app_config: Arc::new(mock_command_app_config),
            stack: Arc::new(mock_command_stack),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = load_stacks(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        // load_stackがエラーになること
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod delete_stack_tests {
    use std::sync::Arc;

    use crate::{
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };

    use super::*;

    // stackの削除に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack";
        let stack_file_name_json = format!("{}.template.json", stack_file_name);
        let meta_file_name_json = format!("{}.meta.json", stack_file_name);
        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name_json.clone());
        stacks.insert(
            "other_stack_id".to_string(),
            "other_stack.template.json".to_string(),
        );
        let deleted_workspace = WorkspaceConfig {
            version: 1,
            stacks: stacks.clone(),
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // workspace_configのreadのモック設定
        let deleted_workspace_clone = deleted_workspace.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| Ok(deleted_workspace_clone.clone()));

        // remove_fileのモック設定
        let stack_file_path = PathBuf::from(workspace_directory).join(stack_file_name_json);
        let stack_meta_file_path = PathBuf::from(workspace_directory).join(meta_file_name_json);
        mock_file_system
            .expect_remove_file()
            .times(2)
            .withf(move |path| path == &stack_file_path || path == &stack_meta_file_path)
            .returning(move |_path| Ok(()));

        // workspace_configのwriteのモック設定
        let updated_stacks = {
            let mut map = HashMap::new();
            map.insert(
                "other_stack_id".to_string(),
                "other_stack.template.json".to_string(),
            );
            map
        };
        let updated_stacks_clone = updated_stacks.clone();
        let deleted_workspace_clone1 = deleted_workspace.clone();
        mock_workspace_config
            .expect_write()
            .withf(move |workspace, _file_system, workspace_dir| {
                return workspace.version == 1
                    && workspace.stacks == updated_stacks_clone
                    && workspace.name == deleted_workspace.name
                    && workspace.description == deleted_workspace.description
                    && workspace_dir == workspace_directory;
            })
            .returning(move |_workspace, _file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: updated_stacks.clone(),
                    name: deleted_workspace_clone1.name.clone(),
                    description: deleted_workspace_clone1.description.clone(),
                })
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_stack(&app_context, &config_context, workspace_directory, stack_id).await;

        // ######### 検証 #########
        // stackが正しく削除できていること
        assert!(result.is_ok());
    }

    // remove_fileでファイルが見つからない場合でも成功すること
    #[tokio::test]
    async fn success_remove_file_not_found() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack";
        let stack_file_name_json = format!("{}.template.json", stack_file_name);
        let meta_file_name_json = format!("{}.meta.json", stack_file_name);
        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name_json.clone());
        stacks.insert(
            "other_stack_id".to_string(),
            "other_stack.template.json".to_string(),
        );
        let deleted_workspace = WorkspaceConfig {
            version: 1,
            stacks: stacks.clone(),
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // workspace_configのreadのモック設定
        let deleted_workspace_clone = deleted_workspace.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| Ok(deleted_workspace_clone.clone()));

        // remove_fileのモック設定
        let stack_file_path = PathBuf::from(workspace_directory).join(stack_file_name_json);
        let stack_meta_file_path = PathBuf::from(workspace_directory).join(meta_file_name_json);
        mock_file_system
            .expect_remove_file()
            .times(2)
            .withf(move |path| path == &stack_file_path || path == &stack_meta_file_path)
            .returning(move |_path| {
                Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::NotFound,
                    "file not found",
                ))
            });

        // workspace_configのwriteのモック設定
        let updated_stacks = {
            let mut map = HashMap::new();
            map.insert(
                "other_stack_id".to_string(),
                "other_stack.template.json".to_string(),
            );
            map
        };
        let updated_stacks_clone = updated_stacks.clone();
        let deleted_workspace_clone1 = deleted_workspace.clone();
        mock_workspace_config
            .expect_write()
            .withf(move |workspace, _file_system, workspace_dir| {
                return workspace.version == 1
                    && workspace.stacks == updated_stacks_clone
                    && workspace.name == deleted_workspace.name
                    && workspace.description == deleted_workspace.description
                    && workspace_dir == workspace_directory;
            })
            .returning(move |_workspace, _file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: updated_stacks.clone(),
                    name: deleted_workspace_clone1.name.clone(),
                    description: deleted_workspace_clone1.description.clone(),
                })
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_stack(&app_context, &config_context, workspace_directory, stack_id).await;

        // ######### 検証 #########
        // エラーを返さないこと
        assert!(result.is_ok());
    }

    // workspace_configのreadに失敗した場合、エラーになること
    #[tokio::test]
    async fn workspace_config_read_error() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Err(WorkspaceConfigError::App(AppError::new(
                    "Failed to read workspace config",
                )))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_stack(&app_context, &config_context, workspace_directory, stack_id).await;

        // ######### 検証 #########
        // delete_stackがエラーになること
        assert!(result.is_err());
    }

    // stack_idが一致するスタックがworkspaceに存在しない場合、エラーになること
    #[tokio::test]
    async fn stack_none_error() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";
        let mut stacks = HashMap::new();
        stacks.insert(
            "other_stack_id".to_string(),
            "other_stack.template.json".to_string(),
        );
        let deleted_workspace = WorkspaceConfig {
            version: 1,
            stacks: stacks.clone(),
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // workspace_configのreadのモック設定
        let deleted_workspace_clone = deleted_workspace.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| Ok(deleted_workspace_clone.clone()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_stack(&app_context, &config_context, workspace_directory, stack_id).await;

        // ######### 検証 #########
        // delete_stackがエラーになること
        assert!(result.is_err());
    }

    // 1回目のremove_fileでその他のエラーが発生した場合、エラーになること
    #[tokio::test]
    async fn remove_file_other_error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack";
        let stack_file_name_json = format!("{}.template.json", stack_file_name);
        let meta_file_name_json = format!("{}.meta.json", stack_file_name);
        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name_json.clone());
        stacks.insert(
            "other_stack_id".to_string(),
            "other_stack.template.json".to_string(),
        );
        let deleted_workspace = WorkspaceConfig {
            version: 1,
            stacks: stacks.clone(),
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // workspace_configのreadのモック設定
        let deleted_workspace_clone = deleted_workspace.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| Ok(deleted_workspace_clone.clone()));

        // remove_fileのモック設定
        let stack_file_path = PathBuf::from(workspace_directory).join(stack_file_name_json);
        let stack_meta_file_path = PathBuf::from(workspace_directory).join(meta_file_name_json);
        mock_file_system
            .expect_remove_file()
            .times(1)
            .withf(move |path| path == &stack_file_path || path == &stack_meta_file_path)
            .returning({
                let mut call_count = 0;
                move |_path| {
                    call_count += 1;
                    match call_count {
                        1 => Err(tokio::io::Error::new(
                            tokio::io::ErrorKind::Other,
                            "remove_file error",
                        )), // 1回目の呼び出しでErrを返す
                        _ => panic!("Unexpected call"),
                    }
                }
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_stack(&app_context, &config_context, workspace_directory, stack_id).await;

        // ######### 検証 #########
        // delete_stackがエラーになること
        assert!(result.is_err());
    }

    // 2回目のremove_fileでその他のエラーが発生した場合、エラーになること
    #[tokio::test]
    async fn remove_file_other_error_second() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack";
        let stack_file_name_json = format!("{}.template.json", stack_file_name);
        let meta_file_name_json = format!("{}.meta.json", stack_file_name);
        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name_json.clone());
        stacks.insert(
            "other_stack_id".to_string(),
            "other_stack.template.json".to_string(),
        );
        let deleted_workspace = WorkspaceConfig {
            version: 1,
            stacks: stacks.clone(),
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // workspace_configのreadのモック設定
        let deleted_workspace_clone = deleted_workspace.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| Ok(deleted_workspace_clone.clone()));

        // remove_fileのモック設定
        let stack_file_path = PathBuf::from(workspace_directory).join(stack_file_name_json);
        let stack_meta_file_path = PathBuf::from(workspace_directory).join(meta_file_name_json);
        mock_file_system
            .expect_remove_file()
            .times(2)
            .withf(move |path| path == &stack_file_path || path == &stack_meta_file_path)
            .returning({
                let mut call_count = 0;
                move |_path| {
                    call_count += 1;
                    match call_count {
                        1 => Ok(()),
                        2 => Err(tokio::io::Error::new(
                            tokio::io::ErrorKind::Other,
                            "remove_file error",
                        )), // 2回目の呼び出しでErrを返す
                        _ => panic!("Unexpected call"),
                    }
                }
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_stack(&app_context, &config_context, workspace_directory, stack_id).await;

        // ######### 検証 #########
        // delete_stackがエラーになること
        assert!(result.is_err());
    }

    // workspace_configのwriteに失敗した場合、エラーになること
    #[tokio::test]
    async fn workspace_config_write_error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";

        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack";
        let stack_file_name_json = format!("{}.template.json", stack_file_name);
        let meta_file_name_json = format!("{}.meta.json", stack_file_name);
        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name_json.clone());
        stacks.insert(
            "other_stack_id".to_string(),
            "other_stack.template.json".to_string(),
        );
        let deleted_workspace = WorkspaceConfig {
            version: 1,
            stacks: stacks.clone(),
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // workspace_configのreadのモック設定
        let deleted_workspace_clone = deleted_workspace.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| Ok(deleted_workspace_clone.clone()));

        // remove_fileのモック設定
        let stack_file_path = PathBuf::from(workspace_directory).join(stack_file_name_json);
        let stack_meta_file_path = PathBuf::from(workspace_directory).join(meta_file_name_json);
        mock_file_system
            .expect_remove_file()
            .times(2)
            .withf(move |path| path == &stack_file_path || path == &stack_meta_file_path)
            .returning(move |_path| Ok(()));

        // workspace_configのwriteのモック設定
        let updated_stacks = {
            let mut map = HashMap::new();
            map.insert(
                "other_stack_id".to_string(),
                "other_stack.template.json".to_string(),
            );
            map
        };
        let updated_stacks_clone = updated_stacks.clone();
        mock_workspace_config
            .expect_write()
            .withf(move |workspace, _file_system, workspace_dir| {
                return workspace.version == 1
                    && workspace.stacks == updated_stacks_clone
                    && workspace.name == deleted_workspace.name
                    && workspace.description == deleted_workspace.description
                    && workspace_dir == workspace_directory;
            })
            .returning(move |_workspace, _file_system, _workspace_dir| {
                Err(WorkspaceConfigError::App(AppError::new(
                    "Failed to write workspace config",
                )))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            delete_stack(&app_context, &config_context, workspace_directory, stack_id).await;

        // ######### 検証 #########
        // delete_stackがエラーになること
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod import_stack_tests {
    use super::*;
    use crate::{
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };
    use std::sync::Arc;

    // stackのインポートに成功すること
    // インポート元と先が同じファイルパス
    #[tokio::test]
    async fn success_import_same_path() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "sample_workspace";
        let file_name = "new_stack.template.json";
        let stack_file_path_buf = PathBuf::from(workspace_directory).join(file_name);
        let stack_file_path = stack_file_path_buf.to_str().unwrap().to_string();

        let workspace_config_before = WorkspaceConfig {
            version: 1,
            stacks: HashMap::new(),
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // workspace_configのreadのモック設定
        let workspace_config_before_clone = workspace_config_before.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(workspace_config_before_clone.clone())
            });

        // workspace_configのwriteのモック設定
        let updated_stacks = {
            let mut map = HashMap::new();
            map.insert(
                "imported_stack".to_string(),
                "new_stack.template.json".to_string(),
            );
            map
        };
        mock_workspace_config
            .expect_write()
            .withf(move |workspace, _file_system, workspace_dir| {
                // stack_idがUUID形式であることを確認
                let stack_id = workspace.stacks.keys().next().unwrap();
                let is_uuid = uuid::Uuid::parse_str(stack_id).is_ok();

                let stack_name = workspace.stacks.values().next().unwrap();
                return is_uuid
                    && stack_name == "new_stack.template.json"
                    && workspace_dir == workspace_directory;
            })
            .returning(move |_workspace, _file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: workspace_config_before.version,
                    stacks: updated_stacks.clone(),
                    name: workspace_config_before.name.clone(),
                    description: workspace_config_before.description.clone(),
                })
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = import_stack(
            &app_context,
            &config_context,
            workspace_directory,
            stack_file_path.as_str(),
        )
        .await;

        // ######### 検証 #########
        // stackが正しくインポートできていること
        assert!(result.is_ok());
    }

    // stackが存在する場合、追加されずに成功すること
    // インポート元と先が異なるファイルパス
    #[tokio::test]
    async fn success_import_different_paths_json() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "sample_workspace";
        let file_name = "existing_stack.template.json";
        let stack_file_path_buf = PathBuf::from("sample_workspace2").join(file_name);
        let stack_file_path = stack_file_path_buf.to_str().unwrap().to_string();

        let mut stacks_before = HashMap::new();
        // スタックは登録済み
        stacks_before.insert("existing_stack_id".to_string(), file_name.to_string());
        let workspace_config_before = WorkspaceConfig {
            version: 1,
            stacks: stacks_before,
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // file_systemのcopy_fileのモック設定
        let copy_file_path = PathBuf::from(workspace_directory).join(file_name);
        let stack_file_path_cloned = stack_file_path.clone();
        mock_file_system
            .expect_copy_file()
            .withf(move |src, dest| {
                src == PathBuf::from(stack_file_path_cloned.clone()) && dest == &copy_file_path
            })
            .returning(move |_src, _dest| Ok(1024));

        // workspace_configのreadのモック設定
        let workspace_config_before_clone = workspace_config_before.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(workspace_config_before_clone.clone())
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = import_stack(
            &app_context,
            &config_context,
            workspace_directory,
            stack_file_path.as_str(),
        )
        .await;

        // ######### 検証 #########
        // stackが正しくインポートできていること
        assert!(result.is_ok());
    }

    // yamlファイル形式のstackのインポートに成功すること
    // インポート元と先が異なるファイルパス
    #[tokio::test]
    async fn success_import_different_paths_yaml() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "sample_workspace";
        let file_name = "existing_stack.template.yaml";
        let stack_file_path_buf = PathBuf::from("sample_workspace2").join(file_name);
        let stack_file_path = stack_file_path_buf.to_str().unwrap().to_string();

        let mut stacks_before = HashMap::new();
        // スタックは登録済み
        stacks_before.insert("existing_stack_id".to_string(), file_name.to_string());
        let workspace_config_before = WorkspaceConfig {
            version: 1,
            stacks: stacks_before,
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // file_systemのread_fileのモック設定
        let stack_content = r#"{
            "name": "New Stack",
            "description": "A new stack",
            "version": "1"
        }"#;
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == stack_file_path_buf)
            .returning(move |_path| Ok(stack_content.to_string()));

        // file_systemのwrite_fileのモック設定
        mock_file_system
            .expect_write_file()
            .withf(move |path, content| {
                let is_correct_path = path
                    == &PathBuf::from(workspace_directory).join("existing_stack.template.json");
                // バイト列をJSONとしてパースして比較
                let is_correct_content = match serde_json::from_slice::<serde_json::Value>(content)
                {
                    Ok(got_json) => serde_json::from_str::<serde_json::Value>(stack_content)
                        .map(|expected| expected == got_json)
                        .unwrap_or(false),
                    Err(_) => false,
                };
                is_correct_path && is_correct_content
            })
            .returning(move |_path, _content| Ok(()));

        // workspace_configのreadのモック設定
        let workspace_config_before_clone = workspace_config_before.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(workspace_config_before_clone.clone())
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = import_stack(
            &app_context,
            &config_context,
            workspace_directory,
            stack_file_path.as_str(),
        )
        .await;

        // ######### 検証 #########
        // stackが正しくインポートできていること
        assert!(result.is_ok());
    }

    // ymlファイル形式のstackのread_fileが失敗した場合、エラーになること
    // インポート元と先が異なるファイルパス
    #[tokio::test]
    async fn import_different_paths_yml_read_file_error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config: MockAppConfigTrait = MockAppConfigTrait::new();
        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "sample_workspace";
        let file_name = "existing_stack.template.yml";
        let stack_file_path_buf = PathBuf::from("sample_workspace2").join(file_name);
        let stack_file_path = stack_file_path_buf.to_str().unwrap().to_string();

        // file_systemのread_fileのモック設定
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == stack_file_path_buf)
            .returning(move |_path| {
                Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::NotFound,
                    "read_file error",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = import_stack(
            &app_context,
            &config_context,
            workspace_directory,
            stack_file_path.as_str(),
        )
        .await;

        // ######### 検証 #########
        // stackのインポートがエラーになること
        assert!(result.is_err());
    }

    // ymlファイル形式のstackのwrite_fileが失敗した場合、エラーになること
    // インポート元と先が異なるファイルパス
    #[tokio::test]
    async fn import_different_paths_yml_write_file_error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "sample_workspace";
        let file_name = "existing_stack.template.yaml";
        let stack_file_path_buf = PathBuf::from("sample_workspace2").join(file_name);
        let stack_file_path = stack_file_path_buf.to_str().unwrap().to_string();

        // file_systemのread_fileのモック設定
        let stack_content = r#"{
            "name": "New Stack",
            "description": "A new stack",
            "version": "1"
        }"#;
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == stack_file_path_buf)
            .returning(move |_path| Ok(stack_content.to_string()));

        // file_systemのwrite_fileのモック設定
        mock_file_system
            .expect_write_file()
            .returning(move |_path, _content| {
                Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::Other,
                    "write_file error",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = import_stack(
            &app_context,
            &config_context,
            workspace_directory,
            stack_file_path.as_str(),
        )
        .await;

        // ######### 検証 #########
        // stackのインポートがエラーになること
        assert!(result.is_err());
    }

    // jsonファイル形式のstackのcopy_fileが失敗した場合、エラーになること
    // インポート元と先が異なるファイルパス
    #[tokio::test]
    async fn import_different_paths_json_copy_file_error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "sample_workspace";
        let file_name = "existing_stack.template.json";
        let stack_file_path_buf = PathBuf::from("sample_workspace2").join(file_name);
        let stack_file_path = stack_file_path_buf.to_str().unwrap().to_string();

        // file_systemのcopy_fileのモック設定
        let copy_file_path = PathBuf::from(workspace_directory).join(file_name);
        let stack_file_path_cloned = stack_file_path.clone();
        mock_file_system
            .expect_copy_file()
            .withf(move |src, dest| {
                src == PathBuf::from(stack_file_path_cloned.clone()) && dest == &copy_file_path
            })
            .returning(move |_src, _dest| {
                Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::Other,
                    "copy_file error",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = import_stack(
            &app_context,
            &config_context,
            workspace_directory,
            stack_file_path.as_str(),
        )
        .await;

        // ######### 検証 #########
        // stackのインポートがエラーになること
        assert!(result.is_err());
    }

    // workspace_configのreadに失敗した場合、エラーになること
    // インポート元と先が同じファイルパス
    #[tokio::test]
    async fn import_same_path_workspace_config_read_error() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "sample_workspace";
        let file_name = "new_stack.template.json";
        let stack_file_path_buf = PathBuf::from(workspace_directory).join(file_name);
        let stack_file_path = stack_file_path_buf.to_str().unwrap().to_string();

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Err(WorkspaceConfigError::App(AppError::new(
                    "Failed to read workspace config",
                )))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = import_stack(
            &app_context,
            &config_context,
            workspace_directory,
            stack_file_path.as_str(),
        )
        .await;

        // ######### 検証 #########
        // stackのインポートがエラーになること
        assert!(result.is_err());
    }

    // workspace_configのwriteに失敗した場合、エラーになること
    // インポート元と先が同じファイルパス
    #[tokio::test]
    async fn import_same_path_workspace_config_write_error() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "sample_workspace";
        let file_name = "new_stack.template.json";
        let stack_file_path_buf = PathBuf::from(workspace_directory).join(file_name);
        let stack_file_path = stack_file_path_buf.to_str().unwrap().to_string();

        let workspace_config_before = WorkspaceConfig {
            version: 1,
            stacks: HashMap::new(),
            name: "Sample Workspace".to_string(),
            description: "workspace_description sample".to_string(),
        };

        // workspace_configのreadのモック設定
        let workspace_config_before_clone = workspace_config_before.clone();
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(workspace_config_before_clone.clone())
            });

        // workspace_configのwriteのモック設定
        mock_workspace_config.expect_write().returning(
            move |_workspace, _file_system, _workspace_dir| {
                Err(WorkspaceConfigError::App(AppError::new(
                    "Failed to write workspace config",
                )))
            },
        );

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = import_stack(
            &app_context,
            &config_context,
            workspace_directory,
            stack_file_path.as_str(),
        )
        .await;

        // ######### 検証 #########
        // stackのインポートがエラーになること
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod load_stack_from_info_tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            stack_meta_config::StackMetaConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };

    /// stackの取得に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack.template.json";

        let stack_name: &str = "Sample Stack Name";
        let description_from_meta = "description_from_meta sample";
        let description_from_stack = "description_from_stack sample";

        // file_systemのread_fileのモック設定
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Description": description_from_stack,
                    "Resources": {}
                });
                Ok(template_json.to_string())
            });

        // stack_meta_configのreadのモック設定
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "sample_stack"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Ok(StackMetaConfig {
                    version: 1,
                    name: stack_name.to_string(),
                    description: description_from_meta.to_string(),
                    reasons: HashMap::new(),
                })
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = load_stack_from_info(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
            stack_file_name,
        )
        .await;

        // ######### 検証 #########
        // stackが正しく取得できていること
        assert!(result.is_ok());
        let stack = result.unwrap();
        assert_eq!(stack.id, stack_id.to_string());
        assert_eq!(stack.name, stack_name.to_string());
        assert_eq!(
            stack.description_from_meta,
            Some(description_from_meta.to_string())
        );
        assert_eq!(
            stack.description_from_stack,
            Some(description_from_stack.to_string())
        );
        assert_eq!(stack.exist, true);
    }

    // descriptionがstackにない場合、Noneになること
    #[tokio::test]
    async fn success_stack_description_none() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack.template.json";

        let stack_name: &str = "Sample Stack Name";

        // file_systemのread_fileのモック設定
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Resources": {}
                });
                Ok(template_json.to_string())
            });

        // stack_meta_configのreadのモック設定
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "sample_stack"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Ok(StackMetaConfig {
                    version: 1,
                    name: stack_name.to_string(),
                    description: "".to_string(),
                    reasons: HashMap::new(),
                })
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = load_stack_from_info(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
            stack_file_name,
        )
        .await;

        // ######### 検証 #########
        // stackが正しく取得できていること
        assert!(result.is_ok());
        let stack = result.unwrap();
        assert_eq!(stack.id, stack_id.to_string());
        assert_eq!(stack.name, stack_name.to_string());
        assert_eq!(stack.description_from_meta, Some("".to_string()));
        assert_eq!(stack.description_from_stack, None);
        assert_eq!(stack.exist, true);
    }

    // read_fileが失敗した場合、エラーになること
    #[tokio::test]
    async fn read_file_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack.template.json";

        // file_systemのread_fileのモック設定
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::Other,
                    "read_file error",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = load_stack_from_info(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
            stack_file_name,
        )
        .await;

        // ######### 検証 #########
        // load_stack_from_infoがエラーになること
        assert!(result.is_err());
    }

    // stack_meta_configのreadが失敗した場合、エラーになること
    #[tokio::test]
    async fn stack_meta_config_read_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "sample_stack_id";
        let stack_file_name = "sample_stack.template.json";

        let description_from_stack = "description_from_stack sample";

        // file_systemのread_fileのモック設定
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Description": description_from_stack,
                    "Resources": {}
                });
                Ok(template_json.to_string())
            });

        // stack_meta_configのreadのモック設定
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "sample_stack"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Err(StackMetaConfigError::App(AppError::new("Read error")))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result = load_stack_from_info(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
            stack_file_name,
        )
        .await;

        // ######### 検証 #########
        // load_stack_from_infoがエラーになること
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod load_stack_from_id_tests {
    use std::sync::Arc;

    use super::*;
    use crate::{
        command::context::{
            app_config_command_trait::MockAppConfigCommandTrait,
            cloudformation_schema_trait::MockCloudFormationSchemaTrait,
            manual_management_resource_trait::MockManualManagementResourceTrait,
            stack_command_trait::MockStackCommandTrait,
        },
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };

    /// stackの取得に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_command_app_config = MockAppConfigCommandTrait::new();
        let mut mock_command_stack = MockStackCommandTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "sample_stack_id";

        let stack_file_name = "sample_stack.template.json";
        let stack_name: &str = "Sample Stack Name";
        let description_from_meta = "description_from_meta sample";
        let description_from_stack = "description_from_stack sample";

        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name.to_string());

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // load_stack_from_infoのモック設定
        mock_command_stack
            .expect_load_stack_from_info()
            .withf(
                move |_app_context,
                      _config_context,
                      workspace_dir,
                      stack_id_arg,
                      stack_file_name_arg| {
                    workspace_dir == workspace_directory
                        && stack_id_arg == stack_id
                        && stack_file_name_arg == stack_file_name
                },
            )
            .returning(
                move |_app_context,
                      _config_context,
                      _workspace_dir,
                      _stack_id_arg,
                      _stack_file_name_arg| {
                    Ok(Stack {
                        id: stack_id.to_string(),
                        name: stack_name.to_string(),
                        description_from_meta: Some(description_from_meta.to_string()),
                        description_from_stack: Some(description_from_stack.to_string()),
                        exist: true,
                    })
                },
            );

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let command_context = CommandContext {
            app_config: Arc::new(mock_command_app_config),
            stack: Arc::new(mock_command_stack),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = load_stack_from_id(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        // stackが正しく取得できていること
        assert!(result.is_ok());
        let stack = result.unwrap();
        assert_eq!(stack.id, stack_id.to_string());
        assert_eq!(stack.name, stack_name.to_string());
        assert_eq!(
            stack.description_from_meta,
            Some(description_from_meta.to_string())
        );
        assert_eq!(
            stack.description_from_stack,
            Some(description_from_stack.to_string())
        );
        assert_eq!(stack.exist, true);
    }

    // workspace_configのreadが失敗した場合、エラーになること
    #[tokio::test]
    async fn workspace_config_read_err() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_command_app_config = MockAppConfigCommandTrait::new();
        let mock_command_stack = MockStackCommandTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "sample_stack_id";

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Err(WorkspaceConfigError::App(AppError::new(
                    "Failed to read workspace config",
                )))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let command_context = CommandContext {
            app_config: Arc::new(mock_command_app_config),
            stack: Arc::new(mock_command_stack),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = load_stack_from_id(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        // stackの取得がエラーになること
        assert!(result.is_err());
    }

    // stack_idがworkspace_configに存在しない場合、エラーになること
    #[tokio::test]
    async fn stack_id_not_found_err() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();
        let mock_command_app_config = MockAppConfigCommandTrait::new();
        let mock_command_stack = MockStackCommandTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "sample_stack_id";

        let stack_file_name = "sample_stack.template.json";

        let mut stacks = HashMap::new();
        stacks.insert(
            "different_stack_id".to_string(),
            stack_file_name.to_string(),
        );

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        let command_context = CommandContext {
            app_config: Arc::new(mock_command_app_config),
            stack: Arc::new(mock_command_stack),
            cloudformation_schema: Arc::new(MockCloudFormationSchemaTrait::new()),
            manual_management_resource: Arc::new(MockManualManagementResourceTrait::new()),
        };

        // ######### 実行 #########
        let result = load_stack_from_id(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        // stackの取得がエラーになること
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod load_template_summary_tests {
    use super::*;
    use crate::{
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            stack_meta_config::StackMetaConfig,
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };
    use std::sync::Arc;

    /// テンプレートサマリーの取得に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id_1 = "stack_id_1";
        let stack_file_name_1 = "stack1.template.json";
        let stack_id_2 = "stack_id_2";
        let stack_file_name_2 = "stack2.template.json";

        let mut stacks = HashMap::new();
        stacks.insert(stack_id_1.to_string(), stack_file_name_1.to_string());
        stacks.insert(stack_id_2.to_string(), stack_file_name_2.to_string());

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // stack_meta_configのreadのモック設定 (stack1用)
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "stack1"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Ok(StackMetaConfig {
                    version: 1,
                    name: "Stack 1 Name".to_string(),
                    description: "Stack 1 Description".to_string(),
                    reasons: HashMap::new(),
                })
            });

        // stack_meta_configのreadのモック設定 (stack2用)
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "stack2"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Ok(StackMetaConfig {
                    version: 1,
                    name: "Stack 2 Name".to_string(),
                    description: "Stack 2 Description".to_string(),
                    reasons: HashMap::new(),
                })
            });

        // file_systemのread_fileのモック設定 (stack1用)
        let template_path_1 = PathBuf::from(workspace_directory).join(stack_file_name_1);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path_1)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Resources": {
                        "MyS3Bucket": {
                            "Type": "AWS::S3::Bucket"
                        },
                        "MyDynamoDBTable": {
                            "Type": "AWS::DynamoDB::Table"
                        }
                    }
                });
                Ok(template_json.to_string())
            });

        // file_systemのread_fileのモック設定 (stack2用)
        let template_path_2 = PathBuf::from(workspace_directory).join(stack_file_name_2);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path_2)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Resources": {
                        "MyLambdaFunction": {
                            "Type": "AWS::Lambda::Function"
                        },
                        "MyS3BucketAnother": {
                            "Type": "AWS::S3::Bucket"
                        }
                    }
                });
                Ok(template_json.to_string())
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            load_template_summary(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 2);

        // stack1のリソース検証
        let stack1 = templates
            .iter()
            .find(|t| t.id == stack_id_1)
            .expect("Stack 1 should exist");
        assert_eq!(stack1.section_group_name, "Stack 1 Name");
        assert_eq!(stack1.resources.len(), 2);

        // stack2のリソース検証
        let stack2 = templates
            .iter()
            .find(|t| t.id == stack_id_2)
            .expect("Stack 2 should exist");
        assert_eq!(stack2.section_group_name, "Stack 2 Name");
        assert_eq!(stack2.resources.len(), 2);
    }

    /// リソースが空のスタックでも正しく処理されること
    #[tokio::test]
    async fn success_empty_resources() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "stack_id";
        let stack_file_name = "empty_stack.template.json";

        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name.to_string());

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // stack_meta_configのreadのモック設定
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "empty_stack"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Ok(StackMetaConfig {
                    version: 1,
                    name: "Empty Stack".to_string(),
                    description: "Stack with no resources".to_string(),
                    reasons: HashMap::new(),
                })
            });

        // file_systemのread_fileのモック設定
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Resources": {}
                });
                Ok(template_json.to_string())
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            load_template_summary(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].id, stack_id);
        assert_eq!(templates[0].section_group_name, "Empty Stack");
        assert_eq!(templates[0].resources.len(), 0);
    }

    /// workspace_configのreadに失敗した場合、エラーになること
    #[tokio::test]
    async fn workspace_config_read_error() {
        // ######### 準備 #########
        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Err(WorkspaceConfigError::App(AppError::new(
                    "Failed to read workspace config",
                )))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            load_template_summary(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// stack_meta_configのreadに失敗した場合、エラーになること
    #[tokio::test]
    async fn stack_meta_config_read_error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "stack_id";
        let stack_file_name = "stack.template.json";

        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name.to_string());

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // file_systemのread_fileのモック設定
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Resources": {}
                });
                Ok(template_json.to_string())
            });

        // stack_meta_configのreadのモック設定（エラーを返す）
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "stack"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Err(StackMetaConfigError::App(AppError::new(
                    "Failed to read stack meta",
                )))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            load_template_summary(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// read_fileに失敗した場合、エラーになること
    #[tokio::test]
    async fn read_file_error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "stack_id";
        let stack_file_name = "stack.template.json";

        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name.to_string());

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // stack_meta_configのreadのモック設定
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "stack"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Ok(StackMetaConfig {
                    version: 1,
                    name: "Stack Name".to_string(),
                    description: "Stack Description".to_string(),
                    reasons: HashMap::new(),
                })
            });

        // file_systemのread_fileのモック設定（エラーを返す）
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::NotFound,
                    "File not found",
                ))
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            load_template_summary(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// 不正な形式のリソースタイプが含まれていてもスキップして正しく処理されること
    #[tokio::test]
    async fn success_with_invalid_resource_types() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "stack_id";
        let stack_file_name = "mixed_stack.template.json";

        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name.to_string());

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // stack_meta_configのreadのモック設定
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "mixed_stack"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Ok(StackMetaConfig {
                    version: 1,
                    name: "Mixed Stack".to_string(),
                    description: "Stack with valid and invalid resource types".to_string(),
                    reasons: HashMap::new(),
                })
            });

        // file_systemのread_fileのモック設定
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Resources": {
                        "ValidS3Bucket": {
                            "Type": "AWS::S3::Bucket"
                        },
                        "InvalidType1": {
                            "Type": "InvalidType"
                        },
                        "InvalidType2": {
                            "Type": "AWS::S3"
                        },
                        "ValidLambda": {
                            "Type": "AWS::Lambda::Function"
                        },
                        "InvalidType3": {
                            "Type": "AWS::S3::Bucket::Extra::Part"
                        }
                    }
                });
                Ok(template_json.to_string())
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            load_template_summary(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].id, stack_id);
        assert_eq!(templates[0].section_group_name, "Mixed Stack");
        // 不正な形式のリソースタイプはスキップされ、有効な2つのみがカウントされること
        assert_eq!(templates[0].resources.len(), 2);

        // S3とLambdaのリソースが含まれていることを確認
        let has_s3 = templates[0]
            .resources
            .iter()
            .any(|r| r.service_name == "S3");
        let has_lambda = templates[0]
            .resources
            .iter()
            .any(|r| r.service_name == "Lambda");
        assert!(has_s3, "S3 resource should be included");
        assert!(has_lambda, "Lambda resource should be included");
    }

    /// 同じサービス名で複数のリソースタイプがある場合、正しく集約されること
    #[tokio::test]
    async fn success_with_multiple_resource_types_same_service() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let mock_app_config = MockAppConfigTrait::new();
        let mut mock_workspace_config = MockWorkspaceConfigTrait::new();
        let mut mock_stack_meta_config = MockStackMetaConfigTrait::new();

        let workspace_directory = "test/sample_workspace";
        let stack_id = "stack_id";
        let stack_file_name = "multi_type_stack.template.json";

        let mut stacks = HashMap::new();
        stacks.insert(stack_id.to_string(), stack_file_name.to_string());

        // workspace_configのreadのモック設定
        mock_workspace_config
            .expect_read()
            .withf(move |_file_system, workspace_dir| workspace_dir == workspace_directory)
            .returning(move |_file_system, _workspace_dir| {
                Ok(WorkspaceConfig {
                    version: 1,
                    stacks: stacks.clone(),
                    name: "Sample Workspace".to_string(),
                    description: "workspace_description sample".to_string(),
                })
            });

        // stack_meta_configのreadのモック設定
        mock_stack_meta_config
            .expect_read()
            .withf(move |_file_system, workspace_dir, stack_name| {
                workspace_dir == workspace_directory && stack_name == "multi_type_stack"
            })
            .returning(|_file_system, _workspace_dir, _stack_name| {
                Ok(StackMetaConfig {
                    version: 1,
                    name: "Multi Type Stack".to_string(),
                    description: "Stack with multiple resource types in same service".to_string(),
                    reasons: HashMap::new(),
                })
            });

        // file_systemのread_fileのモック設定
        let template_path = PathBuf::from(workspace_directory).join(stack_file_name);
        mock_file_system
            .expect_read_file()
            .withf(move |path| path == &template_path)
            .returning(move |_path| {
                let template_json = serde_json::json!({
                    "Resources": {
                        "MyS3Bucket": {
                            "Type": "AWS::S3::Bucket"
                        },
                        "MyS3BucketPolicy": {
                            "Type": "AWS::S3::BucketPolicy"
                        },
                        "AnotherS3Bucket": {
                            "Type": "AWS::S3::Bucket"
                        },
                        "MyLambdaFunction": {
                            "Type": "AWS::Lambda::Function"
                        },
                        "MyLambdaPermission": {
                            "Type": "AWS::Lambda::Permission"
                        }
                    }
                });
                Ok(template_json.to_string())
            });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config),
            workspace_config_io: Arc::new(mock_workspace_config),
            stack_meta_config_io: Arc::new(mock_stack_meta_config),
        };

        // ######### 実行 #########
        let result =
            load_template_summary(&app_context, &config_context, workspace_directory).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].id, stack_id);
        assert_eq!(templates[0].section_group_name, "Multi Type Stack");
        // S3とLambdaの2つのサービスがあること
        assert_eq!(templates[0].resources.len(), 2);

        // S3サービスのリソースタイプを確認
        let s3_resource = templates[0]
            .resources
            .iter()
            .find(|r| r.service_name == "S3")
            .expect("S3 service should exist");
        // BucketとBucketPolicyの2つのタイプがあること（同じBucketが2つあっても1つにまとめられる）
        assert_eq!(s3_resource.recourse_type.len(), 2);
        assert!(s3_resource.recourse_type.contains(&"Bucket".to_string()));
        assert!(s3_resource
            .recourse_type
            .contains(&"BucketPolicy".to_string()));

        // Lambdaサービスのリソースタイプを確認
        let lambda_resource = templates[0]
            .resources
            .iter()
            .find(|r| r.service_name == "Lambda")
            .expect("Lambda service should exist");
        // FunctionとPermissionの2つのタイプがあること
        assert_eq!(lambda_resource.recourse_type.len(), 2);
        assert!(lambda_resource
            .recourse_type
            .contains(&"Function".to_string()));
        assert!(lambda_resource
            .recourse_type
            .contains(&"Permission".to_string()));
    }
}

#[cfg(test)]
#[coverage(off)]
mod get_stack_resource_list_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod get_stack_resource_properties_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod get_stack_resource_properties_reasons_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod get_stack_parameters_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod get_stack_outputs_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod get_all_stack_outputs_tests {
    use super::*;
    use crate::{
        command::context::{
            app_config_command_trait::MockAppConfigCommandTrait,
            cloudformation_schema_trait::MockCloudFormationSchemaTrait,
            manual_management_resource_trait::MockManualManagementResourceTrait,
            stack_command_trait::MockStackCommandTrait,
        },
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };
    use std::sync::Arc;

    /// 全スタックのOutputsを正しく取得できること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();

        let mut stacks = HashMap::new();
        stacks.insert("stack1".to_string(), "stack1.template.json".to_string());
        stacks.insert("stack2".to_string(), "stack2.template.json".to_string());

        mock_workspace_config_io
            .expect_read()
            .returning(move |_, _| {
                Ok(WorkspaceConfig {
                    version: 1,
                    name: "Test Workspace".to_string(),
                    description: "Test Description".to_string(),
                    stacks: stacks.clone(),
                })
            });

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mock_app_config_command = MockAppConfigCommandTrait::new();
        let mut mock_stack_command = MockStackCommandTrait::new();
        let mock_cloudformation_schema = MockCloudFormationSchemaTrait::new();
        let mock_manual_management_resource = MockManualManagementResourceTrait::new();

        mock_stack_command
            .expect_get_stack_outputs()
            .withf(|_, _, _, stack_id| stack_id == "stack1")
            .returning(|_, _, _, _| {
                Ok(vec![
                    StackOutput {
                        name: "Output1".to_string(),
                        description: Some("Description 1".to_string()),
                        export_name: Some("ExportName1".to_string()),
                        value: serde_json::json!("Value1"),
                    },
                    StackOutput {
                        name: "Output2".to_string(),
                        description: None,
                        export_name: None,
                        value: serde_json::json!("Value2"),
                    },
                ])
            });

        mock_stack_command
            .expect_get_stack_outputs()
            .withf(|_, _, _, stack_id| stack_id == "stack2")
            .returning(|_, _, _, _| {
                Ok(vec![StackOutput {
                    name: "Output3".to_string(),
                    description: Some("Description 3".to_string()),
                    export_name: Some("ExportName3".to_string()),
                    value: serde_json::json!("Value3"),
                }])
            });

        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            manual_management_resource: Arc::new(mock_manual_management_resource),
        };

        // ######### 実行 #########
        let result = get_all_stack_outputs(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs.get("ExportName1"), Some(&"stack1".to_string()));
        assert_eq!(outputs.get("ExportName3"), Some(&"stack2".to_string()));
    }

    /// Outputsがないスタックが含まれていても正しく動作すること
    #[tokio::test]
    async fn success_with_no_outputs() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();

        let mut stacks = HashMap::new();
        stacks.insert("stack1".to_string(), "stack1.template.json".to_string());
        stacks.insert("stack2".to_string(), "stack2.template.json".to_string());

        mock_workspace_config_io
            .expect_read()
            .returning(move |_, _| {
                Ok(WorkspaceConfig {
                    version: 1,
                    name: "Test Workspace".to_string(),
                    description: "Test Description".to_string(),
                    stacks: stacks.clone(),
                })
            });

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mock_app_config_command = MockAppConfigCommandTrait::new();
        let mut mock_stack_command = MockStackCommandTrait::new();
        let mock_cloudformation_schema = MockCloudFormationSchemaTrait::new();
        let mock_manual_management_resource = MockManualManagementResourceTrait::new();

        mock_stack_command
            .expect_get_stack_outputs()
            .withf(|_, _, _, stack_id| stack_id == "stack1")
            .returning(|_, _, _, _| Ok(vec![]));

        mock_stack_command
            .expect_get_stack_outputs()
            .withf(|_, _, _, stack_id| stack_id == "stack2")
            .returning(|_, _, _, _| {
                Ok(vec![StackOutput {
                    name: "Output1".to_string(),
                    description: None,
                    export_name: Some("ExportName1".to_string()),
                    value: serde_json::json!("Value1"),
                }])
            });

        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            manual_management_resource: Arc::new(mock_manual_management_resource),
        };

        // ######### 実行 #########
        let result = get_all_stack_outputs(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs.get("ExportName1"), Some(&"stack2".to_string()));
    }

    /// export_nameがないOutputsは結果に含まれないこと
    #[tokio::test]
    async fn success_exclude_outputs_without_export_name() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();

        let mut stacks = HashMap::new();
        stacks.insert("stack1".to_string(), "stack1.template.json".to_string());

        mock_workspace_config_io
            .expect_read()
            .returning(move |_, _| {
                Ok(WorkspaceConfig {
                    version: 1,
                    name: "Test Workspace".to_string(),
                    description: "Test Description".to_string(),
                    stacks: stacks.clone(),
                })
            });

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mock_app_config_command = MockAppConfigCommandTrait::new();
        let mut mock_stack_command = MockStackCommandTrait::new();
        let mock_cloudformation_schema = MockCloudFormationSchemaTrait::new();
        let mock_manual_management_resource = MockManualManagementResourceTrait::new();

        mock_stack_command
            .expect_get_stack_outputs()
            .withf(|_, _, _, stack_id| stack_id == "stack1")
            .returning(|_, _, _, _| {
                Ok(vec![
                    StackOutput {
                        name: "Output1".to_string(),
                        description: Some("Description 1".to_string()),
                        export_name: Some("ExportName1".to_string()),
                        value: serde_json::json!("Value1"),
                    },
                    StackOutput {
                        name: "Output2".to_string(),
                        description: None,
                        export_name: None,
                        value: serde_json::json!("Value2"),
                    },
                    StackOutput {
                        name: "Output3".to_string(),
                        description: None,
                        export_name: None,
                        value: serde_json::json!("Value3"),
                    },
                ])
            });

        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            manual_management_resource: Arc::new(mock_manual_management_resource),
        };

        // ######### 実行 #########
        let result = get_all_stack_outputs(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs.get("ExportName1"), Some(&"stack1".to_string()));
    }

    /// スタックが存在しない場合、空のHashMapを返すこと
    #[tokio::test]
    async fn success_no_stacks() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();

        mock_workspace_config_io
            .expect_read()
            .returning(move |_, _| {
                Ok(WorkspaceConfig {
                    version: 1,
                    name: "Test Workspace".to_string(),
                    description: "Test Description".to_string(),
                    stacks: HashMap::new(),
                })
            });

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mock_app_config_command = MockAppConfigCommandTrait::new();
        let mock_stack_command = MockStackCommandTrait::new();
        let mock_cloudformation_schema = MockCloudFormationSchemaTrait::new();
        let mock_manual_management_resource = MockManualManagementResourceTrait::new();

        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            manual_management_resource: Arc::new(mock_manual_management_resource),
        };

        // ######### 実行 #########
        let result = get_all_stack_outputs(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let outputs = result.unwrap();
        assert_eq!(outputs.len(), 0);
    }

    /// workspace_config_ioのreadに失敗した場合、エラーを返すこと
    #[tokio::test]
    async fn workspace_config_read_error() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();

        mock_workspace_config_io
            .expect_read()
            .returning(move |_, _| {
                Err(WorkspaceConfigError::App(AppError::new(
                    "Failed to read workspace config",
                )))
            });

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mock_app_config_command = MockAppConfigCommandTrait::new();
        let mock_stack_command = MockStackCommandTrait::new();
        let mock_cloudformation_schema = MockCloudFormationSchemaTrait::new();
        let mock_manual_management_resource = MockManualManagementResourceTrait::new();

        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            manual_management_resource: Arc::new(mock_manual_management_resource),
        };

        // ######### 実行 #########
        let result = get_all_stack_outputs(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// get_stack_outputsに失敗した場合、エラーを返すこと
    #[tokio::test]
    async fn get_stack_outputs_error() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";

        let mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();

        let mut stacks = HashMap::new();
        stacks.insert("stack1".to_string(), "stack1.template.json".to_string());

        mock_workspace_config_io
            .expect_read()
            .returning(move |_, _| {
                Ok(WorkspaceConfig {
                    version: 1,
                    name: "Test Workspace".to_string(),
                    description: "Test Description".to_string(),
                    stacks: stacks.clone(),
                })
            });

        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
        };

        let mock_app_config_command = MockAppConfigCommandTrait::new();
        let mut mock_stack_command = MockStackCommandTrait::new();
        let mock_cloudformation_schema = MockCloudFormationSchemaTrait::new();
        let mock_manual_management_resource = MockManualManagementResourceTrait::new();

        mock_stack_command
            .expect_get_stack_outputs()
            .returning(|_, _, _, _| {
                Err(StackError::App(AppError::new(
                    "Failed to get stack outputs",
                )))
            });

        let command_context = CommandContext {
            app_config: Arc::new(mock_app_config_command),
            stack: Arc::new(mock_stack_command),
            cloudformation_schema: Arc::new(mock_cloudformation_schema),
            manual_management_resource: Arc::new(mock_manual_management_resource),
        };

        // ######### 実行 #########
        let result = get_all_stack_outputs(
            &app_context,
            &config_context,
            &command_context,
            workspace_directory,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }
}

#[cfg(test)]
#[coverage(off)]
mod update_stack_reasons_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod update_stack_detail_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod load_parameter_and_resource_list_tests {
    use std::sync::Arc;

    use crate::{
        config::{
            context::{
                app_config_trait::MockAppConfigTrait,
                stack_meta_config_trait::MockStackMetaConfigTrait,
                workspace_config_trait::MockWorkspaceConfigTrait,
            },
            workspace_config::WorkspaceConfig,
        },
        utils::context::{file::MockFileSystem, http_client::MockHttpClient},
    };

    use super::*;

    /// スタックのパラメータとリソースリストの取得に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";
        let stack_id = "stack_id";

        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        mock_file_system.expect_read_file().returning(|_| {
            let template_json = serde_json::json!({
                "Parameters": {
                    "Param1": {
                        "Type": "String",
                        "Description": "Value1"
                    },
                    "Param2": {
                        "Type": "Number",
                        "Description": "Value2"
                    }
                },
                "Resources": {
                    "MyBucket": {
                        "Type": "AWS::S3::Bucket",
                        "Properties": {
                            "BucketName": "my-bucket"
                        }
                    },
                    "MyLambda": {
                        "Type": "AWS::Lambda::Function",
                        "Properties": {
                            "FunctionName": "my-function",
                        }
                    }
                }
            });
            Ok(template_json.to_string())
        });
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io.expect_read().returning(|_, _| {
            let stacks =
                HashMap::from([(stack_id.to_string(), "stack_file.template.yaml".to_string())]);
            Ok(WorkspaceConfig {
                version: 1,
                name: "test stack".to_string(),
                description: "test stack description".to_string(),
                stacks,
            })
        });
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
        };

        // ######### 実行 #########
        let result = load_parameter_and_resource_list(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let unwrap_result = result.unwrap();
        assert_eq!(unwrap_result.parameters.len(), 2);
        assert_eq!(unwrap_result.parameters[0], "Param1");
        assert_eq!(unwrap_result.parameters[1], "Param2");
        assert_eq!(unwrap_result.resources.len(), 2);
        let resource1 = unwrap_result.resources.get("MyBucket").unwrap();
        assert_eq!(resource1.service_name, "S3");
        assert_eq!(resource1.recourse_type, "Bucket");
        let resource2 = unwrap_result.resources.get("MyLambda").unwrap();
        assert_eq!(resource2.service_name, "Lambda");
        assert_eq!(resource2.recourse_type, "Function");
    }

    /// スタックにパラメータが存在しない場合に、空のパラメータリストを返すこと
    #[tokio::test]
    async fn success_no_parameters() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";
        let stack_id = "stack_id";

        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        mock_file_system.expect_read_file().returning(|_| {
            let template_json = serde_json::json!({
                "Resources": {
                    "MyBucket": {
                        "Type": "AWS::S3::Bucket",
                        "Properties": {
                            "BucketName": "my-bucket"
                        }
                    }
                }
            });
            Ok(template_json.to_string())
        });
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io.expect_read().returning(|_, _| {
            let stacks =
                HashMap::from([(stack_id.to_string(), "stack_file.template.yaml".to_string())]);
            Ok(WorkspaceConfig {
                version: 1,
                name: "test stack".to_string(),
                description: "test stack description".to_string(),
                stacks,
            })
        });
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
        };

        // ######### 実行 #########
        let result = load_parameter_and_resource_list(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let unwrap_result = result.unwrap();
        assert_eq!(unwrap_result.parameters.len(), 0);
        assert_eq!(unwrap_result.resources.len(), 1);
        let resource1 = unwrap_result.resources.get("MyBucket").unwrap();
        assert_eq!(resource1.service_name, "S3");
        assert_eq!(resource1.recourse_type, "Bucket");
    }

    /// リソースの "Type" が不正な場合に、そのリソースを無視して処理を継続すること
    #[tokio::test]
    async fn success_invalid_resource_type() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";
        let stack_id = "stack_id";

        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        mock_file_system.expect_read_file().returning(|_| {
            let template_json = serde_json::json!({
                "Resources": {
                    "InvalidResource": {
                        "Type": "Invalid::Type", // 不正なリソースタイプ
                        "Properties": {}
                    },
                    "MyBucket": {
                        "Type": "AWS::S3::Bucket",
                        "Properties": {
                            "BucketName": "my-bucket"
                        }
                    }
                }
            });
            Ok(template_json.to_string())
        });
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io.expect_read().returning(|_, _| {
            let stacks =
                HashMap::from([(stack_id.to_string(), "stack_file.template.yaml".to_string())]);
            Ok(WorkspaceConfig {
                version: 1,
                name: "test stack".to_string(),
                description: "test stack description".to_string(),
                stacks,
            })
        });
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
        };

        // ######### 実行 #########
        let result = load_parameter_and_resource_list(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let unwrap_result = result.unwrap();
        assert_eq!(unwrap_result.parameters.len(), 0);
        assert_eq!(unwrap_result.resources.len(), 1);
        let resource1 = unwrap_result.resources.get("MyBucket").unwrap();
        assert_eq!(resource1.service_name, "S3");
        assert_eq!(resource1.recourse_type, "Bucket");
    }

    /// ワークスペースの読み込みに失敗した場合、Err を返すこと
    #[tokio::test]
    async fn failed_workspace_config_read() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";
        let stack_id = "stack_id";

        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        mock_file_system.expect_read_file().times(0);
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io
            .expect_read()
            .returning(|_, _| Err(WorkspaceConfigError::App(AppError::new("Read error"))));
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
        };

        // ######### 実行 #########
        let result = load_parameter_and_resource_list(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// 指定したスタックIDがワークスペースに存在しない場合、Err を返すこと
    #[tokio::test]
    async fn failed_not_found_stack_id() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";
        let stack_id = "stack_id";

        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        mock_file_system.expect_read_file().times(0);
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io.expect_read().returning(|_, _| {
            Ok(WorkspaceConfig {
                version: 1,
                name: "test stack".to_string(),
                description: "test stack description".to_string(),
                stacks: HashMap::new(), // 空のスタックマップ
            })
        });
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
        };

        // ######### 実行 #########
        let result = load_parameter_and_resource_list(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// スタックのテンプレート読み込みに失敗した場合、Err を返すこと
    #[tokio::test]
    async fn failed_template_read() {
        // ######### 準備 #########
        let workspace_directory = "test/workspace";
        let stack_id = "stack_id";

        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();
        mock_file_system.expect_read_file().returning(|_| {
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "read_file error",
            ))
        });
        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let mock_app_config_io = MockAppConfigTrait::new();
        let mock_stack_meta_config_io = MockStackMetaConfigTrait::new();
        let mut mock_workspace_config_io = MockWorkspaceConfigTrait::new();
        mock_workspace_config_io.expect_read().returning(|_, _| {
            let stacks =
                HashMap::from([(stack_id.to_string(), "stack_file.template.yaml".to_string())]);
            Ok(WorkspaceConfig {
                version: 1,
                name: "test stack".to_string(),
                description: "test stack description".to_string(),
                stacks,
            })
        });
        let config_context = ConfigContext {
            app_config_io: Arc::new(mock_app_config_io),
            stack_meta_config_io: Arc::new(mock_stack_meta_config_io),
            workspace_config_io: Arc::new(mock_workspace_config_io),
        };

        // ######### 実行 #########
        let result = load_parameter_and_resource_list(
            &app_context,
            &config_context,
            workspace_directory,
            stack_id,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }
}
