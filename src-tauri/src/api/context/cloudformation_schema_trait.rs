use async_trait::async_trait;

use crate::{
    api::cloudformation::{self, schema::DlSchemaError},
    utils::context::app_context::AppContext,
};

#[mockall::automock]
#[async_trait]
pub trait CloudformationSchemaTrait: Send + Sync {
    async fn dl_resource_provider(
        &self,
        ctx: &AppContext,
        region: &str,
    ) -> Result<(), DlSchemaError>;
}

pub struct CloudformationSchema;
#[async_trait]
impl CloudformationSchemaTrait for CloudformationSchema {
    async fn dl_resource_provider(
        &self,
        ctx: &AppContext,
        region: &str,
    ) -> Result<(), DlSchemaError> {
        cloudformation::schema::dl_resource_provider(ctx, region).await
    }
}
