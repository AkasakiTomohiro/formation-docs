use std::sync::Arc;

use super::app_config_trait;
use super::manual_management_resources_meta_config_trait;
use super::stack_meta_config_trait;
use super::workspace_config_trait;

pub struct ConfigContext {
    pub app_config_io: Arc<dyn app_config_trait::AppConfigTrait>,
    pub stack_meta_config_io: Arc<dyn stack_meta_config_trait::StackMetaConfigTrait>,
    pub workspace_config_io: Arc<dyn workspace_config_trait::WorkspaceConfigTrait>,
    pub manual_management_resources_meta_config_io: Arc<
        dyn manual_management_resources_meta_config_trait::ManualManagementResourcesMetaConfigTrait,
    >,
}

#[coverage(off)]
impl ConfigContext {
    pub fn new() -> Self {
        Self {
            app_config_io: Arc::new(app_config_trait::AppConfigIO {}),
            stack_meta_config_io: Arc::new(stack_meta_config_trait::StackMetaConfigIO {}),
            workspace_config_io: Arc::new(workspace_config_trait::WorkspaceConfigIO {}),
            manual_management_resources_meta_config_io: Arc::new(
                manual_management_resources_meta_config_trait::ManualManagementResourcesMetaConfigIO {},
            ),
        }
    }
}
