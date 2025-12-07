use std::collections::HashMap;

use crate::data_model::types::TimeFrame;
use crate::discovery::engine::StrategyCandidate;
use crate::discovery::types::{ConditionInfo, IndicatorInfo};
use crate::strategy::types::{
    ConditionBindingSpec, ConditionDeclarativeSpec, ConditionInputSpec, ConditionOperator,
    IndicatorBindingSpec,
};

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
        let mut alias_to_timeframes: std::collections::HashMap<
            String,
            std::collections::HashSet<TimeFrame>,
        > = std::collections::HashMap::new();
        for binding in indicator_bindings {
            alias_to_timeframes
                .entry(binding.alias.clone())
                .or_default()
                .insert(binding.timeframe.clone());
        }

        match condition.condition_type.as_str() {
            "indicator_price" => {
                let indicator_alias = &condition.primary_indicator_alias;
                let price_field = if let Some(ref pf_str) = condition.price_field {
                    Self::parse_price_field_from_string(pf_str)
                        .unwrap_or_else(|| crate::strategy::types::PriceField::Close)
                } else {
                    Self::extract_price_field_from_condition_id(&condition.id)
                        .unwrap_or_else(|| crate::strategy::types::PriceField::Close)
                };

                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                        indicator_alias.clone(),
                        tf.clone(),
                    )
                } else if let Some(timeframes) = alias_to_timeframes.get(indicator_alias.as_str()) {
                    if let Some(tf) = timeframes.iter().next() {
                        crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                            indicator_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        crate::strategy::types::DataSeriesSource::indicator(indicator_alias)
                    }
                } else {
                    crate::strategy::types::DataSeriesSource::indicator(indicator_alias)
                };

                let secondary_source = if let Some(ref tf) = condition.secondary_timeframe {
                    crate::strategy::types::DataSeriesSource::price_with_timeframe(
                        price_field,
                        tf.clone(),
                    )
                } else {
                    crate::strategy::types::DataSeriesSource::price(price_field)
                };

                let percent_param = condition
                    .optimization_params
                    .iter()
                    .find(|p| p.name == "percent" || p.name == "percentage");

                if let Some(_percent_param) = percent_param {
                    let range =
                        crate::condition::parameters::ConditionParameterPresets::percentage();
                    let percent_value = ((range.min + range.max) / 2.0) as f32;
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
            "indicator_indicator" => {
                let primary_alias = &condition.primary_indicator_alias;
                let secondary_alias = condition.secondary_indicator_alias.as_ref().ok_or_else(
                    || StrategyConversionError::InvalidConditionFormat {
                        condition_id: condition.id.clone(),
                        reason:
                            "Missing secondary_indicator_alias for indicator_indicator condition"
                                .to_string(),
                    },
                )?;

                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                        primary_alias.clone(),
                        tf.clone(),
                    )
                } else if let Some(timeframes) = alias_to_timeframes.get(primary_alias) {
                    if let Some(tf) = timeframes.iter().next() {
                        crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                            primary_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        crate::strategy::types::DataSeriesSource::indicator(primary_alias.clone())
                    }
                } else {
                    crate::strategy::types::DataSeriesSource::indicator(primary_alias.clone())
                };

                let secondary_source = if let Some(ref tf) = condition.secondary_timeframe {
                    crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                        secondary_alias.clone(),
                        tf.clone(),
                    )
                } else if let Some(timeframes) = alias_to_timeframes.get(secondary_alias) {
                    if let Some(tf) = timeframes.iter().next() {
                        crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                            secondary_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        crate::strategy::types::DataSeriesSource::indicator(secondary_alias.clone())
                    }
                } else {
                    crate::strategy::types::DataSeriesSource::indicator(secondary_alias.clone())
                };

                let percent_param = condition
                    .optimization_params
                    .iter()
                    .find(|p| p.name == "percent" || p.name == "percentage");

                if let Some(_percent_param) = percent_param {
                    let range =
                        crate::condition::parameters::ConditionParameterPresets::percentage();
                    let percent_value = ((range.min + range.max) / 2.0) as f32;
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
            "trend_condition" => {
                let indicator_alias = &condition.primary_indicator_alias;

                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                        indicator_alias.clone(),
                        tf.clone(),
                    )
                } else if let Some(timeframes) = alias_to_timeframes.get(indicator_alias.as_str()) {
                    if let Some(tf) = timeframes.iter().next() {
                        crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                            indicator_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        crate::strategy::types::DataSeriesSource::indicator(indicator_alias)
                    }
                } else {
                    crate::strategy::types::DataSeriesSource::indicator(indicator_alias)
                };

                Ok(ConditionInputSpec::Single {
                    source: primary_source,
                })
            }
            "indicator_constant" => {
                let indicator_alias = &condition.primary_indicator_alias;
                let constant_value = condition.constant_value.unwrap_or(0.0) as f32;

                let primary_source = if let Some(ref tf) = condition.primary_timeframe {
                    crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                        indicator_alias.clone(),
                        tf.clone(),
                    )
                } else if let Some(timeframes) = alias_to_timeframes.get(indicator_alias.as_str()) {
                    if let Some(tf) = timeframes.iter().next() {
                        crate::strategy::types::DataSeriesSource::indicator_with_timeframe(
                            indicator_alias.clone(),
                            tf.clone(),
                        )
                    } else {
                        crate::strategy::types::DataSeriesSource::indicator(indicator_alias)
                    }
                } else {
                    crate::strategy::types::DataSeriesSource::indicator(indicator_alias)
                };

                Ok(ConditionInputSpec::Dual {
                    primary: primary_source,
                    secondary: crate::strategy::types::DataSeriesSource::custom(format!(
                        "constant_{}",
                        constant_value
                    )),
                })
            }
            _ => Err(StrategyConversionError::UnsupportedConditionType {
                condition_type: condition.condition_type.clone(),
            }),
        }
    }

    fn parse_price_field_from_string(
        price_field_str: &str,
    ) -> Option<crate::strategy::types::PriceField> {
        match price_field_str {
            "Open" => Some(crate::strategy::types::PriceField::Open),
            "High" => Some(crate::strategy::types::PriceField::High),
            "Low" => Some(crate::strategy::types::PriceField::Low),
            "Close" => Some(crate::strategy::types::PriceField::Close),
            "Volume" => Some(crate::strategy::types::PriceField::Volume),
            _ => None,
        }
    }

    fn extract_price_field_from_condition_id(
        condition_id: &str,
    ) -> Option<crate::strategy::types::PriceField> {
        if condition_id.starts_with("ind_price_") {
            let parts: Vec<&str> = condition_id.split('_').collect();
            if parts.len() >= 4 {
                match parts[3] {
                    "Open" => Some(crate::strategy::types::PriceField::Open),
                    "High" => Some(crate::strategy::types::PriceField::High),
                    "Low" => Some(crate::strategy::types::PriceField::Low),
                    "Close" => Some(crate::strategy::types::PriceField::Close),
                    "Volume" => Some(crate::strategy::types::PriceField::Volume),
                    _ => Some(crate::strategy::types::PriceField::Close),
                }
            } else {
                Some(crate::strategy::types::PriceField::Close)
            }
        } else {
            Some(crate::strategy::types::PriceField::Close)
        }
    }
}
