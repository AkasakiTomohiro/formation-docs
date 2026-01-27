use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    config::app_config::{AppConfig, AppConfigError},
    utils::context::file::FileSystem,
};

#[mockall::automock]
#[async_trait]
pub trait AppConfigTrait: Send + Sync {
    async fn read(&self, file_system: Arc<dyn FileSystem>) -> Result<AppConfig, AppConfigError>;
    async fn write(
        &self,
        config: AppConfig,
        file_system: Arc<dyn FileSystem>,
    ) -> Result<AppConfig, AppConfigError>;
}

pub struct AppConfigIO;
#[coverage(off)]
#[async_trait]
impl AppConfigTrait for AppConfigIO {
    async fn read(&self, file_system: Arc<dyn FileSystem>) -> Result<AppConfig, AppConfigError> {
        AppConfig::read(file_system).await
    }
    async fn write(
        &self,
        config: AppConfig,
        file_system: Arc<dyn FileSystem>,
    ) -> Result<AppConfig, AppConfigError> {
        AppConfig::write(config, file_system).await
    }
}
