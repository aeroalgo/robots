use super::context::StrategyContext;
use super::types::{
    IndicatorBindingSpec, PreparedCondition, StopHandlerSpec, StrategyDecision, StrategyError,
    StrategyId, StrategyMetadata, StrategyParameterMap, StrategyRuleSpec, TimeframeRequirement,
};
use crate::risk::AuxiliaryIndicatorSpec;

pub trait Strategy: Send + Sync {
    fn id(&self) -> &StrategyId;
    fn metadata(&self) -> &StrategyMetadata;
    fn parameters(&self) -> &StrategyParameterMap;
    fn indicator_bindings(&self) -> &[IndicatorBindingSpec];
    fn conditions(&self) -> &[PreparedCondition];
    fn entry_rules(&self) -> &[StrategyRuleSpec];
    fn exit_rules(&self) -> &[StrategyRuleSpec];
    fn timeframe_requirements(&self) -> &[TimeframeRequirement];
    fn evaluate(&self, context: &StrategyContext) -> Result<StrategyDecision, StrategyError>;
    fn clone_box(&self) -> Box<dyn Strategy>;

    fn auxiliary_indicator_specs(&self) -> &[AuxiliaryIndicatorSpec] {
        &[]
    }

    fn stop_handler_specs(&self) -> &[StopHandlerSpec] {
        &[]
    }
}
