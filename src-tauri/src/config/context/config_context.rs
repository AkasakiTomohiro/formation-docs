use std::sync::Arc;

use super::app_config_trait;
use super::stack_meta_config_trait;
use super::workspace_config_trait;

pub struct ConfigContext {
    pub app_config_io: Arc<dyn app_config_trait::AppConfigTrait>,
    pub stack_meta_config_io: Arc<dyn stack_meta_config_trait::StackMetaConfigTrait>,
    pub workspace_config_io: Arc<dyn workspace_config_trait::WorkspaceConfigTrait>,
}

impl ConfigContext {
    pub fn new() -> Self {
        Self {
            app_config_io: Arc::new(app_config_trait::AppConfigIO {}),
            stack_meta_config_io: Arc::new(stack_meta_config_trait::StackMetaConfigIO {}),
            workspace_config_io: Arc::new(workspace_config_trait::WorkspaceConfigIO {}),
        }
    }
}
