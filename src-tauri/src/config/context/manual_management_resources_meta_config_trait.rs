use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    config::manual_management_resources_meta_config::{
        ManualManagementResourcesMetaConfig, ManualManagementResourcesMetaConfigError,
    },
    utils::context::file::FileSystem,
};

#[mockall::automock]
#[async_trait]
pub trait ManualManagementResourcesMetaConfigTrait: Send + Sync {
    async fn read(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<ManualManagementResourcesMetaConfig, ManualManagementResourcesMetaConfigError>;

    async fn write(
        &self,
        config: ManualManagementResourcesMetaConfig,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<ManualManagementResourcesMetaConfig, ManualManagementResourcesMetaConfigError>;
}

pub struct ManualManagementResourcesMetaConfigIO;
#[coverage(off)]
#[async_trait]
impl ManualManagementResourcesMetaConfigTrait for ManualManagementResourcesMetaConfigIO {
    async fn read(
        &self,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<ManualManagementResourcesMetaConfig, ManualManagementResourcesMetaConfigError> {
        ManualManagementResourcesMetaConfig::read(file_system, workspace_directory).await
    }
    async fn write(
        &self,
        config: ManualManagementResourcesMetaConfig,
        file_system: Arc<dyn FileSystem>,
        workspace_directory: &str,
    ) -> Result<ManualManagementResourcesMetaConfig, ManualManagementResourcesMetaConfigError> {
        ManualManagementResourcesMetaConfig::write(config, file_system, workspace_directory).await
    }
}
