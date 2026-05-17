use crate::api::get_latest_version::get_latest_version::get_latest_version;
use crate::utils::context::app_context::AppContext;
use async_trait::async_trait;

#[mockall::automock]
#[async_trait]
pub trait GetAppVersionTrait: Send + Sync {
    fn get_app_version(&self) -> String;
    async fn get_latest_version(&self, ctx: &AppContext) -> Option<String>;
}

pub struct GetAppVersion;
#[coverage(off)]
#[async_trait]
impl GetAppVersionTrait for GetAppVersion {
    fn get_app_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
    async fn get_latest_version(&self, ctx: &AppContext) -> Option<String> {
        get_latest_version(ctx).await
    }
}
