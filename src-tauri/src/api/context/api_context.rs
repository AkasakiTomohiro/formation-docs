use std::sync::Arc;

use crate::api::context::cloudformation_schema_trait;
use crate::api::context::get_latest_version_trait;

pub struct ApiContext {
    pub cloudformation_schema: Arc<dyn cloudformation_schema_trait::CloudformationSchemaTrait>,
    pub get_latest_version: Arc<dyn get_latest_version_trait::GetLatestVersionTrait>,
}

#[coverage(off)]
impl ApiContext {
    pub fn new() -> Self {
        Self {
            cloudformation_schema: Arc::new(cloudformation_schema_trait::CloudformationSchema {}),
            get_latest_version: Arc::new(get_latest_version_trait::GetLatestVersion {}),
        }
    }
}
