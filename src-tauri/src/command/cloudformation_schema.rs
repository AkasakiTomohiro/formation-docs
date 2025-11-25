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
    state: State<'_, AppContext>,
    service_name: &str,
    resource_name: &str,
) -> Result<String, CloudFormationSchemaError> {
    let schema_directory = cloudformation::schema::get_resource_provider_save_dir("us-east-1")?;
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
    state: State<'_, AppContext>,
) -> Result<HashMap<String, Vec<String>>, CloudFormationSchemaError> {
    let schema_directory =
        super::super::api::cloudformation::schema::get_resource_provider_save_dir("us-east-1")?;
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
    return match get_cloudformation_schema(state, service_name, resource_name).await {
        Ok(schema_json) => Ok(CommandResult::success(schema_json)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_aws_service_list_command(
    state: State<'_, AppContext>,
) -> Result<CommandResult<HashMap<String, Vec<String>>>, CommandResult> {
    return match get_aws_service_list(state).await {
        Ok(services) => Ok(CommandResult::success(services)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
