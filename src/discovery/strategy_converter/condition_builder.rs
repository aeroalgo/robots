use std::collections::{HashMap, HashSet};

use crate::data_model::types::TimeFrame;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo};
use crate::strategy::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    IndicatorBindingSpec, PriceField,
};

use super::condition_converters::ConditionConverterFactory;
use super::helpers::ConverterHelpers;
use super::main::StrategyConversionError;

pub struct ConditionBuilder;

impl ConditionBuilder {
    pub fn create_bindings(
        conditions: &[ConditionInfo],
        candidate: &StrategyCandidate,
        indicator_bindings: &[IndicatorBindingSpec],
        base_timeframe: TimeFrame,
        prefix: &str,
    ) -> Result<Vec<ConditionBindingSpec>, StrategyConversionError> {
        let mut bindings = Vec::with_capacity(conditions.len());

        for condition in conditions {
            let input = Self::create_condition_input(condition, candidate, indicator_bindings)?;
            let declarative = ConditionDeclarativeSpec {
                operator: condition.operator.clone(),
                operands: vec![],
                description: Some(condition.name.clone()),
            };

            let mut parameters =
                Self::extract_condition_parameters(&condition.operator, condition)?;

            let condition_id = if prefix == "exit" {
                format!("exit_{}", condition.id)
            } else {
                condition.id.clone()
            };

            let condition_name = if prefix == "exit" {
                format!("Exit: {}", condition.name)
            } else {
                condition.name.clone()
            };

            let mut tags = Vec::with_capacity(2);
            tags.push(condition.condition_type.clone());
            if prefix == "exit" {
                tags.push("exit".into());
            }

            bindings.push(ConditionBindingSpec {
                id: condition_id,
                name: condition_name,
                timeframe: base_timeframe.clone(),
                declarative,
                parameters,
                input,
                weight: 1.0,
                tags,
                user_formula: None,
            });
        }

        Ok(bindings)
    }

    fn extract_condition_parameters(
        operator: &ConditionOperator,
        condition: &ConditionInfo,
    ) -> Result<HashMap<String, f32>, StrategyConversionError> {
        let mut parameters = HashMap::new();

        if matches!(
            operator,
            ConditionOperator::LowerPercent | ConditionOperator::GreaterPercent
        ) {
            let percent_param = condition
                .optimization_params
                .iter()
                .find(|p| p.name == "percent" || p.name == "percentage");

            if let Some(_percent_param) = percent_param {
                let range = crate::condition::parameters::ConditionParameterPresets::percentage();
                let percent_value = ((range.min + range.max) / 2.0) as f32;
                parameters.insert("percent".into(), percent_value);
            } else {
                return Err(StrategyConversionError::InvalidConditionFormat {
                    condition_id: condition.id.clone(),
                    reason: format!(
                        "Condition {} requires 'percent' parameter in optimization_params",
                        condition.name
                    ),
                });
            }
        } else if matches!(
            operator,
            ConditionOperator::RisingTrend | ConditionOperator::FallingTrend
        ) {
            let period_param = condition
                .optimization_params
                .iter()
                .find(|p| p.name == "period");

            if let Some(_period_param) = period_param {
                let range = crate::condition::parameters::ConditionParameterPresets::trend_period();
                let period_value = ((range.min + range.max) / 2.0) as f32;
                parameters.insert("period".into(), period_value);
            } else {
                return Err(StrategyConversionError::InvalidConditionFormat {
                    condition_id: condition.id.clone(),
                    reason: format!(
                        "Condition {} requires 'period' parameter in optimization_params",
                        condition.name
                    ),
                });
            }
        }

        Ok(parameters)
    }

    pub fn create_condition_input(
        condition: &ConditionInfo,
        _candidate: &StrategyCandidate,
        indicator_bindings: &[IndicatorBindingSpec],
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        let alias_to_timeframes =
            ConverterHelpers::build_alias_to_timeframes_map(indicator_bindings);

        let converter = ConditionConverterFactory::get_converter(&condition.condition_type)?;
        converter.convert_to_input(condition, &alias_to_timeframes)
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
        IndicatorParamInfo as DiscoveryIndicatorParamInfo,
    };
    use crate::indicators::types::ParameterType;
    use crate::strategy::types::{ConditionOperator, IndicatorBindingSpec, IndicatorSourceSpec};

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
            primary_indicator_alias: "sma".to_string(),
            secondary_indicator_alias: None,
            primary_timeframe: None,
            secondary_timeframe: None,
            price_field: None,
        }
    }

    fn create_test_indicator_binding(alias: &str) -> IndicatorBindingSpec {
        IndicatorBindingSpec {
            alias: alias.to_string(),
            timeframe: TimeFrame::Minutes(60),
            source: IndicatorSourceSpec::Registry {
                name: "SMA".to_string(),
                parameters: std::collections::HashMap::new(),
            },
            tags: vec![],
        }
    }

    fn create_test_candidate() -> StrategyCandidate {
        StrategyCandidate {
            indicators: vec![create_test_indicator("SMA", "sma")],
            nested_indicators: vec![],
            conditions: vec![],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        }
    }

    #[test]
    fn test_create_bindings_indicator_price() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let mut condition =
            create_test_condition("cond1", "indicator_price", ConditionOperator::Above);
        condition.price_field = Some("Close".to_string());

        let result = ConditionBuilder::create_bindings(
            &[condition],
            &candidate,
            &indicator_bindings,
            TimeFrame::Minutes(60),
            "entry",
        );

        assert!(result.is_ok());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].id, "cond1");
    }

    #[test]
    fn test_create_bindings_indicator_indicator() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![
            create_test_indicator_binding("sma"),
            create_test_indicator_binding("ema"),
        ];
        let mut condition =
            create_test_condition("cond1", "indicator_indicator", ConditionOperator::Above);
        condition.secondary_indicator_alias = Some("ema".to_string());

        let result = ConditionBuilder::create_bindings(
            &[condition],
            &candidate,
            &indicator_bindings,
            TimeFrame::Minutes(60),
            "entry",
        );

        assert!(result.is_ok());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_create_bindings_trend_condition() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let mut condition =
            create_test_condition("cond1", "trend_condition", ConditionOperator::RisingTrend);
        condition.optimization_params = vec![ConditionParamInfo {
            name: "period".to_string(),
            optimizable: true,
            mutatable: true,
            global_param_name: None,
        }];

        let result = ConditionBuilder::create_bindings(
            &[condition],
            &candidate,
            &indicator_bindings,
            TimeFrame::Minutes(60),
            "entry",
        );

        assert!(result.is_ok());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert!(bindings[0].parameters.contains_key("period"));
    }

    #[test]
    fn test_create_bindings_indicator_constant() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let mut condition =
            create_test_condition("cond1", "indicator_constant", ConditionOperator::Above);
        condition.constant_value = Some(70.0);

        let result = ConditionBuilder::create_bindings(
            &[condition],
            &candidate,
            &indicator_bindings,
            TimeFrame::Minutes(60),
            "entry",
        );

        assert!(result.is_ok());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
    }

    #[test]
    fn test_create_bindings_exit_prefix() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let mut condition =
            create_test_condition("cond1", "indicator_price", ConditionOperator::Below);
        condition.price_field = Some("Close".to_string());

        let result = ConditionBuilder::create_bindings(
            &[condition],
            &candidate,
            &indicator_bindings,
            TimeFrame::Minutes(60),
            "exit",
        );

        assert!(result.is_ok());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].id, "exit_cond1");
        assert!(bindings[0].name.contains("Exit:"));
        assert!(bindings[0].tags.contains(&"exit".to_string()));
    }

    #[test]
    fn test_create_bindings_empty_conditions() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];

        let result = ConditionBuilder::create_bindings(
            &[],
            &candidate,
            &indicator_bindings,
            TimeFrame::Minutes(60),
            "entry",
        );

        assert!(result.is_ok());
        let bindings = result.unwrap();
        assert!(bindings.is_empty());
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

        let result =
            ConditionBuilder::extract_condition_parameters(&condition.operator, &condition);

        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.contains_key("percent"));
    }

    #[test]
    fn test_extract_condition_parameters_percentage_missing() {
        let condition = create_test_condition(
            "cond1",
            "indicator_price",
            ConditionOperator::GreaterPercent,
        );

        let result =
            ConditionBuilder::extract_condition_parameters(&condition.operator, &condition);

        assert!(result.is_err());
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

        let result =
            ConditionBuilder::extract_condition_parameters(&condition.operator, &condition);

        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.contains_key("period"));
    }

    #[test]
    fn test_extract_condition_parameters_trend_missing() {
        let condition =
            create_test_condition("cond1", "trend_condition", ConditionOperator::RisingTrend);

        let result =
            ConditionBuilder::extract_condition_parameters(&condition.operator, &condition);

        assert!(result.is_err());
    }

    #[test]
    fn test_create_condition_input_indicator_price() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let mut condition =
            create_test_condition("cond1", "indicator_price", ConditionOperator::Above);
        condition.price_field = Some("Close".to_string());

        let result =
            ConditionBuilder::create_condition_input(&condition, &candidate, &indicator_bindings);

        assert!(result.is_ok());
        let input = result.unwrap();
        assert!(matches!(input, ConditionInputSpec::Dual { .. }));
    }

    #[test]
    fn test_create_condition_input_indicator_indicator() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![
            create_test_indicator_binding("sma"),
            create_test_indicator_binding("ema"),
        ];
        let mut condition =
            create_test_condition("cond1", "indicator_indicator", ConditionOperator::Above);
        condition.secondary_indicator_alias = Some("ema".to_string());

        let result =
            ConditionBuilder::create_condition_input(&condition, &candidate, &indicator_bindings);

        assert!(result.is_ok());
        let input = result.unwrap();
        assert!(matches!(input, ConditionInputSpec::Dual { .. }));
    }

    #[test]
    fn test_create_condition_input_indicator_indicator_missing_secondary() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let condition =
            create_test_condition("cond1", "indicator_indicator", ConditionOperator::Above);

        let result =
            ConditionBuilder::create_condition_input(&condition, &candidate, &indicator_bindings);

        assert!(result.is_err());
    }

    #[test]
    fn test_create_condition_input_trend_condition() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let condition =
            create_test_condition("cond1", "trend_condition", ConditionOperator::RisingTrend);

        let result =
            ConditionBuilder::create_condition_input(&condition, &candidate, &indicator_bindings);

        assert!(result.is_ok());
        let input = result.unwrap();
        assert!(matches!(input, ConditionInputSpec::Single { .. }));
    }

    #[test]
    fn test_create_condition_input_indicator_constant() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let mut condition =
            create_test_condition("cond1", "indicator_constant", ConditionOperator::Above);
        condition.constant_value = Some(70.0);

        let result =
            ConditionBuilder::create_condition_input(&condition, &candidate, &indicator_bindings);

        assert!(result.is_ok());
        let input = result.unwrap();
        assert!(matches!(input, ConditionInputSpec::Dual { .. }));
    }

    #[test]
    fn test_create_condition_input_unsupported_type() {
        let candidate = create_test_candidate();
        let indicator_bindings = vec![create_test_indicator_binding("sma")];
        let mut condition =
            create_test_condition("cond1", "unknown_type", ConditionOperator::Above);
        condition.condition_type = "unknown_type".to_string();

        let result =
            ConditionBuilder::create_condition_input(&condition, &candidate, &indicator_bindings);

        assert!(result.is_err());
    }
}
