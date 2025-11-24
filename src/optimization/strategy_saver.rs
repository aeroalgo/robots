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
        let mut strategy_def = StrategyConverter::candidate_to_definition(
            &result.candidate,
            base_timeframe,
        )?;
        
        for (param_name, param_value) in &result.parameters {
            strategy_def.parameters.iter_mut()
                .find(|p| p.name == *param_name)
                .map(|p| {
                    p.default_value = match param_value {
                        crate::strategy::types::StrategyParamValue::Number(n) => {
                            crate::strategy::types::StrategyParamValue::Number(*n)
                        }
                        crate::strategy::types::StrategyParamValue::Integer(i) => {
                            crate::strategy::types::StrategyParamValue::Integer(*i)
                        }
                        crate::strategy::types::StrategyParamValue::Flag(b) => {
                            crate::strategy::types::StrategyParamValue::Flag(*b)
                        }
                        _ => param_value.clone(),
                    };
                });
        }
        
        Ok(strategy_def)
    }

}

impl Default for StrategySaver {
    fn default() -> Self {
        Self::new()
    }
}

