use crate::api::context::api_context::ApiContext;
use crate::command::cloudformation_schema::CloudFormationSchemaError;
use crate::utils::context::app_context::AppContext;
use async_trait::async_trait;

#[mockall::automock]
#[async_trait]
pub trait CloudFormationSchemaTrait: Send + Sync {
    async fn get_cloudformation_schema(
        &self,
        app_context: &AppContext,
        api_context: &ApiContext,
        service_name: &str,
        resource_name: &str,
    ) -> Result<String, CloudFormationSchemaError>;
}

pub struct CloudFormationSchema;
#[coverage(off)]
#[async_trait]
impl CloudFormationSchemaTrait for CloudFormationSchema {
    async fn get_cloudformation_schema(
        &self,
        app_context: &AppContext,
        api_context: &ApiContext,
        service_name: &str,
        resource_name: &str,
    ) -> Result<String, CloudFormationSchemaError> {
        crate::command::cloudformation_schema::get_cloudformation_schema(
            app_context,
            api_context,
            service_name,
            resource_name,
        )
        .await
    }
}
