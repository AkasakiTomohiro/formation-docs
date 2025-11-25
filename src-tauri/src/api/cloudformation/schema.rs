use super::super::super::utils::AppError;
use crate::config::app_config::APP_CONFIG_DIRECTORY_NAME;
use crate::utils::context::app_context::AppContext;
use regex::Regex;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    path::PathBuf,
};
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
    Io(#[from] tokio::io::Error),
    #[error("zip error: {0:?}")]
    Zip(#[from] zip::result::ZipError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("task join error: {0}")]
    Join(#[from] tokio::task::JoinError),
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
    cxt: &AppContext,
    output_dir: PathBuf,
) -> Result<(), DlSchemaError> {
    let mut result_map: HashMap<String, HashSet<String>> = HashMap::new();
    let re = Regex::new(r"aws-([a-z\d]+)-([a-z\d]+)\.json").unwrap();
    let entries = cxt.file_system.read_dir(&output_dir).await?;
    for path in entries {
        if let Some(entry) = path.file_name().and_then(|f| f.to_str()) {
            if re.is_match(entry) {
                let file_path = output_dir.join(entry);
                let schema_json = cxt.file_system.read_file(&file_path).await?;
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
    cxt.file_system
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

pub async fn dl_resource_provider(ctx: &AppContext, region: &str) -> Result<(), DlSchemaError> {
    let url = get_resource_provider_dl_path(region);
    let save_path = get_resource_provider_save_path(region)?;
    if ctx.file_system.path_exists(&save_path) {
        ctx.file_system.remove_file(&save_path).await?
    }

    // Zipファイルをダウンロード
    let response = ctx.http_client.get(url).await?;
    let bytes = response.bytes().await?;

    let output_dir = get_resource_provider_save_dir(region)?;
    let output_dir_tmp = output_dir.clone();
    let file_system = ctx.file_system.clone();

    // ZIPファイルを解凍
    // 非同期処理内で同期処理を行うため（ZipArchiveが同期処理）、spawn_blockingで別スレッドに処理を移す
    // spawn_blocking内では非同期関数は使えないため、tokioではなくstdクレートを使用
    tokio::task::spawn_blocking(move || {
        let content = Cursor::new(bytes);
        let mut archive = ZipArchive::new(content)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let out_path = output_dir_tmp.join(file.name());

            if file.is_dir() {
                file_system.create_dir_all_sync(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    file_system.create_dir_all_sync(parent)?;
                }

                let mut outfile = file_system.touch_and_open_file(&out_path)?;
                file_system.copy_file_stream(&mut file, &mut outfile)?;
            }
        }
        Ok::<(), DlSchemaError>(())
    })
    .await??;

    generate_summary_service_list(&ctx, output_dir).await?;
    return Ok(());
}
