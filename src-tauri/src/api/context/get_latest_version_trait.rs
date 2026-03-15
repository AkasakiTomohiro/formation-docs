use crate::api::get_latest_version::get_latest_version::get_latest_version;
use crate::utils::context::app_context::AppContext;
use async_trait::async_trait;

#[mockall::automock]
#[async_trait]
pub trait GetLatestVersionTrait: Send + Sync {
    async fn get_latest_version(&self, ctx: &AppContext) -> Option<String>;
}

pub struct GetLatestVersion;
#[coverage(off)]
#[async_trait]
impl GetLatestVersionTrait for GetLatestVersion {
    async fn get_latest_version(&self, ctx: &AppContext) -> Option<String> {
        get_latest_version(ctx).await
    }
}
