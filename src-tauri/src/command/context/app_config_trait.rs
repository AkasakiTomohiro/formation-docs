use async_trait::async_trait;

#[mockall::automock]
#[async_trait]
pub trait AppConfigTrait: Send + Sync {}

pub struct AppConfigCommand;
#[coverage(off)]
#[async_trait]
impl AppConfigTrait for AppConfigCommand {}
