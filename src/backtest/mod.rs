mod backtest_orchestrator;
mod condition_evaluator;
mod config_builder;
mod constants;
mod engine;
mod equity_calculator;
mod feed_manager;
mod helpers;
mod indicator_engine;
mod session_manager;
mod timeframe_aggregation_service;
mod traits;

#[cfg(test)]
mod tests;

pub use backtest_orchestrator::BacktestOrchestrator;
pub use condition_evaluator::ConditionEvaluator;
pub use config_builder::BacktestConfigBuilder;
pub use engine::BacktestEngine;
pub use equity_calculator::EquityCalculator;
pub use feed_manager::FeedManager;
pub use indicator_engine::IndicatorEngine;
pub use session_manager::{SessionManager, SessionState};
pub use timeframe_aggregation_service::TimeFrameAggregationService;
pub use traits::{ConditionEvaluatorTrait, FeedProvider, IndicatorComputer};

use thiserror::Error;

use crate::position::PositionError;
use crate::strategy::types::StrategyError;

#[derive(Debug, Error)]
pub enum BacktestError {
    #[error("strategy evaluation error: {0}")]
    Strategy(#[from] StrategyError),
    #[error("position manager error: {0}")]
    Position(#[from] PositionError),
    #[error("feed error: {0}")]
    Feed(String),
}

#[derive(Clone, Debug)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub use_full_capital: bool,
    pub reinvest_profits: bool,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10000.0,
            use_full_capital: false,
            reinvest_profits: false,
        }
    }
}
