use crate::discovery::types::{IndicatorInfo, NestedIndicator};
use crate::indicators::types::IndicatorCategory;
use crate::optimization::candidate_builder_config::ConditionProbabilities;
use crate::strategy::types::ConditionOperator;
use rand::Rng;

pub fn category_from_str(indicator_type: &str) -> IndicatorCategory {
    match indicator_type {
        "oscillator" => IndicatorCategory::Oscillator,
        "volatility" => IndicatorCategory::Volatility,
        "trend" => IndicatorCategory::Trend,
        "channel" => IndicatorCategory::Channel,
        "volume" => IndicatorCategory::Volume,
        _ => IndicatorCategory::Custom,
    }
}

pub fn select_condition_type(
    category: &IndicatorCategory,
    is_oscillator_used_in_nested: bool,
    is_built_on_oscillator: bool,
    probabilities: &ConditionProbabilities,
    rng: &mut impl Rng,
) -> &'static str {
    match category {
        IndicatorCategory::Oscillator => {
            if is_oscillator_used_in_nested {
                "indicator_indicator"
            } else {
                "indicator_constant"
            }
        }
        IndicatorCategory::Volatility => "indicator_constant",
        IndicatorCategory::Trend
        | IndicatorCategory::Channel
        | IndicatorCategory::Volume
        | IndicatorCategory::SupportResistance
        | IndicatorCategory::Custom => {
            if is_built_on_oscillator {
                "indicator_indicator"
            } else if rng.gen::<f64>() < probabilities.use_trend_condition {
                "trend_condition"
            } else if rng.gen::<f64>() < probabilities.use_indicator_indicator_condition {
                "indicator_indicator"
            } else {
                "indicator_price"
            }
        }
    }
}

pub fn select_operator(
    category: &IndicatorCategory,
    condition_type: &str,
    probabilities: &ConditionProbabilities,
    rng: &mut impl Rng,
) -> ConditionOperator {
    match category {
        IndicatorCategory::Oscillator => {
            if rng.gen_bool(0.5) {
                ConditionOperator::Above
            } else {
                ConditionOperator::Below
            }
        }
        IndicatorCategory::Volatility => {
            if rng.gen_bool(0.5) {
                ConditionOperator::GreaterPercent
            } else {
                ConditionOperator::LowerPercent
            }
        }
        IndicatorCategory::Trend
        | IndicatorCategory::Channel
        | IndicatorCategory::Volume
        | IndicatorCategory::SupportResistance
        | IndicatorCategory::Custom => {
            if condition_type == "trend_condition" {
                if rng.gen_bool(0.5) {
                    ConditionOperator::RisingTrend
                } else {
                    ConditionOperator::FallingTrend
                }
            } else if rng.gen::<f64>() < probabilities.use_crosses_operator {
                if rng.gen_bool(0.5) {
                    ConditionOperator::CrossesAbove
                } else {
                    ConditionOperator::CrossesBelow
                }
            } else if rng.gen_bool(0.5) {
                ConditionOperator::Above
            } else {
                ConditionOperator::Below
            }
        }
    }
}

pub struct OperatorSelectorFactory;

impl OperatorSelectorFactory {
    pub fn select_operator_and_condition_type(
        indicator: &IndicatorInfo,
        nested_indicators: &[NestedIndicator],
        all_indicators: &[&IndicatorInfo],
        probabilities: &ConditionProbabilities,
        rng: &mut impl Rng,
    ) -> (ConditionOperator, &'static str) {
        let is_oscillator_used_in_nested = nested_indicators
            .iter()
            .any(|nested| nested.input_indicator_alias == indicator.alias);

        let is_built_on_oscillator = nested_indicators
            .iter()
            .find(|n| n.indicator.alias == indicator.alias)
            .and_then(|nested| {
                all_indicators
                    .iter()
                    .find(|ind| ind.alias == nested.input_indicator_alias)
                    .map(|input| input.indicator_type == "oscillator")
            })
            .unwrap_or(false);

        let category = category_from_str(&indicator.indicator_type);

        let condition_type_str = select_condition_type(
            &category,
            is_oscillator_used_in_nested,
            is_built_on_oscillator,
            probabilities,
            rng,
        );

        let operator = select_operator(&category, condition_type_str, probabilities, rng);

        (operator, condition_type_str)
    }
}
