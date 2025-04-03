use super::super::api::github;
use std::{io::Cursor, path::PathBuf};
use tokio::fs;
use zip::ZipArchive;

use dirs::config_local_dir;

fn get_resource_provider_dl_path(region: &str) -> String {
    return format!(
        "https://schema.cloudformation.{}.amazonaws.com/CloudformationSchema.zip",
        region
    );
}

fn get_resource_provider_save_path(region: &str) -> Option<PathBuf> {
    return match config_local_dir() {
        Some(dir) => Some(
            dir.join(super::app_config::APP_CONFIG_DIRECTORY_NAME)
                .join(format!("CloudformationSchema-{}", region)),
        ),
        None => None,
    };
}

fn get_resource_provider_save_dir(region: &str) -> Option<PathBuf> {
    return match config_local_dir() {
        Some(dir) => Some(
            dir.join(super::app_config::APP_CONFIG_DIRECTORY_NAME)
                .join("CloudformationSchema")
                .join(region),
        ),
        None => None,
    };
}

async fn get_resource_provider_dl(region: &str) -> Result<(), ()> {
    let url = get_resource_provider_dl_path(region);
    let save_path = match get_resource_provider_save_path(region) {
        Some(path) => path,
        None => {
            println!("Failed to get config local dir");
            return Err(());
        }
    };
    if save_path.exists() {
        if let Err(_) = fs::remove_file(&save_path).await {
            println!("Failed to remove existing resource provider file");
            return Err(());
        }
    }
    let response = match reqwest::get(url).await {
        Ok(res) => res,
        Err(_) => {
            println!("Failed to get resource provider");
            return Err(());
        }
    };
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Failed to read resource provider bytes");
            return Err(());
        }
    };
    let content = Cursor::new(bytes);

    // ZIPファイルを解凍
    let output_dir = match get_resource_provider_save_dir(region) {
        Some(path) => path,
        None => {
            println!("Failed to get config local dir");
            return Err(());
        }
    };
    let mut archive = match ZipArchive::new(content) {
        Ok(archive) => archive,
        Err(_) => {
            println!("Failed to create ZIP archive");
            return Err(());
        }
    };
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(_) => {
                println!("Failed to get file from ZIP archive");
                return Err(());
            }
        };
        let out_path = output_dir.join(file.name());

        if file.is_dir() {
            if let Err(_) = std::fs::create_dir_all(&out_path) {
                println!("Failed to create directory in output path");
                return Err(());
            };
        } else {
            if let Some(parent) = out_path.parent() {
                if let Err(_) = std::fs::create_dir_all(parent) {
                    println!("Failed to create parent directory in output path");
                    return Err(());
                };
            }
            let mut outfile = match std::fs::File::create(&out_path) {
                Ok(file) => file,
                Err(_) => {
                    println!("Failed to create file in output directory");
                    return Err(());
                }
            };
            if let Err(_) = std::io::copy(&mut file, &mut outfile) {
                println!("Failed to copy file to output directory");
                return Err(());
            };
        }
    }

    Ok(())
}

async fn get_service_data_dl() -> Result<(), ()> {
    let branch = github::branch::get_branch("aws", "aws-cli", "v2")
        .await
        .unwrap();

    let commit_sha = match branch["commit"]["sha"].as_str() {
        Some(sha) => sha,
        None => {
            println!("Failed to get commit SHA from branch data");
            return Err(());
        }
    };
    println!("Branch commit SHA: {:?}", commit_sha);

    return Ok(());
}

#[tauri::command]
pub async fn setup_app() -> Result<(), ()> {
    if let Err(_) = get_resource_provider_dl("us-east-1").await {
        return Err(());
    }
    if let Err(_) = get_service_data_dl().await {
        return Err(());
    }
    if let Err(_) = super::app_config::initialized_app_config().await {
        return Err(());
    }
    return Ok(());
}
