use std::collections::HashMap;

use super::super::api::cloudformation;
use crate::utils::context::app_context::AppContext;
use crate::utils::{AppError, CommandResult};
use tauri::State;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CloudFormationSchemaError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("cloud formation error: {0}")]
    Cloudformation(#[from] cloudformation::schema::DlSchemaError),
}

pub async fn get_cloudformation_schema(
    state: &AppContext,
    service_name: &str,
    resource_name: &str,
) -> Result<String, CloudFormationSchemaError> {
    let schema_directory = cloudformation::schema::get_resource_provider_save_dir(
        state.file_system.clone(),
        "us-east-1",
    )?;
    let filename = format!(
        "aws-{}-{}.json",
        service_name.to_lowercase(),
        resource_name.to_lowercase()
    );
    let schema_path = schema_directory.join(filename);
    if !state.file_system.path_exists(&schema_path) {
        return Err(CloudFormationSchemaError::App(AppError::new(
            "Schema file not found",
        )));
    }
    let schema_json = state.file_system.read_file(&schema_path).await?;

    return Ok(schema_json);
}

async fn get_aws_service_list(
    state: &AppContext,
) -> Result<HashMap<String, Vec<String>>, CloudFormationSchemaError> {
    let schema_directory =
        super::super::api::cloudformation::schema::get_resource_provider_save_dir(
            state.file_system.clone(),
            "us-east-1",
        )?;
    let summary_file_path =
        schema_directory.join(cloudformation::schema::SUMMARY_SERVICE_LIST_FILE);
    if !state.file_system.path_exists(&summary_file_path) {
        cloudformation::schema::generate_summary_service_list(&state, schema_directory.clone())
            .await?;
    }
    let summary_json = state.file_system.read_file(&summary_file_path).await?;
    let summary_json = serde_json::from_str::<HashMap<String, Vec<String>>>(&summary_json)?;
    return Ok(summary_json);
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_cloudformation_schema_command(
    state: State<'_, AppContext>,
    service_name: &str,
    resource_name: &str,
) -> Result<CommandResult<String>, CommandResult> {
    return match get_cloudformation_schema(&state, service_name, resource_name).await {
        Ok(schema_json) => Ok(CommandResult::success(schema_json)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_aws_service_list_command(
    state: State<'_, AppContext>,
) -> Result<CommandResult<HashMap<String, Vec<String>>>, CommandResult> {
    return match get_aws_service_list(&state).await {
        Ok(services) => Ok(CommandResult::success(services)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[cfg(test)]
#[coverage(off)]
mod get_cloudformation_schema_tests {
    use super::*;
    use std::{
        io::{Error, ErrorKind},
        path::PathBuf,
        str::FromStr,
        sync::Arc,
    };

    use crate::{
        config::app_config::APP_CONFIG_DIRECTORY_NAME,
        utils::context::{
            app_context::AppContext, file::MockFileSystem, http_client::MockHttpClient,
        },
    };

    /// cloudformation schemaを取得できることを確認
    #[tokio::test]
    async fn get_cloudformation_schema_success() {
        // ######### 準備 #########
        let mut mock_file_system: MockFileSystem = MockFileSystem::new();

        // config_local_dirのモック
        let config_local_dir_path = PathBuf::from_str("config_local_dir_path").unwrap();
        let config_local_dir_path_clone = config_local_dir_path.clone();
        mock_file_system
            .expect_config_local_dir()
            .returning(move || return Some(config_local_dir_path_clone.clone()));

        // path_existsのモック
        mock_file_system.expect_path_exists().return_const(true);

        // read_fileのモック
        let app_config_json = r#"app_config_json_content"#;
        let service_name = "ServiceName";
        let resource_name = "ResourceName";
        mock_file_system
            .expect_read_file()
            .withf(move |path| {
                let schema_directory = config_local_dir_path
                    .clone()
                    .join(APP_CONFIG_DIRECTORY_NAME)
                    .join("CloudformationSchema")
                    .join("us-east-1");

                let filename = format!(
                    "aws-{}-{}.json",
                    service_name.to_lowercase(),
                    resource_name.to_lowercase()
                );

                let schema_path = schema_directory.join(filename);

                return path == &schema_path;
            })
            .returning(move |_path| {
                return Ok(app_config_json.to_string());
            });

        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_cloudformation_schema(&app_context, service_name, resource_name).await;

        // ######### 検証 #########
        // cloudformation schemaの取得できたこと
        assert!(result.is_ok());
        let schema = result.unwrap();
        assert_eq!(schema, app_config_json);
    }

    // config_local_dirの戻り値がNoneの場合、get_cloudformation_schemaがErrを返すことを確認
    #[tokio::test]
    async fn get_cloudformation_schema_err_get_resource_provider() {
        // ######### 準備 #########
        let mut mock_file_system: MockFileSystem = MockFileSystem::new();

        // config_local_dirのモック
        mock_file_system
            .expect_config_local_dir()
            .returning(|| return None);

        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_cloudformation_schema(&app_context, "service_name", "resource_name").await;

        // ######### 検証 #########
        // get_cloudformation_schemaの戻り値がErrであること
        assert!(result.is_err());
    }

    // path_existsがfalseの場合、get_cloudformation_schemaがErrを返すことを確認
    #[tokio::test]
    async fn get_cloudformation_schema_err_path_exists() {
        // ######### 準備 #########
        let mut mock_file_system: MockFileSystem = MockFileSystem::new();

        // config_local_dirのモック
        mock_file_system
            .expect_config_local_dir()
            .returning(|| return Some(PathBuf::from_str("config_local_dir_path").unwrap()));

        // path_existsのモック
        mock_file_system.expect_path_exists().return_const(false);

        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_cloudformation_schema(&app_context, "service_name", "resource_name").await;

        // ######### 検証 #########
        // get_cloudformation_schemaの戻り値がErrであること
        assert!(result.is_err());
    }

    // read_fileの戻り値がErrの場合、get_cloudformation_schemaがErrを返すことを確認
    #[tokio::test]
    async fn get_cloudformation_schema_err_read_file() {
        // ######### 準備 #########
        let mut mock_file_system: MockFileSystem = MockFileSystem::new();

        // config_local_dirのモック
        mock_file_system
            .expect_config_local_dir()
            .returning(|| return Some(PathBuf::from_str("config_local_dir_path").unwrap()));

        // path_existsのモック
        mock_file_system.expect_path_exists().return_const(true);

        // read_fileのモック
        mock_file_system
            .expect_read_file()
            .returning(|_path| return Err(Error::new(ErrorKind::Other, "read_file error")));

        let mock_http_client = MockHttpClient::new();

        let app_context = AppContext {
            file_system: Arc::new(mock_file_system),
            http_client: Arc::new(mock_http_client),
        };

        // ######### 実行 #########
        let result = get_cloudformation_schema(&app_context, "service_name", "resource_name").await;

        // ######### 検証 #########
        // get_cloudformation_schemaの戻り値がErrであること
        assert!(result.is_err());
    }
}
