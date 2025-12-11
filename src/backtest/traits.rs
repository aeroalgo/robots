use crate::data_model::quote_frame::QuoteFrame;
use crate::data_model::types::TimeFrame;
use crate::strategy::base::Strategy;
use crate::strategy::context::StrategyContext;
use std::collections::HashMap;
use std::sync::Arc;

use super::BacktestError;

pub trait FeedProvider: Send + Sync {
    fn frames(&self) -> &HashMap<TimeFrame, Arc<QuoteFrame>>;
    fn primary_timeframe(&self) -> Option<&TimeFrame>;
    fn get_frame(&self, timeframe: &TimeFrame) -> Option<&Arc<QuoteFrame>>;
    fn step(&mut self, context: &mut StrategyContext) -> bool;
    fn reset(&mut self);
}

pub trait IndicatorComputer: Send + Sync {
    fn populate_indicators(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError>;

    fn populate_auxiliary_indicators(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError>;

    fn populate_custom_data(
        &mut self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError>;
}

pub trait ConditionEvaluatorTrait: Send + Sync {
    fn populate_conditions(
        &self,
        strategy: &dyn Strategy,
        frames: &HashMap<TimeFrame, Arc<QuoteFrame>>,
        context: &mut StrategyContext,
    ) -> Result<(), BacktestError>;
}

