mod condition_evaluator;
mod engine;
mod feed_manager;
mod indicator_engine;

pub use condition_evaluator::ConditionEvaluator;
pub use engine::BacktestEngine;
pub use feed_manager::FeedManager;
pub use indicator_engine::IndicatorEngine;

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
