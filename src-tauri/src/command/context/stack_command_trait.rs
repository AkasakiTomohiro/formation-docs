use async_trait::async_trait;

use crate::{
    command::stack::{get_stack_outputs, load_stack_from_info, Stack, StackError, StackOutput},
    config::context::config_context::ConfigContext,
    utils::context::app_context::AppContext,
};

#[mockall::automock]
#[async_trait]
pub trait StackCommandTrait: Send + Sync {
    async fn load_stack_from_info(
        &self,
        app_context: &AppContext,
        config_context: &ConfigContext,
        workspace_directory: &str,
        stack_id: &str,
        stack_file_name: &str,
    ) -> Result<Stack, StackError>;

    async fn get_stack_outputs(
        &self,
        app_context: &AppContext,
        config_context: &ConfigContext,
        workspace_directory: &str,
        stack_id: &str,
    ) -> Result<Vec<StackOutput>, StackError> {
        get_stack_outputs(app_context, config_context, workspace_directory, stack_id).await
    }
}

pub struct StackCommand;
#[coverage(off)]
#[async_trait]
impl StackCommandTrait for StackCommand {
    async fn load_stack_from_info(
        &self,
        app_context: &AppContext,
        config_context: &ConfigContext,
        workspace_directory: &str,
        stack_id: &str,
        stack_file_name: &str,
    ) -> Result<Stack, StackError> {
        load_stack_from_info(
            app_context,
            config_context,
            workspace_directory,
            stack_id,
            stack_file_name,
        )
        .await
    }

    async fn get_stack_outputs(
        &self,
        app_context: &AppContext,
        config_context: &ConfigContext,
        workspace_directory: &str,
        stack_id: &str,
    ) -> Result<Vec<StackOutput>, StackError> {
        get_stack_outputs(app_context, config_context, workspace_directory, stack_id).await
    }
}
