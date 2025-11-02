use async_trait::async_trait;
use reqwest::{Response, Result};

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: String) -> Result<Response>;
}

pub struct RealHttpClient;
#[async_trait]
impl HttpClient for RealHttpClient {
    async fn get(&self, url: String) -> Result<Response> {
        return reqwest::get(url).await;
    }
}
