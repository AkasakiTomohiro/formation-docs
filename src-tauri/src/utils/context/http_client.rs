use async_trait::async_trait;
use bytes::Bytes;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("reqwest error: {0}")]
    Reqwest(String),
}

#[coverage(off)]
impl From<reqwest::Error> for HttpClientError {
    fn from(err: reqwest::Error) -> Self {
        HttpClientError::Reqwest(err.to_string())
    }
}

#[mockall::automock]
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: String) -> Result<Bytes, HttpClientError>;
}

pub struct RealHttpClient;
#[coverage(off)]
#[async_trait]
impl HttpClient for RealHttpClient {
    async fn get(&self, url: String) -> Result<Bytes, HttpClientError> {
        let response = reqwest::get(&url).await?;
        return Ok(response.bytes().await?);
    }
}
