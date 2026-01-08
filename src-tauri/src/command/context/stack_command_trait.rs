use async_trait::async_trait;

#[mockall::automock]
#[async_trait]
pub trait StackCommandTrait: Send + Sync {}

pub struct StackCommand;
#[coverage(off)]
#[async_trait]
impl StackCommandTrait for StackCommand {}
