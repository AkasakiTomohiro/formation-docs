use std::sync::Arc;

use crate::api::context::cloudformation_schema_trait;
use crate::api::context::get_app_version_trait;

pub struct ApiContext {
    pub cloudformation_schema: Arc<dyn cloudformation_schema_trait::CloudformationSchemaTrait>,
    pub get_app_version: Arc<dyn get_app_version_trait::GetAppVersionTrait>,
}

#[coverage(off)]
impl ApiContext {
    pub fn new() -> Self {
        Self {
            cloudformation_schema: Arc::new(cloudformation_schema_trait::CloudformationSchema {}),
            get_app_version: Arc::new(get_app_version_trait::GetAppVersion {}),
        }
    }
}
