use std::sync::Arc;

use crate::command::context::{
    app_config_command_trait, cloudformation_schema_trait, manual_management_resource_trait,
    stack_command_trait,
};

pub struct CommandContext {
    pub app_config: Arc<dyn app_config_command_trait::AppConfigCommandTrait>,
    pub stack: Arc<dyn stack_command_trait::StackCommandTrait>,
    pub cloudformation_schema: Arc<dyn cloudformation_schema_trait::CloudFormationSchemaTrait>,
    pub manual_management_resource:
        Arc<dyn manual_management_resource_trait::ManualManagementResourceTrait>,
}

#[coverage(off)]
impl CommandContext {
    pub fn new() -> Self {
        Self {
            app_config: Arc::new(app_config_command_trait::AppConfigCommand {}),
            stack: Arc::new(stack_command_trait::StackCommand {}),
            cloudformation_schema: Arc::new(cloudformation_schema_trait::CloudFormationSchema {}),
            manual_management_resource: Arc::new(
                manual_management_resource_trait::ManualManagementResource {},
            ),
        }
    }
}
