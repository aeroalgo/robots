use async_trait::async_trait;

use super::context::StrategyContext;
use super::types::{
    IndicatorBindingSpec, PreparedCondition, StopSignal, StrategyDecision, StrategyDefinition,
    StrategyError, StrategyId, StrategyMetadata, StrategyParameterMap, StrategyRuleSpec,
    TimeframeRequirement,
};

#[async_trait]
pub trait Strategy: Send + Sync {
    fn id(&self) -> &StrategyId;
    fn metadata(&self) -> &StrategyMetadata;
    fn parameters(&self) -> &StrategyParameterMap;
    fn indicator_bindings(&self) -> &[IndicatorBindingSpec];
    fn conditions(&self) -> &[PreparedCondition];
    fn entry_rules(&self) -> &[StrategyRuleSpec];
    fn exit_rules(&self) -> &[StrategyRuleSpec];
    fn timeframe_requirements(&self) -> &[TimeframeRequirement];
    async fn evaluate(&self, context: &StrategyContext) -> Result<StrategyDecision, StrategyError>;
    fn evaluate_stop_signals(
        &self,
        _context: &StrategyContext,
    ) -> Result<Vec<StopSignal>, StrategyError> {
        Ok(Vec::new())
    }
    fn clone_box(&self) -> Box<dyn Strategy>;
}

pub trait StrategyDescriptor {
    fn definition(&self) -> &StrategyDefinition;
}
