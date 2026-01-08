use std::sync::Arc;

use crate::command::context::{app_config_trait, stack_trait};

pub struct CommandContext {
    pub app_config: Arc<dyn app_config_trait::AppConfigTrait>,
    pub stack: Arc<dyn stack_trait::StackTrait>,
}

#[coverage(off)]
impl CommandContext {
    pub fn new() -> Self {
        Self {
            app_config: Arc::new(app_config_trait::AppConfigCommand {}),
            stack: Arc::new(stack_trait::StackCommand {}),
        }
    }
}
