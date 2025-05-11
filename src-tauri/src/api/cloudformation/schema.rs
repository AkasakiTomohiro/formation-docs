use super::super::super::utils::AppError;
use crate::command::app_config::APP_CONFIG_DIRECTORY_NAME;
use std::{io::Cursor, path::PathBuf};
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
}

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

pub async fn dl_resource_provider(region: &str) -> Result<(), DlSchemaError> {
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
    return Ok(());
}
