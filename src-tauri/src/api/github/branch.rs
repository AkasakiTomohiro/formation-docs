use super::super::super::utils::AppError;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BranchError {
    #[error("app error: {0}")]
    App(#[from] AppError),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("serde error: {0}")]
    Json(#[from] serde_json::Error),
}

pub async fn get_branch(owner: &str, repo: &str, branch: &str) -> Result<Value, BranchError> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        owner, repo, branch
    );

    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "formation-docs".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());

    let response = client.get(url).headers(headers).send().await?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(BranchError::App(AppError::new("Failed to get branch")));
    }
    let response = response.text().await?;
    return Ok(serde_json::from_str(&response)?);
}
