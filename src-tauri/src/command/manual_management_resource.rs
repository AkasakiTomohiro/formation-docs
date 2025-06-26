use crate::command::cloudformation_schema::get_cloudformation_schema;
use crate::command::stack::Resource;
use crate::utils::get_window_state;
use crate::utils::AppError;
use crate::utils::CommandResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::{collections::HashMap, path::Path};
use thiserror::Error;
use tokio::fs;

/// 手動管理リソースのJSONファイル名
const MANUAL_MANAGEMENT_RESOURCES_FILE: &str = "manual_management_resources.json";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManualManagementResource {
    pub description: String,
    pub r#type: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManualManagementResources {
    pub resources: HashMap<String, ManualManagementResource>,
}

#[derive(Debug, Error)]
enum ManualManagementResourceError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("cloud formation schema error: {0}")]
    CloudFormationSchemaError(
        #[from] crate::command::cloudformation_schema::CloudFormationSchemaError,
    ),
}

/// ワークスペース内にある手動管理リソース用のJSONファイルを読み込む
///
/// 対象のJSONファイルが存在しない場合は新規にファイルを作成してから読み込む
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
async fn get_manual_management_resources(
    workspace_directory: &str,
) -> Result<ManualManagementResources, ManualManagementResourceError> {
    let manual_management_resources_path = format!(
        "{}/{}",
        workspace_directory, MANUAL_MANAGEMENT_RESOURCES_FILE
    );
    let manual_management_resources_path = Path::new(&manual_management_resources_path);

    if !manual_management_resources_path.exists() {
        // ファイルが存在しない場合は新規に作成
        let initial_data = ManualManagementResources {
            resources: HashMap::new(),
        };
        let initial_json = serde_json::to_string(&initial_data)?;
        fs::write(&manual_management_resources_path, initial_json).await?;
    }

    // JSONファイルを読み込む
    let template_json = fs::read_to_string(&manual_management_resources_path).await?;
    let template_json = serde_json::from_str(&template_json)?;

    return Ok(template_json);
}

/// 手動管理リソースのJSONファイルを保存する
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
/// - `manual_management_resources` - 保存する手動管理リソースのデータ
async fn save_manual_management_resources(
    workspace_directory: &str,
    manual_management_resources: &ManualManagementResources,
) -> Result<(), ManualManagementResourceError> {
    let manual_management_resources_path = format!(
        "{}/{}",
        workspace_directory, MANUAL_MANAGEMENT_RESOURCES_FILE
    );
    let manual_management_resources_path = Path::new(&manual_management_resources_path);
    let updated_json = serde_json::to_string(&manual_management_resources)?;
    fs::write(&manual_management_resources_path, updated_json).await?;
    Ok(())
}

/// 新しい手動管理リソースを作成する
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
/// - `resource_id` - リソースID
/// - `description` - リソースの説明
/// - `service_name` - サービス名(すべて小文字)
/// - `resource_name` - リソース名(すべて小文字)
async fn new_manual_management_resource(
    workspace_directory: &str,
    resource_id: &str,
    description: &str,
    service_name: &str,
    resource_name: &str,
) -> Result<(), ManualManagementResourceError> {
    // 指定されたサービス名とリソース名に基づいてCloudFormationのスキーマを取得
    let resource_schema = get_cloudformation_schema(service_name, resource_name)?;
    let resource_schema: Value = serde_json::from_str(&resource_schema)?;
    if resource_schema.is_object() == false {
        return Err(ManualManagementResourceError::App(AppError::new(
            "Invalid resource schema format. Expected an object.",
        )));
    }

    // 指定されたサービス名とリソース名に基づいてCloudFormationにおけるリソースのタイプ名を取得
    let type_name = resource_schema["typeName"].as_str();
    if type_name.is_none() {
        return Err(ManualManagementResourceError::App(AppError::new(
            "Resource schema does not contain 'typeName'.",
        )));
    }
    let type_name = type_name.unwrap().to_string();

    // 手動管理リソースのJSONファイルを取得し、リソースIDが既に存在しないことを確認してから新しいリソースを追加
    let mut manual_management_resources =
        get_manual_management_resources(workspace_directory).await?;
    if manual_management_resources
        .resources
        .contains_key(resource_id)
    {
        return Err(ManualManagementResourceError::App(AppError::new(
            "Resource ID already exists.",
        )));
    }
    manual_management_resources.resources.insert(
        resource_id.to_string(),
        ManualManagementResource {
            description: description.to_string(),
            r#type: type_name,
            properties: HashMap::new(),
        },
    );

    // 更新された手動管理リソースのJSONファイルを保存
    save_manual_management_resources(workspace_directory, &manual_management_resources).await?;

    return Ok(());
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualManagementResourceSummary {
    pub resource_id: String,
    pub r#type: String,
    pub description: String,
}

/// 手動管理リソースの一覧を取得する
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
async fn get_manual_management_resource_list(
    workspace_directory: &str,
) -> Result<Vec<ManualManagementResourceSummary>, ManualManagementResourceError> {
    let manual_management_resources = get_manual_management_resources(workspace_directory).await?;
    let mut result: Vec<ManualManagementResourceSummary> = Vec::new();

    for (resource_id, manual_management_resource) in manual_management_resources.resources.iter() {
        result.push(ManualManagementResourceSummary {
            resource_id: resource_id.clone(),
            r#type: manual_management_resource.r#type.clone(),
            description: manual_management_resource.description.clone(),
        })
    }

    return Ok(result);
}

/// サイドメニューに表示する手動管理リソース用のサービス名とリソース種別の一覧を取得
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
async fn load_manual_management_resource_summary(
    workspace_directory: &str,
) -> Result<Vec<Resource>, ManualManagementResourceError> {
    let manual_management_resources = get_manual_management_resources(workspace_directory).await?;
    let mut resources = HashMap::<String, HashSet<String>>::new();
    for (_, manual_management_resource) in manual_management_resources.resources.iter() {
        let parts: Vec<&str> = manual_management_resource.r#type.split("::").collect();
        let (_, service_name, resource_type): (&str, &str, &str) = match parts[..] {
            [a, b, c] => (a, b, c),
            _ => continue,
        };

        if let Some(original) = resources.get_mut(service_name) {
            original.insert(resource_type.to_string());
        } else {
            let mut resource_types = HashSet::new();
            resource_types.insert(resource_type.to_string());
            resources.insert(service_name.to_string(), resource_types);
        }
    }
    let result = resources
        .iter()
        .map(|(key, value)| {
            let service_name = key.to_string();
            let resource_types = value.clone();
            Resource {
                service_name,
                recourse_type: resource_types.iter().map(|v| v.to_string()).collect(),
            }
        })
        .collect();

    return Ok(result);
}

#[tauri::command(rename_all = "snake_case")]
pub async fn new_manual_management_resource_command(
    window: tauri::Window,
    resource_id: &str,
    description: &str,
    service_name: &str,
    resource_name: &str,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match new_manual_management_resource(
        window_state.workspace_directory.as_str(),
        resource_id,
        description,
        service_name,
        resource_name,
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_manual_management_resource_list_command(
    window: tauri::Window,
) -> Result<CommandResult<Vec<ManualManagementResourceSummary>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_manual_management_resource_list(window_state.workspace_directory.as_str())
        .await
    {
        Ok(list) => Ok(CommandResult::success(list)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn load_manual_management_resource_summary_command(
    window: tauri::Window,
) -> Result<CommandResult<Vec<Resource>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_manual_management_resource_summary(window_state.workspace_directory.as_str())
        .await
    {
        Ok(summary) => Ok(CommandResult::success(summary)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
