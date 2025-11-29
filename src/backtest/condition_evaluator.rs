use std::collections::HashMap;
use std::sync::Arc;

use crate::condition::types::ConditionResultData;
use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::strategy::base::Strategy;
use crate::strategy::context::{StrategyContext, TimeframeData};
use crate::strategy::types::StrategyError;

use super::BacktestError;

pub struct ConditionEvaluator;

impl ConditionEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn populate_conditions(
        &self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError> {
        let conditions_count = strategy.conditions().len();
        let mut grouped: HashMap<TimeFrame, Vec<usize>> =
            HashMap::with_capacity(conditions_count / 2 + 1);

        for (idx, condition) in strategy.conditions().iter().enumerate() {
            grouped
                .entry(condition.timeframe.clone())
                .or_default()
                .push(idx);
        }

        for (timeframe, condition_indices) in grouped {
            let frame = frames.get(&timeframe).ok_or_else(|| {
                BacktestError::Feed(format!(
                    "timeframe {:?} not available in feed for conditions",
                    timeframe
                ))
            })?;

            Self::ensure_timeframe_data(context, &timeframe, frame);

            let mut results: Vec<(usize, Arc<ConditionResultData>)> = Vec::new();

            for &condition_idx in &condition_indices {
                let condition = strategy
                    .conditions()
                    .get(condition_idx)
                    .ok_or_else(|| {
                        BacktestError::Feed(format!(
                            "condition at index {} not found",
                            condition_idx
                        ))
                    })?;

                {
                    let data = context
                        .timeframe_mut(&timeframe)
                        .map_err(|e| BacktestError::Strategy(e))?;
                    data.register_condition_id(condition.id.clone(), condition_idx);
                }

                let input = context
                    .prepare_condition_input(condition)
                    .map_err(|err| BacktestError::Strategy(err))?;

                let result = condition.condition.check(input).map_err(|err| {
                    BacktestError::Strategy(StrategyError::ConditionFailure {
                        condition_id: condition.id.clone(),
                        source: err,
                    })
                })?;

                results.push((condition_idx, Arc::new(result)));
            }

            {
                let data = context
                    .timeframe_mut(&timeframe)
                    .map_err(|e| BacktestError::Strategy(e))?;
                for (condition_idx, result) in results {
                    data.insert_condition_result_by_index(condition_idx, result);
                }
            }
        }

        Ok(())
    }

    fn ensure_timeframe_data(
        context: &mut StrategyContext,
        timeframe: &TimeFrame,
        frame: &Arc<QuoteFrame>,
    ) {
        if context.timeframe(timeframe).is_err() {
            let data = TimeframeData::with_quote_frame(frame.as_ref(), 0);
            context.insert_timeframe(timeframe.clone(), data);
        }
    }
}

impl Default for ConditionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}
