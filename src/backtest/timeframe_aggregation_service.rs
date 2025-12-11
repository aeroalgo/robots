use std::collections::HashMap;
use std::sync::Arc;

use crate::candles::aggregator::TimeFrameAggregator;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::strategy::base::Strategy;

use super::{BacktestError, FeedManager};

pub struct TimeFrameAggregationService;

impl TimeFrameAggregationService {
    pub fn new() -> Self {
        Self
    }

    pub fn collect_required_timeframes(strategy: &dyn Strategy) -> Vec<TimeFrame> {
        let mut required_timeframes: std::collections::HashSet<TimeFrame> =
            std::collections::HashSet::new();

        for binding in strategy.indicator_bindings() {
            required_timeframes.insert(binding.timeframe.clone());
        }

        for condition in strategy.conditions() {
            required_timeframes.insert(condition.timeframe.clone());
        }

        for requirement in strategy.timeframe_requirements() {
            required_timeframes.insert(requirement.timeframe.clone());
        }

        required_timeframes.into_iter().collect()
    }

    pub fn aggregate_required_timeframes(
        &self,
        base_frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        base_timeframes: &[TimeFrame],
        required_timeframes: &[TimeFrame],
    ) -> Result<HashMap<TimeFrame, QuoteFrame>, BacktestError> {
        let mut aggregated_frames = HashMap::new();

        for base_tf in base_timeframes {
            if let Some(base_frame) = base_frames.get(base_tf) {
                let all_required: Vec<TimeFrame> = required_timeframes
                    .iter()
                    .filter(|&tf| {
                        !base_frames.contains_key(tf)
                            && !aggregated_frames.contains_key(tf)
                            && FeedManager::is_higher_timeframe(tf, base_tf)
                            && FeedManager::is_multiple_of(base_tf, tf)
                    })
                    .cloned()
                    .collect();

                for target_tf in all_required {
                    match TimeFrameAggregator::aggregate(base_frame.as_ref(), target_tf.clone()) {
                        Ok(aggregated) => {
                            aggregated_frames.insert(target_tf, aggregated.frame);
                        }
                        Err(e) => {
                            return Err(BacktestError::Feed(format!(
                                "Failed to aggregate timeframe {:?} from {:?}: {}",
                                target_tf, base_tf, e
                            )));
                        }
                    }
                }
            }
        }

        Ok(aggregated_frames)
    }
}

impl Default for TimeFrameAggregationService {
    fn default() -> Self {
        Self::new()
    }
}

