use async_trait::async_trait;

#[mockall::automock]
#[async_trait]
pub trait AppConfigCommandTrait: Send + Sync {}

pub struct AppConfigCommand;
#[coverage(off)]
#[async_trait]
impl AppConfigCommandTrait for AppConfigCommand {}
