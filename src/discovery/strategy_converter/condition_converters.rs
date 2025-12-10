use crate::data_model::types::TimeFrame;
use crate::discovery::types::ConditionInfo;
use crate::strategy::types::{ConditionInputSpec, DataSeriesSource};
use std::collections::{HashMap, HashSet};

use super::helpers::ConverterHelpers;
use super::main::StrategyConversionError;

pub enum ConditionConverterType {
    IndicatorPrice,
    IndicatorIndicator,
    TrendCondition,
    IndicatorConstant,
}

impl ConditionConverterType {
    pub fn convert_to_input(
        &self,
        condition: &ConditionInfo,
        alias_to_timeframes: &HashMap<String, HashSet<TimeFrame>>,
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        match self {
            ConditionConverterType::IndicatorPrice => {
                Self::convert_indicator_price(condition, alias_to_timeframes)
            }
            ConditionConverterType::IndicatorIndicator => {
                Self::convert_indicator_indicator(condition, alias_to_timeframes)
            }
            ConditionConverterType::TrendCondition => {
                Self::convert_trend_condition(condition, alias_to_timeframes)
            }
            ConditionConverterType::IndicatorConstant => {
                Self::convert_indicator_constant(condition, alias_to_timeframes)
            }
        }
    }

    fn convert_indicator_price(
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

    fn convert_indicator_indicator(
        condition: &ConditionInfo,
        alias_to_timeframes: &HashMap<String, HashSet<TimeFrame>>,
    ) -> Result<ConditionInputSpec, StrategyConversionError> {
        let primary_alias = &condition.primary_indicator_alias;
        let secondary_alias = condition
            .secondary_indicator_alias
            .as_ref()
            .ok_or_else(|| StrategyConversionError::InvalidConditionFormat {
                condition_id: condition.id.clone(),
                reason: "Missing secondary_indicator_alias for indicator_indicator condition"
                    .to_string(),
            })?;

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

    fn convert_trend_condition(
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

    fn convert_indicator_constant(
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
            secondary: DataSeriesSource::custom(format!("constant_{}", constant_value)),
        })
    }
}

pub struct ConditionConverterFactory;

impl ConditionConverterFactory {
    pub fn get_converter(
        condition_type: &str,
    ) -> Result<ConditionConverterType, StrategyConversionError> {
        match condition_type {
            "indicator_price" => Ok(ConditionConverterType::IndicatorPrice),
            "indicator_indicator" => Ok(ConditionConverterType::IndicatorIndicator),
            "trend_condition" => Ok(ConditionConverterType::TrendCondition),
            "indicator_constant" => Ok(ConditionConverterType::IndicatorConstant),
            _ => Err(StrategyConversionError::UnsupportedConditionType {
                condition_type: condition_type.to_string(),
            }),
        }
    }
}
