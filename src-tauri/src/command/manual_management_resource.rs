use crate::api::context::api_context::ApiContext;
use crate::command::cloudformation_schema::get_cloudformation_schema;
use crate::command::stack::Resource;
use crate::utils::context::app_context::AppContext;
use crate::utils::get_window_state;
use crate::utils::AppError;
use crate::utils::CommandResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use tauri::State;
use thiserror::Error;

/// 手動管理リソースのJSONファイル名
const MANUAL_MANAGEMENT_RESOURCES_FILE: &str = "manual_management_resources.json";

/// 手動管理リソースのmetaファイル名
const MANUAL_MANAGEMENT_RESOURCES_META_FILE: &str = "manual_management_resources.meta.json";

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualManagementMeta {
    pub reasons: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualManagementMetaReasonsUpdate {
    pub resource_id: String,
    pub reasons: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManualManagementMetaUpdate {
    pub reasons: Option<ManualManagementMetaReasonsUpdate>,
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
    state: &AppContext,
    workspace_directory: &str,
) -> Result<ManualManagementResources, ManualManagementResourceError> {
    let manual_management_resources_path =
        PathBuf::from(workspace_directory).join(MANUAL_MANAGEMENT_RESOURCES_FILE);

    if !state
        .file_system
        .path_exists(&manual_management_resources_path)
    {
        // ファイルが存在しない場合は新規に作成
        let initial_data = ManualManagementResources {
            resources: HashMap::new(),
        };
        let initial_json = serde_json::to_string(&initial_data)?;
        state
            .file_system
            .write_file(&manual_management_resources_path, initial_json.as_bytes())
            .await?;
    }

    // JSONファイルを読み込む
    let template_json = state
        .file_system
        .read_file(&manual_management_resources_path)
        .await?;
    let template_json = serde_json::from_str(&template_json)?;

    return Ok(template_json);
}

/// 手動管理リソースのJSONファイルを保存する
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
/// - `manual_management_resources` - 保存する手動管理リソースのデータ
async fn save_manual_management_resources(
    state: &AppContext,
    workspace_directory: &str,
    manual_management_resources: &ManualManagementResources,
) -> Result<(), ManualManagementResourceError> {
    let manual_management_resources_path =
        PathBuf::from(workspace_directory).join(MANUAL_MANAGEMENT_RESOURCES_FILE);
    let updated_json = serde_json::to_string(&manual_management_resources)?;
    state
        .file_system
        .write_file(&manual_management_resources_path, updated_json.as_bytes())
        .await?;
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
    app_context: &AppContext,
    api_context: &ApiContext,
    workspace_directory: &str,
    resource_id: &str,
    description: &str,
    service_name: &str,
    resource_name: &str,
) -> Result<(), ManualManagementResourceError> {
    // 指定されたサービス名とリソース名に基づいてCloudFormationのスキーマを取得
    let resource_schema =
        get_cloudformation_schema(app_context, api_context, service_name, resource_name).await?;
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
        get_manual_management_resources(app_context, workspace_directory).await?;
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
    save_manual_management_resources(
        app_context,
        workspace_directory,
        &manual_management_resources,
    )
    .await?;

    return Ok(());
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualManagementResourceSummary {
    pub resource_id: String,
    pub r#type: String,
    pub description: String,
}

/// 指定したサービス名・リソースタイプの手動管理リソースの一覧を取得する
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
async fn get_manual_management_resource_list(
    state: &AppContext,
    workspace_directory: &str,
    service_name: &str,
    resource_name: &str,
) -> Result<Vec<ManualManagementResourceSummary>, ManualManagementResourceError> {
    let manual_management_resources =
        get_manual_management_resources(state, workspace_directory).await?;
    let mut result: Vec<ManualManagementResourceSummary> = Vec::new();

    let r#type = format!("AWS::{}::{}", service_name, resource_name);
    for (resource_id, manual_management_resource) in manual_management_resources.resources.iter() {
        if manual_management_resource.r#type == r#type {
            result.push(ManualManagementResourceSummary {
                resource_id: resource_id.clone(),
                r#type: manual_management_resource.r#type.clone(),
                description: manual_management_resource.description.clone(),
            })
        }
    }

    return Ok(result);
}

/// サイドメニューに表示する手動管理リソース用のサービス名とリソース種別の一覧を取得
///
/// - `workspace_directory` - ワークスペースのディレクトリパス
async fn load_manual_management_resource_summary(
    state: &AppContext,
    workspace_directory: &str,
) -> Result<Vec<Resource>, ManualManagementResourceError> {
    let manual_management_resources =
        get_manual_management_resources(state, workspace_directory).await?;
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

async fn load_manual_resource_meta(
    state: &AppContext,
    workspace_directory: &str,
) -> Result<ManualManagementMeta, ManualManagementResourceError> {
    // 手動管理リソースのmeta.jsonが存在するか確認
    let meta_path = PathBuf::from(workspace_directory).join(MANUAL_MANAGEMENT_RESOURCES_META_FILE);
    if !state.file_system.path_exists(&meta_path) {
        // 空のJSONを作成
        let empty_json = ManualManagementMeta {
            reasons: HashMap::new(),
        };
        let empty_json = serde_json::to_string(&empty_json).unwrap();
        state
            .file_system
            .write_file(&meta_path, empty_json.as_bytes())
            .await?;
    }

    // meta.jsonを読み込む
    let meta_json = state.file_system.read_file(&meta_path).await?;
    let meta_json = serde_json::from_str::<ManualManagementMeta>(&meta_json)?;
    return Ok(meta_json);
}

async fn get_manual_resource_properties(
    state: &AppContext,
    workspace_directory: &str,
    resource_id: &str,
) -> Result<HashMap<String, Value>, ManualManagementResourceError> {
    let manual_management_resources =
        get_manual_management_resources(state, workspace_directory).await?;
    let resource = manual_management_resources
        .resources
        .get(resource_id)
        .ok_or(ManualManagementResourceError::App(AppError::new(
            "Resource ID not found.",
        )))?;
    return Ok(resource.properties.clone());
}

async fn get_manual_resource_reasons(
    state: &AppContext,
    workspace_directory: &str,
    resource_id: &str,
) -> Result<Value, ManualManagementResourceError> {
    let meta_json = load_manual_resource_meta(state, workspace_directory).await?;
    if let Some(reasons) = meta_json.reasons.get(resource_id) {
        // 指定されたリソースIDのreasonsが存在する場合はそのまま返す
        return Ok(serde_json::to_value(reasons.clone())?);
    } else {
        // 指定されたリソースIDのreasonsが存在しない場合は空のオブジェクトを返す
        return Ok(Value::Object(serde_json::Map::new()));
    }
}

async fn update_manual_resource_meta(
    state: &AppContext,
    workspace_directory: &str,
    update_manual_resource_meta: ManualManagementMetaUpdate,
) -> Result<(), ManualManagementResourceError> {
    let manual_resource_meta = load_manual_resource_meta(state, workspace_directory).await?;
    let new_manual_resource_meta = ManualManagementMeta {
        reasons: match update_manual_resource_meta.reasons {
            Some(update_meta) => {
                let mut new_reasons = manual_resource_meta.reasons.clone();
                new_reasons.insert(update_meta.resource_id, update_meta.reasons);
                new_reasons
            }
            None => manual_resource_meta.reasons,
        },
    };
    let meta_path = PathBuf::from(workspace_directory).join(MANUAL_MANAGEMENT_RESOURCES_META_FILE);
    let manual_resource_meta_json = serde_json::to_string(&new_manual_resource_meta).unwrap();
    state
        .file_system
        .write_file(&meta_path, manual_resource_meta_json.as_bytes())
        .await?;
    return Ok(());
}

async fn update_manual_resource_properties(
    state: &AppContext,
    workspace_directory: &str,
    resource_id: &str,
    properties: String,
) -> Result<(), ManualManagementResourceError> {
    let mut manual_management_resources =
        get_manual_management_resources(&state, workspace_directory).await?;
    let resource = manual_management_resources
        .resources
        .get_mut(resource_id)
        .ok_or(ManualManagementResourceError::App(AppError::new(
            "Resource ID not found.",
        )))?;
    let parse_properties: Value = serde_json::from_str(&properties)?;
    if parse_properties.is_object() == false {
        return Err(ManualManagementResourceError::App(AppError::new(
            "Invalid properties format. Expected an object.",
        )));
    }
    let parse_properties: HashMap<String, Value> = parse_properties
        .as_object()
        .unwrap()
        .clone()
        .into_iter()
        .collect();
    resource.properties = parse_properties;
    save_manual_management_resources(state, workspace_directory, &manual_management_resources)
        .await?;
    return Ok(());
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn new_manual_management_resource_command(
    app_context_state: State<'_, AppContext>,
    api_context_state: State<'_, ApiContext>,
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
        &app_context_state,
        &api_context_state,
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

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_manual_management_resource_list_command(
    state: State<'_, AppContext>,
    window: tauri::Window,
    service_name: &str,
    resource_name: &str,
) -> Result<CommandResult<Vec<ManualManagementResourceSummary>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_manual_management_resource_list(
        &state,
        window_state.workspace_directory.as_str(),
        service_name,
        resource_name,
    )
    .await
    {
        Ok(list) => Ok(CommandResult::success(list)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn load_manual_management_resource_summary_command(
    state: State<'_, AppContext>,
    window: tauri::Window,
) -> Result<CommandResult<Vec<Resource>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match load_manual_management_resource_summary(
        &state,
        window_state.workspace_directory.as_str(),
    )
    .await
    {
        Ok(summary) => Ok(CommandResult::success(summary)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_manual_resource_properties_command(
    state: State<'_, AppContext>,
    window: tauri::Window,
    resource_id: &str,
) -> Result<CommandResult<HashMap<String, Value>>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_manual_resource_properties(
        &state,
        window_state.workspace_directory.as_str(),
        resource_id,
    )
    .await
    {
        Ok(properties) => Ok(CommandResult::success(properties)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn get_manual_resource_reasons_command(
    state: State<'_, AppContext>,
    window: tauri::Window,
    resource_id: &str,
) -> Result<CommandResult<Value>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match get_manual_resource_reasons(
        &state,
        window_state.workspace_directory.as_str(),
        resource_id,
    )
    .await
    {
        Ok(reasons) => Ok(CommandResult::success(reasons)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn update_manual_resource_meta_command(
    state: State<'_, AppContext>,
    window: tauri::Window,
    resource_id: &str,
    reasons: HashMap<String, String>,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match update_manual_resource_meta(
        &state,
        window_state.workspace_directory.as_str(),
        ManualManagementMetaUpdate {
            reasons: Some(ManualManagementMetaReasonsUpdate {
                resource_id: resource_id.to_string(),
                reasons,
            }),
        },
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[coverage(off)]
#[tauri::command(rename_all = "snake_case")]
pub async fn update_manual_resource_properties_command(
    state: State<'_, AppContext>,
    window: tauri::Window,
    resource_id: &str,
    properties: String,
) -> Result<CommandResult<()>, CommandResult> {
    let window_state = match get_window_state(window) {
        Some(state) => state,
        None => {
            return Err(CommandResult::failed("Window state not found"));
        }
    };
    return match update_manual_resource_properties(
        &state,
        window_state.workspace_directory.as_str(),
        resource_id,
        properties,
    )
    .await
    {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[cfg(test)]
#[coverage(off)]
mod get_manual_management_resources_tests {
    use super::*;
    use crate::utils::context::file::MockFileSystem;
    use crate::utils::context::http_client::MockHttpClient;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_manual_management_resources_creates_file_if_not_exists() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        mock_file_system.expect_path_exists().return_const(false);

        mock_file_system
            .expect_write_file()
            // 書き込み内容が期待通りであること
            .withf(|_, contents| {
                let contents = std::str::from_utf8(contents).unwrap();
                let config = serde_json::from_str::<ManualManagementResources>(contents);
                if config.is_ok() {
                    return config.unwrap().resources.len() == 0;
                } else {
                    return false;
                }
            })
            .returning(|_, _| Ok(()));

        let initial_data = ManualManagementResources {
            resources: HashMap::new(),
        };
        mock_file_system.expect_read_file().returning(move |_| {
            let initial_json = serde_json::to_string(&initial_data).unwrap();
            Ok(initial_json)
        });

        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_dir = "/test/workspace";

        // ######### 実行 #########
        let result = get_manual_management_resources(&app_context, workspace_dir).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let resources = result.unwrap();
        assert_eq!(resources.resources.len(), 0);
    }

    #[tokio::test]
    async fn test_get_manual_management_resources_reads_existing_file() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        mock_file_system.expect_path_exists().return_const(true);

        mock_file_system.expect_read_file().returning(|_| {
            let existing_data = ManualManagementResources {
                resources: {
                    let mut map = HashMap::new();
                    map.insert(
                        "TestResource".to_string(),
                        ManualManagementResource {
                            description: "Test Description".to_string(),
                            r#type: "AWS::S3::Bucket".to_string(),
                            properties: HashMap::new(),
                        },
                    );
                    map
                },
            };
            let existing_json = serde_json::to_string(&existing_data).unwrap();
            Ok(existing_json)
        });

        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_dir = "/test/workspace";

        // ######### 実行 #########
        let result = get_manual_management_resources(&app_context, workspace_dir).await;

        // ######### 検証 #########
        assert!(result.is_ok());
        let resources = result.unwrap();
        assert_eq!(resources.resources.len(), 1);
        assert!(resources.resources.contains_key("TestResource"));
        assert_eq!(
            resources.resources.get("TestResource").unwrap().description,
            "Test Description"
        );
    }

    #[tokio::test]
    async fn test_get_manual_management_resources_handles_invalid_json() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        mock_file_system.expect_path_exists().return_const(true);

        mock_file_system
            .expect_read_file()
            .returning(|_| Ok("{ invalid json }".to_string()));

        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_dir = "/test/workspace";

        // ######### 実行 #########
        let result = get_manual_management_resources(&app_context, workspace_dir).await;

        // ######### 検証 #########
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManualManagementResourceError::Json(_)
        ));
    }

    #[tokio::test]
    async fn test_get_manual_management_resources_handles_io_error() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();

        mock_file_system.expect_path_exists().return_const(true);

        mock_file_system.expect_read_file().returning(|_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found",
            ))
        });

        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_dir = "/test/workspace";

        // ######### 実行 #########
        let result = get_manual_management_resources(&app_context, workspace_dir).await;

        // ######### 検証 #########
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManualManagementResourceError::Io(_)
        ));
    }
}

#[cfg(test)]
#[coverage(off)]
mod save_manual_management_resources_tests {
    use super::*;
    use crate::utils::context::file::MockFileSystem;
    use crate::utils::context::http_client::MockHttpClient;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_save_manual_management_resources_succeeds() {
        // ######### 準備 #########

        let workspace_dir = "/test/workspace";
        let manual_management_resources = ManualManagementResources {
            resources: HashMap::new(),
        };

        let mut mock_file_system = MockFileSystem::new();

        mock_file_system
            .expect_write_file()
            .withf(move |path, contents| {
                let expected_path =
                    PathBuf::from(workspace_dir).join(MANUAL_MANAGEMENT_RESOURCES_FILE);
                if path != &expected_path {
                    return false;
                }
                let contents_str = std::str::from_utf8(contents).unwrap();
                let parsed: Result<ManualManagementResources, _> =
                    serde_json::from_str(contents_str);
                if parsed.is_err() {
                    return false;
                }
                let parsed = parsed.unwrap();
                parsed.resources.len() == 0
            })
            .returning(|_, _| Ok(()));
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = save_manual_management_resources(
            &app_context,
            workspace_dir,
            &manual_management_resources,
        );

        // ######### 検証 #########
        assert!(result.await.is_ok());
    }

    #[tokio::test]
    async fn test_save_manual_management_resources_handles_write_error() {
        // ######### 準備 #########

        let workspace_dir = "/test/workspace";
        let manual_management_resources = ManualManagementResources {
            resources: HashMap::new(),
        };

        let mut mock_file_system = MockFileSystem::new();

        mock_file_system
            .expect_write_file()
            .returning(|_, _| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Permission denied",
                ))
            });
        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = save_manual_management_resources(
            &app_context,
            workspace_dir,
            &manual_management_resources,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManualManagementResourceError::Io(_)
        ));
    }
}

#[cfg(test)]
#[coverage(off)]
mod new_manual_management_resource_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod get_manual_management_resource_list_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod load_manual_management_resource_summary_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod load_manual_resource_meta_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod get_manual_resource_properties_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod get_manual_resource_reasons_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod update_manual_resource_meta_tests {
    use super::*;
}

#[cfg(test)]
#[coverage(off)]
mod update_manual_resource_properties_tests {
    use std::sync::Arc;

    use crate::utils::context::{file::MockFileSystem, http_client::MockHttpClient};

    use super::*;

    /// プロパティの更新に成功すること
    #[tokio::test]
    async fn success() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        // get_manual_management_resources のモック
        mock_file_system.expect_path_exists().return_const(true);
        mock_file_system.expect_read_file().returning(|_| {
            Ok(r#"{ 
                "Resources": { 
                    "TestResource": { 
                        "Description": "Test Description",
                        "Type": "AWS::S3::Bucket", 
                        "Properties": {
                            "OldKey": "oldValue"
                        }
                    }
                } 
            }"#
            .to_string())
        });

        // save_manual_management_resources のモック
        mock_file_system
            .expect_write_file()
            .returning(|_, _| Ok(()));

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_directory = "/test/workspace";
        let resource_id = "TestResource";
        let properties = r#"{"Key1": "value1", "Key2": 2}"#.to_string();

        // ######### 実行 #########
        let result = update_manual_resource_properties(
            &app_context,
            workspace_directory,
            resource_id,
            properties,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_ok());
    }

    /// get_manual_management_resources に失敗し、エラーを返すこと
    #[tokio::test]
    async fn get_manual_management_resources_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        // get_manual_management_resources のモック
        mock_file_system.expect_path_exists().return_const(true);
        mock_file_system.expect_read_file().returning(|_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "read_file error",
            ))
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_directory = "/test/workspace";
        let resource_id = "TestResource";
        let properties = r#"{"Key1": "value1", "Key2": 2}"#.to_string();

        // ######### 実行 #########
        let result = update_manual_resource_properties(
            &app_context,
            workspace_directory,
            resource_id,
            properties,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// 指定したリソースIDが存在しない場合、エラーを返すこと
    #[tokio::test]
    async fn resource_id_not_found() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        // get_manual_management_resources のモック
        mock_file_system.expect_path_exists().return_const(true);
        mock_file_system.expect_read_file().returning(|_| {
            Ok(r#"{ 
                "Resources": { 
                    "AnotherResource": { 
                        "Description": "Test Description",
                        "Type": "AWS::S3::Bucket", 
                        "Properties": {}
                    }
                } 
            }"#
            .to_string())
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_directory = "/test/workspace";
        let resource_id = "TestResource"; // 存在しないリソースID
        let properties = r#"{"Key1": "value1", "Key2": 2}"#.to_string();

        // ######### 実行 #########
        let result = update_manual_resource_properties(
            &app_context,
            workspace_directory,
            resource_id,
            properties,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// serde_json::from_str に失敗し、エラーを返すこと
    #[tokio::test]
    async fn from_str_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        // get_manual_management_resources のモック
        mock_file_system.expect_path_exists().return_const(true);
        mock_file_system.expect_read_file().returning(|_| {
            Ok(r#"{ 
                "Resources": { 
                    "TestResource": { 
                        "Description": "Test Description",
                        "Type": "AWS::S3::Bucket", 
                        "Properties": {}
                    }
                } 
            }"#
            .to_string())
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_directory = "/test/workspace";
        let resource_id = "TestResource";
        let properties = r#"invalid json"#.to_string(); // 無効なJSON

        // ######### 実行 #########
        let result = update_manual_resource_properties(
            &app_context,
            workspace_directory,
            resource_id,
            properties,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// プロパティがおbジェクトでない場合、エラーを返すこと
    #[tokio::test]
    async fn properties_not_object() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        // get_manual_management_resources のモック
        mock_file_system.expect_path_exists().return_const(true);
        mock_file_system.expect_read_file().returning(|_| {
            Ok(r#"{ 
                "Resources": { 
                    "TestResource": { 
                        "Description": "Test Description",
                        "Type": "AWS::S3::Bucket", 
                        "Properties": {}
                    }
                } 
            }"#
            .to_string())
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_directory = "/test/workspace";
        let resource_id = "TestResource";
        let properties = r#"[]"#.to_string(); // オブジェクトでないJSON

        // ######### 実行 #########
        let result = update_manual_resource_properties(
            &app_context,
            workspace_directory,
            resource_id,
            properties,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }

    /// save_manual_management_resources に失敗し、エラーを返すこと
    #[tokio::test]
    async fn save_manual_management_resources_err() {
        // ######### 準備 #########
        let mut mock_file_system = MockFileSystem::new();
        let mock_http_client = MockHttpClient::new();

        // get_manual_management_resources のモック
        mock_file_system.expect_path_exists().return_const(true);
        mock_file_system.expect_read_file().returning(|_| {
            Ok(r#"{ 
                "Resources": { 
                    "TestResource": { 
                        "Description": "Test Description",
                        "Type": "AWS::S3::Bucket", 
                        "Properties": {}
                    }
                } 
            }"#
            .to_string())
        });

        // save_manual_management_resources のモック
        mock_file_system.expect_write_file().returning(|_, _| {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "write_file error",
            ))
        });

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        let workspace_directory = "/test/workspace";
        let resource_id = "TestResource";
        let properties = r#"{"Key1": "value1", "Key2": 2}"#.to_string();

        // ######### 実行 #########
        let result = update_manual_resource_properties(
            &app_context,
            workspace_directory,
            resource_id,
            properties,
        )
        .await;

        // ######### 検証 #########
        assert!(result.is_err());
    }
}
