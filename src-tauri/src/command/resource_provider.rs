use std::io::Cursor;
use std::path::PathBuf;

use crate::api::github::repos::dl_zip_file;
use crate::api::github::repos::GithubReposError;
use crate::command::app_config::APP_CONFIG_DIRECTORY_NAME;
use crate::utils::AppError;
use dirs::config_local_dir;
use globset::Glob;
use regex::Regex;
use thiserror::Error;
use zip::ZipArchive;

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
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("github repos error: {0:?}")]
    GithubRepo(#[from] GithubReposError),
    #[error("zip error: {0:?}")]
    Zip(#[from] zip::result::ZipError),
    #[error("globset error: {0:?}")]
    Globset(#[from] globset::Error),
}

fn get_service_data_save_dir() -> Result<PathBuf, GetServiceDataDownloadError> {
    return match config_local_dir() {
        Some(dir) => Ok(dir.join(APP_CONFIG_DIRECTORY_NAME).join("aws-cli")),
        None => Err(GetServiceDataDownloadError::App(AppError::new(
            "Failed to get config local dir",
        ))),
    };
}

async fn get_service_data_dl() -> Result<(), GetServiceDataDownloadError> {
    let branch = github::repos::get_branch("aws", "aws-cli", "v2")
        .await
        .unwrap();

    // コミットハッシュ取得
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

    // FIXME: コミットハッシュが保存されているものと一致しない場合のみダウンロードするように修正

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
                    return Err(GetServiceDataDownloadError::App(AppError::new(
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

    return Ok(());
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
