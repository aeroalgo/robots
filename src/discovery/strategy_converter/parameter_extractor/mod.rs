mod condition;
mod handler;
mod helpers;
mod indicator;

use crate::discovery::engine::StrategyCandidate;
use crate::strategy::types::StrategyParameterSpec;

pub use handler::{get_default_stop_params, get_default_take_params};

pub struct ParameterExtractor;

impl ParameterExtractor {
    pub fn extract_all(candidate: &StrategyCandidate) -> Vec<StrategyParameterSpec> {
        let estimated_capacity = candidate.indicators.len()
            + candidate.nested_indicators.len()
            + candidate.conditions.len()
            + candidate.exit_conditions.len()
            + candidate.stop_handlers.len()
            + candidate.take_handlers.len();
        let mut params = Vec::with_capacity(estimated_capacity * 2);

        params.extend(indicator::extract_indicator_parameters(
            &candidate.indicators,
            false,
        ));
        params.extend(indicator::extract_nested_indicator_parameters(
            &candidate.nested_indicators,
        ));
        params.extend(condition::extract_condition_parameters(
            &candidate.conditions,
            "entry",
        ));
        params.extend(condition::extract_condition_parameters(
            &candidate.exit_conditions,
            "exit",
        ));
        params.extend(handler::extract_stop_handler_parameters(
            &candidate.stop_handlers,
        ));
        params.extend(handler::extract_take_handler_parameters(
            &candidate.take_handlers,
        ));

        params
    }

    pub fn extract_indicator_parameters(
        indicators: &[crate::discovery::types::IndicatorInfo],
        is_nested: bool,
    ) -> Vec<StrategyParameterSpec> {
        indicator::extract_indicator_parameters(indicators, is_nested)
    }

    pub fn extract_nested_indicator_parameters(
        nested_indicators: &[crate::discovery::types::NestedIndicator],
    ) -> Vec<StrategyParameterSpec> {
        indicator::extract_nested_indicator_parameters(nested_indicators)
    }

    pub fn extract_condition_parameters(
        conditions: &[crate::discovery::types::ConditionInfo],
        prefix: &str,
    ) -> Vec<StrategyParameterSpec> {
        condition::extract_condition_parameters(conditions, prefix)
    }

    pub fn get_condition_param_range(
        param_name: &str,
    ) -> (f64, Option<f64>, Option<f64>, Option<f64>) {
        helpers::get_condition_param_range(param_name)
    }

    pub fn param_value_to_strategy_param_from_enum(
        param_type: &crate::indicators::types::ParameterType,
        default: f64,
    ) -> crate::strategy::types::StrategyParamValue {
        helpers::param_value_to_strategy_param_from_enum(param_type, default)
    }

    pub fn get_default_stop_params(
        handler_name: &str,
    ) -> std::collections::HashMap<String, crate::strategy::types::StrategyParamValue> {
        handler::get_default_stop_params(handler_name)
    }

    pub fn get_default_take_params(
        handler_name: &str,
    ) -> std::collections::HashMap<String, crate::strategy::types::StrategyParamValue> {
        handler::get_default_take_params(handler_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_model::types::TimeFrame;
    use crate::discovery::config::StrategyDiscoveryConfig;
    use crate::discovery::engine::StrategyCandidate;
    use crate::discovery::types::{
        ConditionInfo, ConditionParamInfo, IndicatorInfo,
        IndicatorParamInfo as DiscoveryIndicatorParamInfo, NestedIndicator, StopHandlerInfo,
    };
    use crate::indicators::types::ParameterType;
    use crate::strategy::types::ConditionOperator;

    fn create_test_indicator(name: &str, alias: &str) -> IndicatorInfo {
        IndicatorInfo {
            name: name.to_string(),
            alias: alias.to_string(),
            parameters: vec![DiscoveryIndicatorParamInfo {
                name: "period".to_string(),
                param_type: ParameterType::Period,
                optimizable: true,
                mutatable: true,
                global_param_name: Some("period".to_string()),
            }],
            can_use_indicator_input: false,
            input_type: "price".to_string(),
            indicator_type: "trend".to_string(),
        }
    }

    fn create_test_condition(
        id: &str,
        condition_type: &str,
        operator: ConditionOperator,
    ) -> ConditionInfo {
        ConditionInfo {
            id: id.to_string(),
            name: format!("Test {}", id),
            operator,
            condition_type: condition_type.to_string(),
            optimization_params: vec![],
            constant_value: None,
            primary_indicator_alias: "test_sma".to_string(),
            secondary_indicator_alias: None,
            primary_timeframe: None,
            secondary_timeframe: None,
            price_field: None,
        }
    }

    #[test]
    fn test_extract_indicator_parameters() {
        let indicators = vec![create_test_indicator("SMA", "sma")];
        let params = ParameterExtractor::extract_indicator_parameters(&indicators, false);

        assert!(!params.is_empty());
        let period_param = params.iter().find(|p| p.name.contains("period"));
        assert!(period_param.is_some());
    }

    #[test]
    fn test_extract_nested_indicator_parameters() {
        let nested = vec![NestedIndicator {
            indicator: create_test_indicator("SMA", "sma_nested"),
            input_indicator_alias: "ema".to_string(),
            depth: 1,
        }];
        let params = ParameterExtractor::extract_nested_indicator_parameters(&nested);

        assert!(!params.is_empty());
        let period_param = params.iter().find(|p| p.name.contains("period"));
        assert!(period_param.is_some());
    }

    #[test]
    fn test_extract_condition_parameters_trend() {
        let mut condition =
            create_test_condition("cond1", "trend_condition", ConditionOperator::RisingTrend);
        condition.optimization_params = vec![ConditionParamInfo {
            name: "period".to_string(),
            optimizable: true,
            mutatable: true,
            global_param_name: None,
        }];

        let params = ParameterExtractor::extract_condition_parameters(&[condition], "entry");

        assert!(!params.is_empty());
        let period_param = params.iter().find(|p| p.name.contains("period"));
        assert!(period_param.is_some());
        if let Some(param) = period_param {
            assert_eq!(param.min, Some(2.0));
            assert_eq!(param.max, Some(4.0));
        }
    }

    #[test]
    fn test_extract_condition_parameters_percentage() {
        let mut condition = create_test_condition(
            "cond1",
            "indicator_price",
            ConditionOperator::GreaterPercent,
        );
        condition.optimization_params = vec![ConditionParamInfo {
            name: "percent".to_string(),
            optimizable: true,
            mutatable: true,
            global_param_name: None,
        }];

        let params = ParameterExtractor::extract_condition_parameters(&[condition], "entry");

        assert!(!params.is_empty());
        let percent_param = params.iter().find(|p| p.name.contains("percent"));
        assert!(percent_param.is_some());
        if let Some(param) = percent_param {
            assert_eq!(param.min, Some(0.5));
            assert_eq!(param.max, Some(10.0));
        }
    }

    #[test]
    fn test_extract_condition_parameters_exit_prefix() {
        let mut condition =
            create_test_condition("cond1", "trend_condition", ConditionOperator::RisingTrend);
        condition.optimization_params = vec![ConditionParamInfo {
            name: "period".to_string(),
            optimizable: true,
            mutatable: true,
            global_param_name: None,
        }];

        let params = ParameterExtractor::extract_condition_parameters(&[condition], "exit");

        assert!(!params.is_empty());
        let period_param = params.iter().find(|p| p.name.contains("period"));
        assert!(period_param.is_some());
        if let Some(param) = period_param {
            assert!(param.name.contains("exit_"));
        }
    }

    #[test]
    fn test_get_condition_param_range_period() {
        let (default, min, max, step) = ParameterExtractor::get_condition_param_range("period");
        assert_eq!(min, Some(2.0));
        assert_eq!(max, Some(4.0));
        assert_eq!(step, Some(1.0));
        assert_eq!(default, 3.0);
    }

    #[test]
    fn test_get_condition_param_range_percentage() {
        let (default, min, max, step) = ParameterExtractor::get_condition_param_range("percentage");
        assert_eq!(min, Some(0.5));
        assert_eq!(max, Some(10.0));
        assert_eq!(step, Some(0.5));
        assert!(default > 0.0 && default < 10.0);
    }

    #[test]
    fn test_get_condition_param_range_unknown() {
        let (default, min, max, step) = ParameterExtractor::get_condition_param_range("unknown");
        assert_eq!(min, None);
        assert_eq!(max, None);
        assert_eq!(step, None);
        assert_eq!(default, 1.0);
    }

    #[test]
    fn test_extract_all_empty_candidate() {
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

        let params = ParameterExtractor::extract_all(&candidate);
        assert!(params.is_empty());
    }

    #[test]
    fn test_extract_all_with_all_components() {
        let candidate = StrategyCandidate {
            indicators: vec![create_test_indicator("SMA", "sma")],
            nested_indicators: vec![],
            conditions: vec![{
                let mut cond = create_test_condition(
                    "cond1",
                    "trend_condition",
                    ConditionOperator::RisingTrend,
                );
                cond.optimization_params = vec![ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    mutatable: true,
                    global_param_name: None,
                }];
                cond
            }],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let params = ParameterExtractor::extract_all(&candidate);
        assert!(!params.is_empty());
        assert!(params.iter().any(|p| p.name.contains("sma")));
        assert!(params.iter().any(|p| p.name.contains("cond1")));
    }

    #[test]
    fn test_param_value_to_strategy_param_from_enum() {
        use crate::indicators::types::ParameterType;
        use crate::strategy::types::StrategyParamValue;

        let period_val = ParameterExtractor::param_value_to_strategy_param_from_enum(
            &ParameterType::Period,
            20.0,
        );
        assert!(matches!(period_val, StrategyParamValue::Integer(20)));

        let multiplier_val = ParameterExtractor::param_value_to_strategy_param_from_enum(
            &ParameterType::Multiplier,
            2.5,
        );
        assert!(matches!(multiplier_val, StrategyParamValue::Number(2.5)));
    }
}
