use std::sync::Arc;

use crate::api::context::cloudformation_schema_trait;

pub struct ApiContext {
    pub cloudformation_schema: Arc<dyn cloudformation_schema_trait::CloudformationSchemaTrait>,
}

#[coverage(off)]
impl ApiContext {
    pub fn new() -> Self {
        Self {
            cloudformation_schema: Arc::new(cloudformation_schema_trait::CloudformationSchema {}),
        }
    }
}
