use crate::data_model::types::TimeFrame;
use crate::discovery::types::{ConditionInfo, IndicatorInfo, NestedIndicator};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::optimization::builders::ConditionBuilder;
use crate::optimization::candidate_builder_config::{
    CandidateBuilderConfig, ElementConstraints, TimeframeProbabilities,
};

pub struct TimeframeBuilder<'a> {
    config: &'a CandidateBuilderConfig,
    rng: &'a mut rand::rngs::ThreadRng,
}

impl<'a> TimeframeBuilder<'a> {
    pub fn new(config: &'a CandidateBuilderConfig, rng: &'a mut rand::rngs::ThreadRng) -> Self {
        Self { config, rng }
    }

    pub fn add_higher_timeframes_with_probability(
        &mut self,
        timeframes: &mut Vec<TimeFrame>,
        available_timeframes: &[TimeFrame],
        constraints: &ElementConstraints,
        timeframe_probs: &TimeframeProbabilities,
    ) {
        if available_timeframes.len() <= 1 {
            return;
        }

        if timeframes.len() < constraints.max_timeframes
            && self.should_add(timeframe_probs.use_higher_timeframe)
        {
            let higher_timeframes: Vec<&TimeFrame> = available_timeframes
                .iter()
                .skip(1)
                .filter(|tf| !timeframes.contains(tf))
                .collect();

            if let Some(higher_tf) = higher_timeframes.choose(&mut *self.rng) {
                timeframes.push((*higher_tf).clone());
            }
        }

        if available_timeframes.len() > 2
            && timeframes.len() >= 2
            && timeframes.len() < constraints.max_timeframes
            && self.should_add(timeframe_probs.use_multiple_timeframes)
        {
            let remaining_timeframes: Vec<&TimeFrame> = available_timeframes
                .iter()
                .filter(|tf| !timeframes.contains(tf))
                .collect();

            if let Some(tf) = remaining_timeframes.choose(&mut *self.rng) {
                timeframes.push((*tf).clone());
            }
        }
    }

    pub fn ensure_higher_timeframes_used(
        &mut self,
        timeframes: &[TimeFrame],
        available_timeframes: &[TimeFrame],
        indicators: &[IndicatorInfo],
        nested_indicators: &[NestedIndicator],
        entry_conditions: &mut Vec<ConditionInfo>,
        exit_conditions: &[ConditionInfo],
        constraints: &ElementConstraints,
    ) {
        if available_timeframes.is_empty() || timeframes.is_empty() {
            return;
        }

        let base_timeframe = &available_timeframes[0];
        let higher_timeframes: Vec<&TimeFrame> = timeframes
            .iter()
            .filter(|tf| *tf != base_timeframe)
            .collect();

        if higher_timeframes.is_empty() {
            return;
        }

        let all_indicators: Vec<&IndicatorInfo> = indicators
            .iter()
            .chain(nested_indicators.iter().map(|n| &n.indicator))
            .collect();

        if all_indicators.is_empty() {
            return;
        }

        for higher_tf in higher_timeframes {
            if entry_conditions.len() >= constraints.max_entry_conditions {
                break;
            }

            let is_used = entry_conditions
                .iter()
                .chain(exit_conditions.iter())
                .any(|cond| {
                    cond.primary_timeframe.as_ref() == Some(higher_tf)
                        || cond.secondary_timeframe.as_ref() == Some(higher_tf)
                });

            if !is_used {
                if let Some(indicator) = all_indicators.choose(&mut *self.rng) {
                    let mut condition_builder =
                        ConditionBuilder::new(self.config, self.rng);
                    let condition = condition_builder.build_condition_simple_with_timeframe(
                        indicator,
                        true,
                        Some(higher_tf.clone()),
                    );
                    if let Some(cond) = condition {
                        if !ConditionBuilder::is_duplicate_condition(&cond, entry_conditions)
                            && !ConditionBuilder::has_conflicting_comparison_operator(
                                &cond,
                                entry_conditions,
                            )
                        {
                            entry_conditions.push(cond);
                        }
                    }
                }
            }
        }
    }

    fn should_add(&mut self, probability: f64) -> bool {
        self.rng.gen::<f64>() < probability
    }
}
