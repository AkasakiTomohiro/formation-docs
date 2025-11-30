use std::sync::Arc;

use super::app_config_trait;

pub struct ConfigContext {
    pub app_config_io: Arc<dyn app_config_trait::AppConfigTrait>,
}

impl ConfigContext {
    pub fn new() -> Self {
        Self {
            app_config_io: Arc::new(app_config_trait::AppConfigIO {}),
        }
    }
}
