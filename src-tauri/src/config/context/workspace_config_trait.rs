use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    config::workspace_config::{WorkspaceConfig, WorkspaceConfigError},
    utils::context::file::FileSystem,
};

#[mockall::automock]
#[async_trait]
pub trait WorkspaceConfigTrait: Send + Sync {
    async fn read(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<WorkspaceConfig, WorkspaceConfigError>;
    async fn write(
        &self,
        config: WorkspaceConfig,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<WorkspaceConfig, WorkspaceConfigError>;
}

pub struct WorkspaceConfigIO;
#[async_trait]
impl WorkspaceConfigTrait for WorkspaceConfigIO {
    async fn read(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<WorkspaceConfig, WorkspaceConfigError> {
        WorkspaceConfig::read(file_system, workspace_directory).await
    }
    async fn write(
        &self,
        config: WorkspaceConfig,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<WorkspaceConfig, WorkspaceConfigError> {
        WorkspaceConfig::write(config, file_system, workspace_directory).await
    }
}
