use crate::command::app_config::APP_CONFIG_DIRECTORY_NAME;
use crate::utils::AppError;
use dirs::config_local_dir;
use thiserror::Error;
use tokio::fs;

use super::super::api::cloudformation;
use super::super::api::github;

const AWS_CLI_OWNER: &str = "aws";
const AWS_CLI_REPO: &str = "aws-cli";
const AWS_CLI_PATH: &str = "awscli/botocore/data";

#[derive(Debug, Error)]
pub enum GetServiceDataDownloadError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("octocrab error: {0}")]
    Octocrab(#[from] octocrab::Error),
}

async fn get_service_data_dl() -> Result<(), GetServiceDataDownloadError> {
    let branch = github::branch::get_branch("aws", "aws-cli", "v2")
        .await
        .unwrap();

    let commit_sha = match branch["commit"]["sha"].as_str() {
        Some(sha) => sha,
        None => {
            println!("Failed to get commit SHA from branch data");
            return Err(GetServiceDataDownloadError::App(AppError::new(
                "Failed to get commit SHA",
            )));
        }
    };
    println!("Branch commit SHA: {:?}", commit_sha);

    let list = octocrab::instance()
        .repos(AWS_CLI_OWNER, AWS_CLI_REPO)
        .get_content()
        .path(AWS_CLI_PATH)
        .r#ref(commit_sha)
        .send()
        .await?;
    let service = octocrab::instance()
        .repos(AWS_CLI_OWNER, AWS_CLI_REPO)
        .get_content()
        .path(format!("{}/{}", AWS_CLI_PATH, list.items[0].name))
        .r#ref(commit_sha)
        .send()
        .await?;
    let mut content = octocrab::instance()
        .repos(AWS_CLI_OWNER, AWS_CLI_REPO)
        .get_content()
        .path(format!(
            "{}/{}/{}/{}",
            AWS_CLI_PATH, list.items[0].name, service.items[0].name, "service-2.json"
        ))
        .r#ref(commit_sha)
        .send()
        .await?;
    let contents = content.take_items();
    let c = &contents[0];
    if c.r#type != "file" {
        return Err(GetServiceDataDownloadError::App(AppError::new(
            "Failed to get service-2.json",
        )));
    }
    let decoded_content = c.decoded_content().unwrap();
    println!("Decoded content");

    let save_path = match config_local_dir() {
        Some(dir) => Some(dir.join(APP_CONFIG_DIRECTORY_NAME).join("aws-cli")),
        None => None,
    };
    let save_path = save_path.unwrap();
    println!("{:?}", save_path);
    std::fs::create_dir_all(&save_path);
    return match fs::write(
        save_path.join(format!("{}.json", list.items[0].name)),
        decoded_content,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(GetServiceDataDownloadError::App(AppError::new(
            "Failed to write service-2.json",
        ))),
    };
}

#[tauri::command]
pub async fn setup_app() -> Result<(), ()> {
    if let Err(_) = cloudformation::schema::dl_resource_provider("us-east-1").await {
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
