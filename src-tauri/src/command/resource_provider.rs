use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

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
                .join(format!("CloudformationSchema-{}.zip", region)),
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
    let mut response = match reqwest::get(url).await {
        Ok(res) => res,
        Err(_) => {
            println!("Failed to get resource provider");
            return Err(());
        }
    };
    let mut file = match fs::File::create(save_path).await {
        Ok(file) => file,
        Err(_) => {
            println!("Failed to create resource provider file");
            return Err(());
        }
    };
    while let Some(chunk_result) = response.chunk().await.transpose() {
        match chunk_result {
            Ok(chunk) => {
                if let Err(e) = file.write_all(&chunk).await {
                    println!("Failed to write chunk to file: {}", e);
                    return Err(());
                }
            }
            Err(e) => {
                println!("Failed to read chunk from response: {}", e);
                return Err(());
            }
        }
    }
    return Ok(());
}

#[tauri::command]
pub async fn setup_app() -> Result<(), ()> {
    if let Err(_) = get_resource_provider_dl("us-east-1").await {
        return Err(());
    }
    if let Err(_) = super::app_config::initialized_app_config().await {
        return Err(());
    }
    return Ok(());
}
