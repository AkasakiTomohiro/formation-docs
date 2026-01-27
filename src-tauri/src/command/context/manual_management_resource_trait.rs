use crate::command::manual_management_resource::ManualManagementMeta;
use crate::command::manual_management_resource::ManualManagementResourceError;
use crate::command::manual_management_resource::ManualManagementResources;
use crate::utils::context::app_context::AppContext;
use async_trait::async_trait;

#[mockall::automock]
#[async_trait]
pub trait ManualManagementResourceTrait: Send + Sync {
    async fn get_manual_management_resources(
        &self,
        state: &AppContext,
        workspace_directory: &str,
    ) -> Result<ManualManagementResources, ManualManagementResourceError>;
    async fn save_manual_management_resources(
        &self,
        state: &AppContext,
        workspace_directory: &str,
        manual_management_resources: &ManualManagementResources,
    ) -> Result<(), ManualManagementResourceError>;
    async fn load_manual_resource_meta(
        &self,
        state: &AppContext,
        workspace_directory: &str,
    ) -> Result<ManualManagementMeta, ManualManagementResourceError>;
}

pub struct ManualManagementResource;
#[coverage(off)]
#[async_trait]
impl ManualManagementResourceTrait for ManualManagementResource {
    async fn get_manual_management_resources(
        &self,
        state: &AppContext,
        workspace_directory: &str,
    ) -> Result<ManualManagementResources, ManualManagementResourceError> {
        crate::command::manual_management_resource::get_manual_management_resources(
            state,
            workspace_directory,
        )
        .await
    }

    async fn save_manual_management_resources(
        &self,
        state: &AppContext,
        workspace_directory: &str,
        manual_management_resources: &ManualManagementResources,
    ) -> Result<(), ManualManagementResourceError> {
        crate::command::manual_management_resource::save_manual_management_resources(
            state,
            workspace_directory,
            manual_management_resources,
        )
        .await
    }

    async fn load_manual_resource_meta(
        &self,
        state: &AppContext,
        workspace_directory: &str,
    ) -> Result<ManualManagementMeta, ManualManagementResourceError> {
        crate::command::manual_management_resource::load_manual_resource_meta(
            state,
            workspace_directory,
        )
        .await
    }
}
