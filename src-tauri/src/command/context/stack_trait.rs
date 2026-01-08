use async_trait::async_trait;

#[mockall::automock]
#[async_trait]
pub trait StackTrait: Send + Sync {}

pub struct StackCommand;
#[coverage(off)]
#[async_trait]
impl StackTrait for StackCommand {}
