use std::sync::Arc;

use crate::command::context::{app_config_command_trait, stack_command_trait};

pub struct CommandContext {
    pub app_config: Arc<dyn app_config_command_trait::AppConfigCommandTrait>,
    pub stack: Arc<dyn stack_command_trait::StackCommandTrait>,
}

#[coverage(off)]
impl CommandContext {
    pub fn new() -> Self {
        Self {
            app_config: Arc::new(app_config_command_trait::AppConfigCommand {}),
            stack: Arc::new(stack_command_trait::StackCommand {}),
        }
    }
}
