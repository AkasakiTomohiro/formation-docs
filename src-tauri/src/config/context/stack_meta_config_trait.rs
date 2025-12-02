use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    config::stack_meta_config::{StackMetaConfig, StackMetaConfigError},
    utils::context::file::FileSystem,
};

#[async_trait]
pub trait StackMetaConfigTrait: Send + Sync {
    async fn read(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
        stack_name: &str,
    ) -> Result<StackMetaConfig, StackMetaConfigError>;

    async fn write(
        &self,
        config: StackMetaConfig,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
        stack_name: &str,
    ) -> Result<StackMetaConfig, StackMetaConfigError>;
}

pub struct StackMetaConfigIO;
#[async_trait]
impl StackMetaConfigTrait for StackMetaConfigIO {
    async fn read(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
        stack_name: &str,
    ) -> Result<StackMetaConfig, StackMetaConfigError> {
        StackMetaConfig::read(file_system, workspace_directory, stack_name).await
    }
    async fn write(
        &self,
        config: StackMetaConfig,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
        stack_name: &str,
    ) -> Result<StackMetaConfig, StackMetaConfigError> {
        StackMetaConfig::write(config, file_system, workspace_directory, stack_name).await
    }
}
