use async_trait::async_trait;

use crate::{
    api::cloudformation::{self, schema::DlSchemaError},
    utils::context::app_context::AppContext,
    utils::context::file::FileSystem,
};
use std::{path::PathBuf, sync::Arc};

#[mockall::automock]
#[async_trait]
pub trait CloudformationSchemaTrait: Send + Sync {
    async fn generate_summary_service_list(
        &self,
        cxt: &AppContext,
        output_dir: PathBuf,
    ) -> Result<(), DlSchemaError>;

    fn get_resource_provider_save_dir(
        &self,
        file_system: Arc<dyn FileSystem>,
        region: &str,
    ) -> Result<PathBuf, DlSchemaError>;

    async fn dl_resource_provider(
        &self,
        ctx: &AppContext,
        region: &str,
    ) -> Result<(), DlSchemaError>;
}

pub struct CloudformationSchema;
#[coverage(off)]
#[async_trait]
impl CloudformationSchemaTrait for CloudformationSchema {
    async fn generate_summary_service_list(
        &self,
        cxt: &AppContext,
        output_dir: PathBuf,
    ) -> Result<(), DlSchemaError> {
        cloudformation::schema::generate_summary_service_list(cxt, output_dir).await
    }

    fn get_resource_provider_save_dir(
        &self,
        file_system: Arc<dyn FileSystem>,
        region: &str,
    ) -> Result<PathBuf, DlSchemaError> {
        cloudformation::schema::get_resource_provider_save_dir(file_system, region)
    }

    async fn dl_resource_provider(
        &self,
        ctx: &AppContext,
        region: &str,
    ) -> Result<(), DlSchemaError> {
        cloudformation::schema::dl_resource_provider(ctx, region).await
    }
}
