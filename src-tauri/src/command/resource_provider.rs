use std::io::Cursor;
use std::path::PathBuf;

use crate::api::github::repos::dl_zip_file;
use crate::api::github::repos::GithubReposError;
use crate::command::app_config::read_app_config;
use crate::command::app_config::save_app_config;
use crate::command::app_config::AppConfigUpdate;
use crate::command::app_config::APP_CONFIG_DIRECTORY_NAME;
use crate::utils::AppError;
use chrono::Utc;
use dirs::config_local_dir;
use globset::Glob;
use regex::Regex;
use thiserror::Error;
use zip::ZipArchive;

use super::super::api::cloudformation;
use super::super::api::github;
use super::app_config::AppConfig;

const AWS_CLI_OWNER: &str = "aws";
const AWS_CLI_REPO: &str = "aws-cli";
const AWS_CLI_PATH: &str = "awscli/botocore/data";

#[derive(Debug, Error)]
pub enum ResourceProviderError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("github repos error: {0:?}")]
    GithubRepo(#[from] GithubReposError),
    #[error("zip error: {0:?}")]
    Zip(#[from] zip::result::ZipError),
    #[error("globset error: {0:?}")]
    Globset(#[from] globset::Error),
    #[error("app config error: {0}")]
    AppConfig(#[from] crate::command::app_config::AppConfigError),
    #[error("cloud formation error: {0}")]
    Cloudformation(#[from] super::super::api::cloudformation::schema::DlSchemaError),
}

fn get_service_data_save_dir() -> Result<PathBuf, ResourceProviderError> {
    return match config_local_dir() {
        Some(dir) => Ok(dir.join(APP_CONFIG_DIRECTORY_NAME).join("aws-cli")),
        None => Err(ResourceProviderError::App(AppError::new(
            "Failed to get config local dir",
        ))),
    };
}

async fn get_service_data_dl(app_config: AppConfig) -> Result<(), ResourceProviderError> {
    let branch = github::repos::get_branch("aws", "aws-cli", "v2")
        .await
        .unwrap();

    // コミットハッシュ取得
    let commit_sha = match branch["commit"]["sha"].as_str() {
        Some(sha) => sha,
        None => {
            println!("Failed to get commit SHA from branch data");
            return Err(ResourceProviderError::App(AppError::new(
                "Failed to get commit SHA",
            )));
        }
    };
    println!("Branch commit SHA: {:?}", commit_sha);

    // コミットハッシュが保存されているものと一致しない場合のみダウンロード
    if app_config.aws_cli_commit_hash.is_some()
        && app_config.aws_cli_commit_hash.unwrap() == commit_sha.to_string()
    {
        println!("Already downloaded service data, skipping download.");
        return Ok(());
    }

    // aws-cliのリポジトリをZipダウンロード
    let bytes = dl_zip_file(AWS_CLI_OWNER, AWS_CLI_REPO, commit_sha).await?;

    // Zipファイルを解凍&ファイル保存
    let content = Cursor::new(bytes);
    let output_dir = get_service_data_save_dir()?;
    std::fs::create_dir_all(&output_dir)?;
    let mut archive = ZipArchive::new(content)?;

    let glob =
        Glob::new(format!("**/{}/*/*/service-2.json", AWS_CLI_PATH).as_str())?.compile_matcher();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        if file.is_file() && glob.is_match(file.name()) {
            // `awscli/botocore/data/` の直後のディレクトリ名をキャプチャ
            let re = Regex::new(format!(r"{}/([^/]+)/", AWS_CLI_PATH).as_str()).unwrap();
            // キャプチャグループに一致する部分(サービス名)を取得
            let service_name = re
                .captures(file.name())
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()));
            let service_name = match service_name {
                Some(name) => name,
                None => {
                    println!("Failed to get service name from file name: {}", file.name());
                    return Err(ResourceProviderError::App(AppError::new(
                        "Failed to get service name from file name",
                    )));
                }
            };

            // ファイルを保存
            let mut outfile =
                std::fs::File::create(output_dir.join(format!("{}.json", service_name)))?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    // コミットハッシュを保存
    save_app_config(AppConfigUpdate {
        workspaces: None,
        initialized: None,
        initialized_at: None,
        aws_cli_commit_hash: Some(Some(commit_sha.to_string())),
    })
    .await?;

    return Ok(());
}

async fn setup_app() -> Result<(), ResourceProviderError> {
    let app_config = read_app_config().await?;
    if app_config.initialized == false {
        cloudformation::schema::dl_resource_provider("us-east-1").await?;
        save_app_config(AppConfigUpdate {
            workspaces: None,
            initialized: Some(true),
            initialized_at: Some(Utc::now().to_string()),
            aws_cli_commit_hash: None,
        })
        .await?;
    }
    get_service_data_dl(app_config).await?;
    return Ok(());
}

#[tauri::command]
pub async fn setup_app_command() -> Result<(), ()> {
    match setup_app().await {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}
