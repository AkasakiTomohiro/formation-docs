use async_trait::async_trait;
use reqwest::{Response, Result};

#[mockall::automock]
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: String) -> Result<Response>;
}

pub struct RealHttpClient;
#[coverage(off)]
#[async_trait]
impl HttpClient for RealHttpClient {
    async fn get(&self, url: String) -> Result<Response> {
        return reqwest::get(url).await;
    }
}
