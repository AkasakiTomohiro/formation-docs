use std::{collections::HashMap, fs};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct AWSService {
    service_name: String,
    resources: Vec<String>,
}

fn get_aws_service_list() -> Result<Vec<AWSService>, CloudFormationSchemaError> {
    let schema_directory =
        super::super::api::cloudformation::schema::get_resource_provider_save_dir("us-east-1")?;

    let mut result_map: HashMap<String, Vec<String>> = HashMap::new();
    let re = Regex::new(r"aws-([a-z\d]+)-([a-z\d]+)\.json").unwrap();
    for entry in fs::read_dir(schema_directory)? {
        if let Some(entry) = entry?.file_name().to_str() {
            if let Some((service_name, resource_name)) = re.captures(entry).and_then(|caps| {
                Some((
                    caps.get(1).map(|m| m.as_str().to_string()),
                    caps.get(2).map(|m| m.as_str().to_string()),
                ))
            }) {
                let service_name = service_name.unwrap();
                let resource_name = resource_name.unwrap();
                if result_map.contains_key(&service_name) {
                    // 既にサービス名が存在する場合
                    result_map
                        .get_mut(&service_name)
                        .unwrap()
                        .push(resource_name);
                } else {
                    // 初出のサービス名の場合
                    result_map.insert(service_name, vec![resource_name]);
                }
            };
        }
    }

    let result: Vec<AWSService> = result_map
        .into_iter()
        .map(|(service_name, resources)| AWSService {
            service_name,
            resources,
        })
        .collect();
    return Ok(result);
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
pub fn get_aws_service_list_command() -> Result<CommandResult<Vec<AWSService>>, CommandResult> {
    return match get_aws_service_list() {
        Ok(services) => Ok(CommandResult::success(services)),
        Err(e) => Err(CommandResult::failed(e.to_string().as_str())),
    };
}
