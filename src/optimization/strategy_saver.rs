use crate::optimization::per_structure_optimizer::OptimizedStrategyResult;
use crate::strategy::types::StrategyDefinition;
use crate::discovery::strategy_converter::StrategyConverter;

pub struct StrategySaver;

impl StrategySaver {
    pub fn new() -> Self {
        Self
    }

    pub fn convert_to_definition(
        &self,
        result: &OptimizedStrategyResult,
        base_timeframe: crate::data_model::types::TimeFrame,
    ) -> Result<StrategyDefinition, anyhow::Error> {
        let strategy_def = StrategyConverter::candidate_to_definition(
            &result.candidate,
            base_timeframe,
        )?;
        
        Ok(strategy_def)
    }

}

impl Default for StrategySaver {
    fn default() -> Self {
        Self::new()
    }
}

