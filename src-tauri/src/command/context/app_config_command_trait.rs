use async_trait::async_trait;

use crate::command;
use crate::command::app_config::AppConfigCommandError;
use crate::config::context::config_context::ConfigContext;
use crate::utils::context::app_context::AppContext;

#[mockall::automock]
#[async_trait]
pub trait AppConfigCommandTrait: Send + Sync {
    async fn add_workspace_to_app_config(
        &self,
        app_context: &AppContext,
        config_context: &ConfigContext,
        workspace_directory: &str,
    ) -> Result<String, AppConfigCommandError>;
}

pub struct AppConfigCommand;
#[coverage(off)]
#[async_trait]
impl AppConfigCommandTrait for AppConfigCommand {
    async fn add_workspace_to_app_config(
        &self,
        app_context: &AppContext,
        config_context: &ConfigContext,
        workspace_directory: &str,
    ) -> Result<String, AppConfigCommandError> {
        command::app_config::add_workspace_to_app_config(
            app_context,
            config_context,
            workspace_directory,
        )
        .await
    }
}
