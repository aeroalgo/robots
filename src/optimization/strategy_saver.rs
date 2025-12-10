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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::StrategyCandidate;
    use crate::discovery::config::StrategyDiscoveryConfig;
    use crate::data_model::types::TimeFrame;
    use crate::strategy::types::StrategyParamValue;
    use std::collections::HashMap;

    fn create_test_result() -> OptimizedStrategyResult {
        let candidate = StrategyCandidate {
            indicators: vec![],
            nested_indicators: vec![],
            conditions: vec![],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![],
            config: StrategyDiscoveryConfig::default(),
        };
        let mut params = HashMap::new();
        params.insert("param1".to_string(), StrategyParamValue::Number(10.0));
        OptimizedStrategyResult {
            candidate,
            parameters: params,
            fitness: 1.5,
            backtest_report: crate::metrics::backtest::BacktestReport::new(
                vec![],
                crate::metrics::backtest::BacktestMetrics::default(),
                vec![],
            ),
        }
    }

    #[test]
    fn test_strategy_saver_new() {
        let saver = StrategySaver::new();
        assert!(true);
    }

    #[test]
    fn test_strategy_saver_default() {
        let saver = StrategySaver::default();
        assert!(true);
    }

    #[test]
    fn test_convert_to_definition() {
        let saver = StrategySaver::new();
        let result = create_test_result();
        let base_tf = TimeFrame::from_identifier("60");
        let definition = saver.convert_to_definition(&result, base_tf);
        assert!(definition.is_ok());
    }
}

