use std::{collections::HashMap, fs};

use super::super::api::cloudformation;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::utils::{AppError, CommandResult};

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

pub fn get_cloudformation_schema(
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
    if !schema_path.exists() {
        return Err(CloudFormationSchemaError::App(AppError::new(
            "Schema file not found",
        )));
    }
    let schema_json = fs::read_to_string(&schema_path)?;

    return Ok(schema_json);
}

fn get_aws_service_list() -> Result<HashMap<String, Vec<String>>, CloudFormationSchemaError> {
    let schema_directory =
        super::super::api::cloudformation::schema::get_resource_provider_save_dir("us-east-1")?;
    let summary_file_path =
        schema_directory.join(cloudformation::schema::SUMMARY_SERVICE_LIST_FILE);
    if !summary_file_path.exists() {
        cloudformation::schema::generate_summary_service_list(schema_directory.clone())?;
    }
    let summary_json = fs::read_to_string(&summary_file_path)?;
    let summary_json = serde_json::from_str::<HashMap<String, Vec<String>>>(&summary_json)?;
    return Ok(summary_json);
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_cloudformation_schema_command(
    service_name: &str,
    resource_name: &str,
) -> Result<CommandResult<String>, CommandResult> {
    return match get_cloudformation_schema(service_name, resource_name) {
        Ok(schema_json) => Ok(CommandResult::success(schema_json)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_aws_service_list_command(
) -> Result<CommandResult<HashMap<String, Vec<String>>>, CommandResult> {
    return match get_aws_service_list() {
        Ok(services) => Ok(CommandResult::success(services)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
