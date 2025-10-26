use super::super::super::utils::AppError;
use crate::config::app_config::APP_CONFIG_DIRECTORY_NAME;
use crate::utils::context::app_context::AppContext;
use regex::Regex;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs, // TODO:削除
    io::Cursor,
    path::PathBuf,
};
use tauri::State;
use thiserror::Error;
use zip::ZipArchive;

use dirs::config_local_dir;

#[derive(Debug, Error)]
pub enum DlSchemaError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0:?}")]
    Zip(#[from] zip::result::ZipError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// CloudFormationのサービスとリソース種別のサマリー結果を保存するファイル名
pub const SUMMARY_SERVICE_LIST_FILE: &str = "summary_service_list.json";

fn get_resource_provider_dl_path(region: &str) -> String {
    return format!(
        "https://schema.cloudformation.{}.amazonaws.com/CloudformationSchema.zip",
        region
    );
}

fn get_resource_provider_save_path(region: &str) -> Result<PathBuf, DlSchemaError> {
    return match config_local_dir() {
        Some(dir) => Ok(dir
            .join(APP_CONFIG_DIRECTORY_NAME)
            .join(format!("CloudformationSchema-{}", region))),
        None => Err(DlSchemaError::App(AppError::new(
            "Failed to get config local dir",
        ))),
    };
}

pub async fn generate_summary_service_list(
    state: State<'_, AppContext>,
    output_dir: PathBuf,
) -> Result<(), DlSchemaError> {
    let mut result_map: HashMap<String, HashSet<String>> = HashMap::new();
    let re = Regex::new(r"aws-([a-z\d]+)-([a-z\d]+)\.json").unwrap();
    // TODO:tokio版を使用するように修正
    for entry in fs::read_dir(&output_dir)? {
        if let Some(entry) = entry?.file_name().to_str() {
            if re.is_match(entry) {
                let file_path = output_dir.join(entry);
                let schema_json = state.file_system.read_file(&file_path).await?;
                let schema_json: Value = serde_json::from_str(&schema_json)?;
                if schema_json.is_object() == true {
                    let type_name = schema_json["typeName"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    let parts: Vec<&str> = type_name.split("::").collect();
                    let (_, service_name, resource_type): (&str, &str, &str) = match parts[..] {
                        [a, b, c] => (a, b, c),
                        _ => continue,
                    };
                    if let Some(original) = result_map.get_mut(service_name) {
                        original.insert(resource_type.to_string());
                    } else {
                        let mut resource_types = HashSet::new();
                        resource_types.insert(resource_type.to_string());
                        result_map.insert(service_name.to_string(), resource_types);
                    }
                }
            }
        }
    }
    let summary_file_path = output_dir.join(SUMMARY_SERVICE_LIST_FILE);
    let summary_json = serde_json::to_string(&result_map)?;
    state
        .file_system
        .write_file(&summary_file_path, summary_json.as_bytes())
        .await?;
    return Ok(());
}

pub fn get_resource_provider_save_dir(region: &str) -> Result<PathBuf, DlSchemaError> {
    return match config_local_dir() {
        Some(dir) => Ok(dir
            .join(APP_CONFIG_DIRECTORY_NAME)
            .join("CloudformationSchema")
            .join(region)),
        None => Err(DlSchemaError::App(AppError::new(
            "Failed to get config local dir",
        ))),
    };
}

pub async fn dl_resource_provider(
    state: State<'_, AppContext>,
    region: &str,
) -> Result<(), DlSchemaError> {
    let url = get_resource_provider_dl_path(region);
    let save_path = get_resource_provider_save_path(region)?;
    if save_path.exists() {
        std::fs::remove_file(&save_path)?
    }

    // Zipファイルをダウンロード
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    // ZIPファイルを解凍
    let content = Cursor::new(bytes);
    let output_dir = get_resource_provider_save_dir(region)?;
    let mut archive = ZipArchive::new(content)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = output_dir.join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    generate_summary_service_list(state, output_dir).await?;
    return Ok(());
}
