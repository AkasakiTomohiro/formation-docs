use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    config::app_config::{AppConfig, AppConfigError},
    utils::context::file::FileSystem,
};

#[async_trait]
pub trait AppConfigTrait: Send + Sync {
    async fn read(&self, file_system: Arc<dyn FileSystem>) -> Result<AppConfig, AppConfigError>;
}

pub struct AppConfigIO;
#[async_trait]
impl AppConfigTrait for AppConfigIO {
    async fn read(&self, file_system: Arc<dyn FileSystem>) -> Result<AppConfig, AppConfigError> {
        AppConfig::read(file_system).await
    }
}
