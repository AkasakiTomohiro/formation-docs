use std::fs;

use thiserror::Error;

use crate::utils::{AppError, CommandResult};

#[derive(Debug, Error)]
pub enum CloudFormationSchemaError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cloud formation error: {0}")]
    Cloudformation(#[from] super::super::api::cloudformation::schema::DlSchemaError),
}

fn get_cloudformation_schema(
    service_name: &str,
    resource_name: &str,
) -> Result<String, CloudFormationSchemaError> {
    let schema_directory =
        super::super::api::cloudformation::schema::get_resource_provider_save_dir("us-east-1")?;
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
