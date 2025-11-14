use crate::condition::types::{
    ConditionConfig, ConditionError, ConditionInputData, ConditionResult, ConditionResultData,
};
use async_trait::async_trait;

#[async_trait]
pub trait Condition: Send + Sync {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    fn config(&self) -> &ConditionConfig;

    fn min_data_points(&self) -> usize;

    async fn check(&self, input: ConditionInputData<'_>) -> ConditionResult<ConditionResultData>;

    fn validate(&self, input: &ConditionInputData<'_>) -> Result<(), ConditionError>;

    fn clone_box(&self) -> Box<dyn Condition + Send + Sync>;
}
