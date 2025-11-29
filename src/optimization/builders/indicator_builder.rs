use crate::discovery::types::{IndicatorInfo, NestedIndicator};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;

use crate::optimization::candidate_builder_config::{CandidateBuilderConfig, ElementProbabilities};

pub struct IndicatorBuilder<'a> {
    config: &'a CandidateBuilderConfig,
    rng: &'a mut rand::rngs::ThreadRng,
}

impl<'a> IndicatorBuilder<'a> {
    pub fn new(config: &'a CandidateBuilderConfig, rng: &'a mut rand::rngs::ThreadRng) -> Self {
        Self { config, rng }
    }

    pub fn select_single_indicator(
        &mut self,
        available: &[IndicatorInfo],
        probabilities: &ElementProbabilities,
        exclude_aliases: &[String],
        is_phase_1: bool,
    ) -> Option<IndicatorInfo> {
        let exclude_set: HashSet<&str> = exclude_aliases.iter().map(|s| s.as_str()).collect();

        let excluded_indicators: Vec<String> = self.config.rules.excluded_indicators.clone();
        let excluded_indicators_set: HashSet<&str> =
            excluded_indicators.iter().map(|s| s.as_str()).collect();

        let available_filtered: Vec<&IndicatorInfo> = available
            .iter()
            .filter(|indicator| !exclude_set.contains(indicator.alias.as_str()))
            .filter(|indicator| {
                if excluded_indicators_set.contains(indicator.name.as_str()) {
                    return false;
                }
                if is_phase_1 {
                    if indicator.indicator_type == "volatility"
                        || indicator.indicator_type == "volume"
                    {
                        return false;
                    }
                }
                let weight = match indicator.indicator_type.as_str() {
                    "trend" => probabilities.indicators.add_trend_indicator,
                    "oscillator" => probabilities.indicators.add_oscillator_indicator,
                    "volume" => probabilities.indicators.add_volume_indicator,
                    "volatility" => probabilities.indicators.add_volatility_indicator,
                    "channel" => probabilities.indicators.add_channel_indicator,
                    _ => probabilities.indicators.add_base_indicator,
                };
                self.should_add(weight)
            })
            .collect();

        available_filtered
            .choose(&mut *self.rng)
            .map(|ind| (*ind).clone())
    }

    pub fn try_add_nested_indicator(
        &mut self,
        indicators: &[IndicatorInfo],
        nested_indicators: &mut Vec<NestedIndicator>,
        available_indicators: &[IndicatorInfo],
    ) -> bool {
        let add_nested_prob = self
            .config
            .probabilities
            .nested_indicators
            .add_nested_indicator;
        let max_depth = self
            .config
            .probabilities
            .nested_indicators
            .max_nesting_depth;

        if !self.should_add(add_nested_prob) {
            return false;
        }

        if indicators.is_empty() {
            return false;
        }

        let current_max_depth = nested_indicators.iter().map(|n| n.depth).max().unwrap_or(0);

        if current_max_depth >= max_depth {
            return false;
        }

        let base_indicators: Vec<&IndicatorInfo> = indicators
            .iter()
            .chain(nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        let Some(input_indicator) = base_indicators.choose(&mut *self.rng) else {
            return false;
        };

        let input_depth = nested_indicators
            .iter()
            .find(|n| n.indicator.alias == input_indicator.alias)
            .map(|n| n.depth)
            .unwrap_or(0);

        if input_depth >= max_depth {
            return false;
        }

        let nestable_indicators: Vec<&IndicatorInfo> = available_indicators
            .iter()
            .filter(|ind| ind.indicator_type == "trend")
            .filter(|ind| !self.config.rules.excluded_indicators.contains(&ind.name))
            .collect();

        let Some(wrapper_template) = nestable_indicators.choose(&mut *self.rng) else {
            return false;
        };

        let new_alias = format!("{}_on_{}", wrapper_template.alias, input_indicator.alias);

        let already_exists = nested_indicators
            .iter()
            .any(|n| n.indicator.alias == new_alias);

        if already_exists {
            return false;
        }

        let nested_indicator = NestedIndicator {
            indicator: IndicatorInfo {
                name: wrapper_template.name.clone(),
                alias: new_alias,
                parameters: wrapper_template.parameters.clone(),
                can_use_indicator_input: true,
                input_type: "indicator".to_string(),
                indicator_type: wrapper_template.indicator_type.clone(),
            },
            input_indicator_alias: input_indicator.alias.clone(),
            depth: input_depth + 1,
        };

        nested_indicators.push(nested_indicator);
        true
    }

    pub fn is_oscillator_used_in_nested(
        oscillator: &IndicatorInfo,
        nested_indicators: &[NestedIndicator],
    ) -> bool {
        nested_indicators
            .iter()
            .any(|nested| nested.input_indicator_alias == oscillator.alias)
    }

    fn should_add(&mut self, probability: f64) -> bool {
        self.rng.gen::<f64>() < probability
    }
}
