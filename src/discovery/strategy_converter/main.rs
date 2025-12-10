use chrono::Utc;
use std::collections::{BTreeMap, HashMap};

use crate::data_model::types::TimeFrame;
use crate::discovery::config::StrategyDiscoveryConfig;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo};
use crate::optimization::condition_id::ConditionId;

use super::condition_builder::ConditionBuilder;
use super::handler_builder::HandlerBuilder;
use super::indicator_builder::IndicatorBuilder;
use super::metadata_builder::MetadataBuilder;
use super::parameter_extractor::ParameterExtractor;
use super::rule_builder::RuleBuilder;
use crate::strategy::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    DataSeriesSource, IndicatorBindingSpec, IndicatorSourceSpec, PositionDirection, RuleLogic,
    StopHandlerSpec, StrategyCategory, StrategyDefinition, StrategyMetadata, StrategyParamValue,
    StrategyParameterMap, StrategyParameterSpec, StrategyRuleSpec, StrategySignalType,
    TakeHandlerSpec,
};

pub struct StrategyConverter;

impl StrategyConverter {
    pub fn candidate_to_definition(
        candidate: &StrategyCandidate,
        base_timeframe: TimeFrame,
    ) -> Result<StrategyDefinition, StrategyConversionError> {
        let metadata = MetadataBuilder::create_metadata(candidate);
        let parameters = ParameterExtractor::extract_all(candidate);
        let defaults = Self::extract_defaults(candidate);
        let base_tf = base_timeframe.clone();

        let indicator_bindings =
            IndicatorBuilder::create_indicator_bindings(candidate, base_tf.clone())?;
        let condition_bindings = ConditionBuilder::create_bindings(
            &candidate.conditions,
            candidate,
            &indicator_bindings,
            base_tf.clone(),
            "entry",
        )?;
        let mut stop_handlers = HandlerBuilder::create_stop_handlers(candidate, base_tf.clone())?;
        let mut take_handlers = HandlerBuilder::create_take_handlers(candidate, base_tf.clone())?;

        let exit_condition_bindings = ConditionBuilder::create_bindings(
            &candidate.exit_conditions,
            candidate,
            &indicator_bindings,
            base_tf,
            "exit",
        )?;

        let entry_rules = RuleBuilder::create_entry_rules(candidate, &condition_bindings)?;
        let exit_rules = RuleBuilder::create_exit_rules(candidate, &exit_condition_bindings)?;

        let entry_rule_ids: Vec<String> = entry_rules.iter().map(|r| r.id.clone()).collect();
        HandlerBuilder::set_target_entry_ids(
            &mut stop_handlers,
            &mut take_handlers,
            &entry_rule_ids,
        );

        let mut all_condition_bindings = condition_bindings;
        all_condition_bindings.extend(exit_condition_bindings);

        Ok(StrategyDefinition::new(
            metadata,
            parameters,
            indicator_bindings,
            vec![],
            all_condition_bindings,
            entry_rules,
            exit_rules,
            stop_handlers,
            take_handlers,
            defaults,
            BTreeMap::new(),
        ))
    }

    fn extract_defaults(_candidate: &StrategyCandidate) -> StrategyParameterMap {
        HashMap::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StrategyConversionError {
    #[error("Invalid condition format: {condition_id} - {reason}")]
    InvalidConditionFormat {
        condition_id: String,
        reason: String,
    },
    #[error("Unsupported condition type: {condition_type}")]
    UnsupportedConditionType { condition_type: String },
    #[error("Missing parameter range for indicator {indicator_name}, parameter {parameter_name} (type: {parameter_type})")]
    MissingParameterRange {
        indicator_name: String,
        parameter_name: String,
        parameter_type: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::condition::parameters::ConditionParameterPresets;
    use crate::discovery::engine::StrategyCandidate;
    use crate::discovery::types::{
        ConditionInfo, ConditionParamInfo, IndicatorInfo, NestedIndicator,
    };
    use crate::strategy::types::ConditionOperator;

    #[test]
    fn test_condition_parameter_ranges_match_presets() {
        let trend_range = ConditionParameterPresets::trend_period();
        assert_eq!(trend_range.min, 2.0, "trend_period min должен быть 2.0");
        assert_eq!(trend_range.max, 4.0, "trend_period max должен быть 4.0");
        assert_eq!(trend_range.step, 1.0, "trend_period step должен быть 1.0");

        let percentage_range = ConditionParameterPresets::percentage();
        assert_eq!(percentage_range.min, 0.5, "percentage min должен быть 0.5");
        assert_eq!(
            percentage_range.max, 10.0,
            "percentage max должен быть 10.0"
        );
        assert_eq!(
            percentage_range.step, 0.5,
            "percentage step должен быть 0.5"
        );
    }

    #[test]
    fn test_extract_parameters_uses_correct_ranges() {
        let candidate = StrategyCandidate {
            indicators: vec![],
            nested_indicators: vec![],
            conditions: vec![ConditionInfo {
                id: "test_condition_1".to_string(),
                name: "Test RisingTrend".to_string(),
                operator: ConditionOperator::RisingTrend,
                primary_indicator_alias: "test_sma".to_string(),
                secondary_indicator_alias: None,
                condition_type: "trend_condition".to_string(),
                primary_timeframe: Some(TimeFrame::Minutes(60)),
                secondary_timeframe: None,
                price_field: None,
                constant_value: Some(3.0),
                optimization_params: vec![ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    mutatable: true,
                    global_param_name: None,
                }],
            }],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let params = ParameterExtractor::extract_all(&candidate);

        let period_param = params
            .iter()
            .find(|p| p.name.contains("period"))
            .expect("Должен быть параметр period");

        assert_eq!(
            period_param.min,
            Some(2.0),
            "min для period должен быть 2.0"
        );
        assert_eq!(
            period_param.max,
            Some(4.0),
            "max для period должен быть 4.0 (не 10.0!)"
        );
        assert_eq!(
            period_param.step,
            Some(1.0),
            "step для period должен быть 1.0"
        );
    }

    #[test]
    fn test_extract_defaults_returns_empty() {
        let candidate = StrategyCandidate {
            indicators: vec![],
            nested_indicators: vec![],
            conditions: vec![ConditionInfo {
                id: "test_condition_1".to_string(),
                name: "Test RisingTrend".to_string(),
                operator: ConditionOperator::RisingTrend,
                primary_indicator_alias: "test_sma".to_string(),
                secondary_indicator_alias: None,
                condition_type: "trend_condition".to_string(),
                primary_timeframe: Some(TimeFrame::Minutes(60)),
                secondary_timeframe: None,
                price_field: None,
                constant_value: Some(3.0),
                optimization_params: vec![ConditionParamInfo {
                    name: "period".to_string(),
                    optimizable: true,
                    mutatable: true,
                    global_param_name: None,
                }],
            }],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let defaults = StrategyConverter::extract_defaults(&candidate);

        assert!(
            defaults.is_empty(),
            "extract_defaults должен возвращать пустой HashMap, так как все параметры должны передаваться явно"
        );
    }

    fn create_test_indicator(name: &str, alias: &str) -> IndicatorInfo {
        IndicatorInfo {
            name: name.to_string(),
            alias: alias.to_string(),
            parameters: vec![crate::discovery::types::IndicatorParamInfo {
                name: "period".to_string(),
                param_type: crate::indicators::types::ParameterType::Period,
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
    fn test_candidate_to_definition_basic() {
        let candidate = StrategyCandidate {
            indicators: vec![create_test_indicator("SMA", "sma")],
            nested_indicators: vec![],
            conditions: vec![create_test_condition(
                "cond1",
                "indicator_price",
                ConditionOperator::Above,
            )],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let result = StrategyConverter::candidate_to_definition(&candidate, TimeFrame::Minutes(60));
        assert!(result.is_ok());
        let definition = result.unwrap();
        assert_eq!(definition.indicator_bindings.len(), 1);
        assert_eq!(definition.condition_bindings.len(), 1);
        assert_eq!(definition.entry_rules.len(), 1);
    }

    #[test]
    fn test_candidate_to_definition_with_nested_indicators() {
        let base_indicator = create_test_indicator("EMA", "ema");
        let nested_indicator = NestedIndicator {
            indicator: create_test_indicator("SMA", "sma_on_ema"),
            input_indicator_alias: "ema".to_string(),
            depth: 1,
        };

        let mut condition =
            create_test_condition("cond1", "indicator_indicator", ConditionOperator::Above);
        condition.secondary_indicator_alias = Some("sma_on_ema".to_string());

        let candidate = StrategyCandidate {
            indicators: vec![base_indicator],
            nested_indicators: vec![nested_indicator],
            conditions: vec![condition],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let result = StrategyConverter::candidate_to_definition(&candidate, TimeFrame::Minutes(60));
        if let Err(e) = &result {
            eprintln!("Ошибка конвертации: {:?}", e);
        }
        assert!(result.is_ok());
        let definition = result.unwrap();
        assert_eq!(definition.indicator_bindings.len(), 2);
    }

    #[test]
    fn test_candidate_to_definition_with_exit_conditions() {
        let candidate = StrategyCandidate {
            indicators: vec![create_test_indicator("SMA", "sma")],
            nested_indicators: vec![],
            conditions: vec![create_test_condition(
                "entry1",
                "indicator_price",
                ConditionOperator::Above,
            )],
            exit_conditions: vec![create_test_condition(
                "exit1",
                "indicator_price",
                ConditionOperator::Below,
            )],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let result = StrategyConverter::candidate_to_definition(&candidate, TimeFrame::Minutes(60));
        assert!(result.is_ok());
        let definition = result.unwrap();
        assert_eq!(definition.exit_rules.len(), 1);
    }

    #[test]
    fn test_candidate_to_definition_empty_conditions() {
        let candidate = StrategyCandidate {
            indicators: vec![create_test_indicator("SMA", "sma")],
            nested_indicators: vec![],
            conditions: vec![],
            exit_conditions: vec![],
            stop_handlers: vec![],
            take_handlers: vec![],
            timeframes: vec![TimeFrame::Minutes(60)],
            config: StrategyDiscoveryConfig::default(),
        };

        let result = StrategyConverter::candidate_to_definition(&candidate, TimeFrame::Minutes(60));
        assert!(result.is_ok());
        let definition = result.unwrap();
        assert_eq!(definition.entry_rules.len(), 0);
    }
}
