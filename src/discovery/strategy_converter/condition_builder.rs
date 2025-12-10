use std::collections::{HashMap, HashSet};

use crate::data_model::types::TimeFrame;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo};
use crate::strategy::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    IndicatorBindingSpec, PriceField,
};

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
        let mut bindings = Vec::new();

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

            let mut tags = vec![condition.condition_type.clone()];
            if prefix == "exit" {
                tags.push("exit".to_string());
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
                parameters.insert("percent".to_string(), percent_value);
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
                parameters.insert("period".to_string(), period_value);
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
        candidate: &StrategyCandidate,
        indicator_bindings: &[IndicatorBindingSpec],
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        let alias_to_timeframes = ConverterHelpers::build_alias_to_timeframes_map(indicator_bindings);

        match condition.condition_type.as_str() {
            "indicator_price" => {
                Self::create_indicator_price_input(condition, &alias_to_timeframes)
            }
            "indicator_indicator" => {
                Self::create_indicator_indicator_input(condition, &alias_to_timeframes)
            }
            "trend_condition" => {
                Self::create_trend_condition_input(condition, &alias_to_timeframes)
            }
            "indicator_constant" => {
                Self::create_indicator_constant_input(condition, &alias_to_timeframes)
            }
            _ => Err(StrategyConversionError::UnsupportedConditionType {
                condition_type: condition.condition_type.clone(),
            }),
        }
    }

    fn create_indicator_price_input(
        condition: &ConditionInfo,
        alias_to_timeframes: &HashMap<String, HashSet<TimeFrame>>,
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        let indicator_alias = &condition.primary_indicator_alias;
        let price_field = ConverterHelpers::get_price_field_for_condition(condition);

        let primary_source = ConverterHelpers::create_indicator_source(
            indicator_alias,
            condition.primary_timeframe.as_ref(),
            alias_to_timeframes,
        );

        let secondary_source = ConverterHelpers::create_price_source(
            price_field,
            condition.secondary_timeframe.as_ref(),
        );

        if let Some(percent_value) = ConverterHelpers::extract_percent_param(condition) {
            Ok(ConditionInputSpec::DualWithPercent {
                primary: primary_source,
                secondary: secondary_source,
                percent: percent_value,
            })
        } else {
            Ok(ConditionInputSpec::Dual {
                primary: primary_source,
                secondary: secondary_source,
            })
        }
    }

    fn create_indicator_indicator_input(
        condition: &ConditionInfo,
        alias_to_timeframes: &HashMap<String, HashSet<TimeFrame>>,
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        let primary_alias = &condition.primary_indicator_alias;
        let secondary_alias = condition.secondary_indicator_alias.as_ref().ok_or_else(
            || StrategyConversionError::InvalidConditionFormat {
                condition_id: condition.id.clone(),
                reason:
                    "Missing secondary_indicator_alias for indicator_indicator condition"
                        .to_string(),
            },
        )?;

        let primary_source = ConverterHelpers::create_indicator_source(
            primary_alias,
            condition.primary_timeframe.as_ref(),
            alias_to_timeframes,
        );

        let secondary_source = ConverterHelpers::create_indicator_source(
            secondary_alias,
            condition.secondary_timeframe.as_ref(),
            alias_to_timeframes,
        );

        if let Some(percent_value) = ConverterHelpers::extract_percent_param(condition) {
            Ok(ConditionInputSpec::DualWithPercent {
                primary: primary_source,
                secondary: secondary_source,
                percent: percent_value,
            })
        } else {
            Ok(ConditionInputSpec::Dual {
                primary: primary_source,
                secondary: secondary_source,
            })
        }
    }

    fn create_trend_condition_input(
        condition: &ConditionInfo,
        alias_to_timeframes: &HashMap<String, HashSet<TimeFrame>>,
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        let indicator_alias = &condition.primary_indicator_alias;
        let primary_source = ConverterHelpers::create_indicator_source(
            indicator_alias,
            condition.primary_timeframe.as_ref(),
            alias_to_timeframes,
        );

        Ok(ConditionInputSpec::Single {
            source: primary_source,
        })
    }

    fn create_indicator_constant_input(
        condition: &ConditionInfo,
        alias_to_timeframes: &HashMap<String, HashSet<TimeFrame>>,
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        let indicator_alias = &condition.primary_indicator_alias;
        let constant_value = condition.constant_value.unwrap_or(0.0) as f32;

        let primary_source = ConverterHelpers::create_indicator_source(
            indicator_alias,
            condition.primary_timeframe.as_ref(),
            alias_to_timeframes,
        );

        Ok(ConditionInputSpec::Dual {
            primary: primary_source,
            secondary: crate::strategy::types::DataSeriesSource::custom(format!(
                "constant_{}",
                constant_value
            )),
        })
    }

}
